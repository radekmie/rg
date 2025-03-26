mod analyses;
mod display;
mod transforms;
mod validators;

use analyses::ReachableNodes;
use map_id::MapId;
use map_id_macro::MapId;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::sync::Arc;
use utils::position::Span;

pub type Binding<'a, Id> = (&'a Id, &'a Arc<Type<Id>>);
pub type ExprOrType<'a, Id> = Result<&'a Arc<Expression<Id>>, &'a Arc<Type<Id>>>;
pub type Mapping<Id> = BTreeMap<Id, (Id, Arc<Type<Id>>)>;
pub type SetWithIdx<'a, T> = BTreeSet<(usize, &'a T)>;

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "ConstantDeclaration", tag = "kind")]
pub struct Constant<Id> {
    #[serde(skip)]
    pub span: Span,
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Arc<Type<Id>>,
    pub value: Arc<Value<Id>>,
}

impl<Id> Constant<Id> {
    pub fn new(span: Span, identifier: Id, type_: Arc<Type<Id>>, value: Arc<Value<Id>>) -> Self {
        Self {
            span,
            identifier,
            type_,
            value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "EdgeDeclaration", tag = "kind")]
pub struct Edge<Id> {
    #[serde(skip)]
    pub span: Span,
    pub label: Label<Id>,
    pub lhs: Node<Id>,
    pub rhs: Node<Id>,
}

impl<Id> Edge<Id> {
    pub fn new(lhs: Node<Id>, rhs: Node<Id>, label: Label<Id>) -> Self {
        Self {
            span: Span::none(),
            label,
            lhs,
            rhs,
        }
    }

    pub fn is_conditional(&self) -> bool {
        self.label.is_comparison() || self.label.is_reachability()
    }

    pub fn new_skip(lhs: Node<Id>, rhs: Node<Id>) -> Self {
        Self::new(lhs, rhs, Label::new_skip())
    }

    pub fn skip(&mut self) {
        self.label = Label::Skip { span: self.span };
    }
}

impl<Id: Display> Edge<Id> {
    pub fn to_graphviz(&self) -> String {
        let Self {
            label, lhs, rhs, ..
        } = self;
        format!("  \"{lhs}\" -> \"{rhs}\" [label=\"{label}\"];")
    }
}

impl<Id: Ord> Edge<Id> {
    pub fn cmp_outgoing(&self, other: &Self) -> Ordering {
        self.lhs
            .cmp(&other.lhs)
            .then_with(|| self.rhs.cmp(&other.rhs))
            .then_with(|| self.label.cmp(&other.label))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "EdgeLabel", tag = "kind")]
pub enum Label<Id> {
    Assignment {
        lhs: Arc<Expression<Id>>,
        rhs: Arc<Expression<Id>>,
    },
    AssignmentAny {
        lhs: Arc<Expression<Id>>,
        rhs: Arc<Type<Id>>,
    },
    Comparison {
        lhs: Arc<Expression<Id>>,
        rhs: Arc<Expression<Id>>,
        negated: bool,
    },
    Reachability {
        #[serde(skip)]
        span: Span,
        lhs: Node<Id>,
        rhs: Node<Id>,
        negated: bool,
    },
    Skip {
        #[serde(skip)]
        span: Span,
    },
    Tag {
        symbol: Id,
    },
    TagVariable {
        identifier: Id,
    },
}

impl<Id> Label<Id> {
    pub fn as_var_assignment(&self) -> Option<&Id> {
        if let Self::Assignment { lhs, .. } | Self::AssignmentAny { lhs, .. } = self {
            return Some(lhs.access_identifier());
        }
        None
    }

    pub fn as_assignment(&self) -> Option<(&Id, ExprOrType<Id>)> {
        if let Self::Assignment { lhs, rhs } = self {
            return Some((lhs.access_identifier(), Ok(rhs)));
        } else if let Self::AssignmentAny { lhs, rhs } = self {
            return Some((lhs.access_identifier(), Err(rhs)));
        }
        None
    }

    pub fn is_assignment(&self) -> bool {
        matches!(self, Self::Assignment { .. } | Self::AssignmentAny { .. })
    }

    pub fn is_comparison(&self) -> bool {
        matches!(self, Self::Comparison { .. })
    }

    pub fn is_map_assignment(&self) -> bool {
        matches!(self, Self::Assignment { lhs, .. } | Self::AssignmentAny {lhs, .. } if lhs.uncast().is_access())
    }

    pub fn is_reachability(&self) -> bool {
        matches!(self, Self::Reachability { .. })
    }

    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip { .. })
    }

    pub fn is_tag(&self) -> bool {
        matches!(self, Self::Tag { .. })
    }

    pub fn is_tag_and(&self, fn_: impl FnOnce(&Id) -> bool) -> bool {
        matches!(self, Self::Tag { symbol } if fn_(symbol))
    }

    pub fn is_tag_variable(&self) -> bool {
        matches!(self, Self::TagVariable { .. })
    }

    pub fn negate(&mut self) {
        match self {
            Self::Comparison { negated, .. } => *negated = !*negated,
            Self::Reachability { negated, .. } => *negated = !*negated,
            _ => (),
        }
    }

    pub fn new_skip() -> Self {
        Self::Skip { span: Span::none() }
    }
}

impl<Id: Clone + PartialEq> Label<Id> {
    pub fn remove_casts(&self, identifier: &Id) -> Self {
        match self {
            Self::Assignment { lhs, rhs } => Self::Assignment {
                lhs: Arc::new(lhs.remove_casts(identifier)),
                rhs: Arc::new(rhs.remove_casts(identifier)),
            },
            Self::Comparison { lhs, rhs, negated } => Self::Comparison {
                lhs: Arc::new(lhs.remove_casts(identifier)),
                rhs: Arc::new(rhs.remove_casts(identifier)),
                negated: *negated,
            },
            _ => self.clone(),
        }
    }

    pub fn substitute_variable(&self, id: &Id, expression: &Expression<Id>) -> Self {
        match self {
            Self::Assignment { lhs, rhs } => Self::Assignment {
                lhs: Arc::new(lhs.substitute_variable(id, expression)),
                rhs: Arc::new(rhs.substitute_variable(id, expression)),
            },
            Self::Comparison { lhs, rhs, negated } => Self::Comparison {
                lhs: Arc::new(lhs.substitute_variable(id, expression)),
                rhs: Arc::new(rhs.substitute_variable(id, expression)),
                negated: *negated,
            },
            _ => self.clone(),
        }
    }

    /// Like `substitute_variable`, but does not replace `x` in `x = z` or `x[y] = z`
    pub fn substitute_variable_readonly(&self, id: &Id, expression: &Expression<Id>) -> Self {
        match self {
            Self::Assignment { lhs, rhs } => {
                let lhs = Arc::new(lhs.substitute_variable_readonly(id, expression));
                let rhs = Arc::new(rhs.substitute_variable(id, expression));
                Self::Assignment { lhs, rhs }
            }
            _ => self.substitute_variable(id, expression),
        }
    }
}

impl<Id: Ord> Label<Id> {
    pub fn used_variables(&self) -> BTreeSet<&Id> {
        let mut vars = BTreeSet::new();
        match self {
            Self::Assignment { lhs, rhs } if lhs.is_reference() => rhs.collect_variables(&mut vars),
            Self::Assignment { lhs, rhs } => {
                lhs.collect_variables(&mut vars);
                rhs.collect_variables(&mut vars);
            }
            Self::Comparison { lhs, rhs, .. } => {
                lhs.collect_variables(&mut vars);
                rhs.collect_variables(&mut vars);
            }
            _ => {}
        }
        vars
    }
}

impl<Id: PartialEq> Label<Id> {
    pub fn has_binding(&self, binding: &Id) -> bool {
        self.has_variable(binding) || self.is_tag_and(|tag| tag == binding)
    }

    pub fn has_variable(&self, identifier: &Id) -> bool {
        matches!(self, Self::Assignment { lhs, rhs } | Self::Comparison { lhs, rhs, .. } if lhs.has_variable(identifier) || rhs.has_variable(identifier))
    }

    pub fn is_negated(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Comparison {
                    lhs: l1,
                    rhs: r1,
                    negated: n1,
                    ..
                },
                Self::Comparison {
                    lhs: l2,
                    rhs: r2,
                    negated: n2,
                    ..
                },
            ) => l1 == l2 && r1 == r2 && n1 != n2,
            (
                Self::Reachability {
                    lhs: l1,
                    rhs: r1,
                    negated: n1,
                    ..
                },
                Self::Reachability {
                    lhs: l2,
                    rhs: r2,
                    negated: n2,
                    ..
                },
            ) => l1 == l2 && r1 == r2 && n1 != n2,
            _ => false,
        }
    }

    pub fn is_self_assignment(&self) -> bool {
        matches!(self, Self::Assignment { lhs, rhs } if lhs.is_equal_reference(rhs))
    }
}

impl Label<Arc<str>> {
    pub fn is_goals_assignment(&self) -> bool {
        matches!(self, Self::Assignment { lhs, .. } if lhs.access_identifier().as_ref() == "goals")
    }

    pub fn is_player_assignment(&self) -> bool {
        matches!(self, Self::Assignment { lhs, .. } if lhs.uncast().is_player_reference())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "EdgeName", tag = "kind")]
pub struct Node<Id> {
    pub identifier: Id,
}

impl<Id> Node<Id> {
    pub fn new(identifier: Id) -> Self {
        Self { identifier }
    }
}

impl Node<Arc<str>> {
    pub fn is_begin(&self) -> bool {
        self.identifier.as_ref() == "begin"
    }

    pub fn is_end(&self) -> bool {
        self.identifier.as_ref() == "end"
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Error<Id> {
    pub game: Option<Game<Id>>,
    pub reason: ErrorReason<Id>,
}

// Simplify error handling in WASM modules.
impl<Id: Display> From<Error<Id>> for String {
    fn from(error: Error<Id>) -> Self {
        format!("{error}")
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ErrorReason<Id> {
    ArrowTypeExpected {
        got: Arc<Type<Id>>,
    },
    AssignmentTypeMismatch {
        lhs: Arc<Type<Id>>,
        rhs: Arc<Type<Id>>,
    },
    ComparisonTypeMismatch {
        lhs: Arc<Type<Id>>,
        rhs: Arc<Type<Id>>,
    },
    ConstantAssignment {
        identifier: Id,
        label: Label<Id>,
    },
    DuplicatedConstant {
        identifier: Id,
    },
    DuplicatedMapKey {
        key: Option<Id>,
        value: Value<Id>,
    },
    DuplicatedTypedef {
        identifier: Id,
    },
    DuplicatedVariable {
        identifier: Id,
    },
    EmptySetType {
        identifier: Id,
    },
    MissingDefaultValue {
        value: Value<Id>,
    },
    MultipleEdges {
        lhs: Node<Id>,
        rhs: Node<Id>,
    },
    PlayerAnyAssignment {
        label: Label<Id>,
    },
    ReachabilityLoop {
        lhs: Node<Id>,
        rhs: Node<Id>,
    },
    SetTypeExpected {
        got: Arc<Type<Id>>,
    },
    TypeDeclarationMismatch {
        expected: Arc<Type<Id>>,
        identifier: Id,
        resolved: Arc<Type<Id>>,
    },
    Unreachable {
        lhs: Node<Id>,
        rhs: Node<Id>,
    },
    UnresolvedConstant {
        identifier: Id,
    },
    UnresolvedType {
        identifier: Id,
    },
    UnresolvedVariable {
        identifier: Id,
    },
    VariableDeclarationMismatch {
        expected: Arc<Type<Id>>,
        identifier: Id,
        resolved: Arc<Type<Id>>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Expression<Id> {
    Access {
        #[serde(skip)]
        span: Span,
        lhs: Arc<Self>,
        rhs: Arc<Self>,
    },
    Cast {
        #[serde(skip)]
        span: Span,
        lhs: Arc<Type<Id>>,
        rhs: Arc<Self>,
    },
    Reference {
        identifier: Id,
    },
}

impl<Id> Expression<Id> {
    pub fn access_identifier(&self) -> &Id {
        match self {
            Self::Access { lhs, .. } => lhs.access_identifier(),
            Self::Cast { rhs, .. } => rhs.access_identifier(),
            Self::Reference { identifier } => identifier,
        }
    }

    pub fn as_reference(&self) -> Option<&Id> {
        match self {
            Self::Reference { identifier } => Some(identifier),
            _ => None,
        }
    }

    pub fn new(identifier: Id) -> Self {
        Self::Reference { identifier }
    }

    pub fn new_cast(lhs: Arc<Type<Id>>, rhs: Arc<Self>) -> Self {
        Self::Cast {
            span: Span::none(),
            lhs,
            rhs,
        }
    }

    pub fn is_access(&self) -> bool {
        matches!(self, Self::Access { .. })
    }

    pub fn is_cast(&self) -> bool {
        matches!(self, Self::Cast { .. })
    }

    pub fn is_cast_and(&self, fn_: impl FnOnce(&Arc<Type<Id>>, &Arc<Self>) -> bool) -> bool {
        matches!(self, Self::Cast { lhs, rhs, .. } if fn_(lhs, rhs))
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Self::Reference { .. })
    }

    pub fn is_reference_and(&self, fn_: impl FnOnce(&Id) -> bool) -> bool {
        matches!(self, Self::Reference { identifier } if fn_(identifier))
    }

    pub fn uncast(&self) -> &Self {
        match self {
            Self::Cast { rhs, .. } => rhs.uncast(),
            _ => self,
        }
    }
}

impl<Id: Clone + PartialEq> Expression<Id> {
    pub fn infer(&self, game: &Game<Id>) -> Result<Arc<Type<Id>>, Error<Id>> {
        match self {
            Self::Access { lhs, rhs, .. } => {
                let accessed_type = lhs.infer(game)?;
                let Type::Arrow {
                    lhs: key_type,
                    rhs: value_type,
                } = accessed_type.resolve(game)?
                else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: accessed_type });
                };

                let accessor_type = rhs.infer(game)?;
                if !accessor_type.resolve(game)?.is_set() {
                    return game.make_error(ErrorReason::SetTypeExpected { got: accessor_type });
                }

                if !game.is_assignable_type(key_type, &accessor_type, false)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch {
                        lhs: key_type.clone(),
                        rhs: accessor_type,
                    });
                }

                Ok(value_type.clone())
            }
            Self::Cast { lhs, rhs, .. } => {
                let rhs = rhs.infer(game)?;
                if !game.is_assignable_type(lhs, &rhs, false)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch {
                        lhs: lhs.clone(),
                        rhs,
                    });
                }

                Ok(lhs.clone())
            }
            Self::Reference { identifier } => Ok(game.infer(identifier)),
        }
    }

    pub fn remove_casts(&self, identifier: &Id) -> Self {
        match self {
            Self::Access { lhs, rhs, .. } => Self::Access {
                span: Span::none(),
                lhs: Arc::new(lhs.remove_casts(identifier)),
                rhs: Arc::new(rhs.remove_casts(identifier)),
            },
            Self::Cast { lhs, rhs, .. } if lhs.is_identifier(identifier) => {
                rhs.remove_casts(identifier)
            }
            Self::Cast { lhs, rhs, .. } => Self::Cast {
                lhs: lhs.clone(),
                rhs: Arc::new(rhs.remove_casts(identifier)),
                span: Span::none(),
            },
            Self::Reference { identifier } => Self::Reference {
                identifier: identifier.clone(),
            },
        }
    }

    pub fn substitute_variable(&self, identifier: &Id, expression: &Self) -> Self {
        match self {
            Self::Access { lhs, rhs, .. } => Self::Access {
                span: Span::none(),
                lhs: Arc::new(lhs.substitute_variable(identifier, expression)),
                rhs: Arc::new(rhs.substitute_variable(identifier, expression)),
            },
            Self::Cast { lhs, rhs, .. } => Self::Cast {
                span: Span::none(),
                lhs: lhs.clone(),
                rhs: Arc::new(rhs.substitute_variable(identifier, expression)),
            },
            Self::Reference { identifier: x } if x == identifier => expression.clone(),
            Self::Reference { identifier } => Self::Reference {
                identifier: identifier.clone(),
            },
        }
    }

    pub fn substitute_variable_readonly(&self, identifier: &Id, expression: &Self) -> Self {
        match self {
            Self::Access { lhs, rhs, .. } => Self::Access {
                span: Span::none(),
                lhs: Arc::new(lhs.substitute_variable_readonly(identifier, expression)),
                rhs: Arc::new(rhs.substitute_variable(identifier, expression)),
            },
            Self::Cast { lhs, rhs, .. } => Self::Cast {
                span: Span::none(),
                lhs: lhs.clone(),
                rhs: Arc::new(rhs.substitute_variable_readonly(identifier, expression)),
            },
            Self::Reference { identifier } => Self::Reference {
                identifier: identifier.clone(),
            },
        }
    }
}

impl<Id: Ord> Expression<Id> {
    pub fn collect_variables<'a>(&'a self, vars: &mut BTreeSet<&'a Id>) {
        match self {
            Self::Access { lhs, rhs, .. } => {
                lhs.collect_variables(vars);
                rhs.collect_variables(vars);
            }
            Self::Cast { rhs, .. } => rhs.collect_variables(vars),
            Self::Reference { identifier } => {
                vars.insert(identifier);
            }
        }
    }

    pub fn used_variables(&self) -> BTreeSet<&Id> {
        let mut vars = BTreeSet::new();
        self.collect_variables(&mut vars);
        vars
    }
}

impl<Id: PartialEq> Expression<Id> {
    pub fn has_variable(&self, identifier: &Id) -> bool {
        match self {
            Self::Access { lhs, rhs, .. } => {
                lhs.has_variable(identifier) || rhs.has_variable(identifier)
            }
            Self::Cast { rhs, .. } => rhs.has_variable(identifier),
            Self::Reference { identifier: x } => x == identifier,
        }
    }

    pub fn is_equal_reference(&self, other: &Self) -> bool {
        match (self.uncast(), other.uncast()) {
            (
                Self::Access {
                    lhs: x_lhs,
                    rhs: x_rhs,
                    ..
                },
                Self::Access {
                    lhs: y_lhs,
                    rhs: y_rhs,
                    ..
                },
            ) => x_lhs.is_equal_reference(y_lhs) && x_rhs.is_equal_reference(y_rhs),
            (Self::Reference { identifier: x }, Self::Reference { identifier: y }) => x == y,
            _ => false,
        }
    }
}

impl Expression<Arc<str>> {
    pub fn is_player_reference(&self) -> bool {
        matches!(self, Self::Reference { identifier } if &**identifier == "player")
    }
}

#[cfg(test)]
mod expression {
    mod is_equal_reference {
        use crate::parsing::parser::parse_expression;

        fn check(lhs: &str, rhs: &str, expected: bool) {
            let lhs = parse_expression(lhs);
            let rhs = parse_expression(rhs);
            assert_eq!(lhs.is_equal_reference(&rhs), expected);
        }

        #[test]
        fn references() {
            check("x", "x", true);
            check("x", "y", false);
        }

        #[test]
        fn references_with_casts() {
            check("x", "T(x)", true);
            check("T(x)", "x", true);
            check("T(x)", "T(x)", true);

            check("x", "T(y)", false);
            check("T(x)", "y", false);
            check("T(x)", "T(y)", false);
        }

        #[test]
        fn accesses() {
            check("x[y]", "x[y]", true);
            check("x[y]", "z[y]", false);
            check("x[y]", "x[z]", false);
        }

        #[test]
        fn accesses_with_casts() {
            check("x[y]", "T(x[y])", true);
            check("T(x[y])", "x[y]", true);
            check("T(x[y])", "T(x[y])", true);

            check("x[y]", "T(z[y])", false);
            check("T(x[y])", "z[y]", false);
            check("T(x[y])", "T(z[y])", false);

            check("x[y]", "T(x[z])", false);
            check("T(x[y])", "x[z]", false);
            check("T(x[y])", "T(x[z])", false);
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "GameDeclaration", tag = "kind")]
pub struct Game<Id> {
    pub constants: Vec<Constant<Id>>,
    pub edges: Vec<Arc<Edge<Id>>>,
    pub pragmas: Vec<Pragma<Id>>,
    #[serde(rename = "types")]
    pub typedefs: Vec<Typedef<Id>>,
    pub variables: Vec<Variable<Id>>,
}

impl<Id: Clone> Game<Id> {
    pub fn make_error<T>(&self, reason: ErrorReason<Id>) -> Result<T, Error<Id>> {
        Err(Error {
            game: Some(self.clone()),
            reason,
        })
    }
}

impl<Id: Display> Game<Id> {
    pub fn to_graphviz(&self) -> String {
        let mut graphviz = String::new();
        graphviz.push_str("digraph {\n");
        graphviz.push_str("  pad=0.25;\n");
        graphviz.push_str("  edge [fontname=Helvetica];\n");
        graphviz.push_str("  node [fillcolor=\"#f0f0f0\", fontname=Helvetica, penwidth=0, shape=box, style=\"filled,rounded\"];\n");

        for edge in &self.edges {
            graphviz.push_str(&edge.to_graphviz());
            graphviz.push('\n');
        }

        for pragma in &self.pragmas {
            if let Pragma::Repeat { nodes, .. }
            | Pragma::TagIndex { nodes, .. }
            | Pragma::Unique { nodes, .. } = pragma
            {
                graphviz.push_str("  ");
                for (index, node) in nodes.iter().enumerate() {
                    let separator = if index == 0 { "" } else { ", " };
                    graphviz.push_str(&format!("{separator}\"{node}\""));
                }

                match pragma {
                    Pragma::Repeat { .. } => graphviz.push_str(" [fillcolor=\"#f6f7c4\"];\n"),
                    Pragma::TagIndex { index, .. } => {
                        graphviz.push_str(&format!(" [penwidth={}];\n", 1 + index));
                    }
                    Pragma::Unique { .. } => graphviz.push_str(" [fillcolor=\"#caedff\"];\n"),
                    _ => {}
                }
            }
        }

        graphviz.push('}');
        graphviz
    }
}

impl<'a, Id: Clone + Ord + 'a> Game<Id> {
    pub fn create_mappings(
        &self,
        bindings: impl Iterator<Item = Binding<'a, Id>>,
    ) -> Result<Vec<Mapping<Id>>, Error<Id>> {
        let mut mappings = vec![];
        for (identifier, type_) in bindings {
            let values = type_.values(self)?;

            // Seed with an empty mapping.
            if mappings.is_empty() {
                mappings.push(BTreeMap::default());
            }

            // Cartesian product of `mappings` with `values`.
            mappings = mappings
                .into_iter()
                .flat_map(|mapping| {
                    values.iter().map(move |value| {
                        let mut mapping = mapping.clone();
                        mapping.insert(identifier.clone(), (value.clone(), type_.clone()));
                        mapping
                    })
                })
                .collect();
        }

        Ok(mappings)
    }
}

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn infer(&self, identifier: &Id) -> Arc<Type<Id>> {
        self.infer_or_none(identifier).cloned().unwrap_or_else(|| {
            Arc::new(Type::Set {
                span: Span::none(),
                identifiers: vec![identifier.clone()],
            })
        })
    }

    pub fn infer_or_none<'a>(&'a self, identifier: &Id) -> Option<&'a Arc<Type<Id>>> {
        self.resolve_constant(identifier)
            .map(|x| &x.type_)
            .or_else(|| self.resolve_variable(identifier).map(|x| &x.type_))
    }

    pub fn is_assignable_identifier(
        &self,
        lhs: &Arc<Type<Id>>,
        rhs: &Id,
    ) -> Result<bool, Error<Id>> {
        if let Some(rhs) = self.infer_or_none(rhs) {
            // If `rhs` resolves to some type, it has to be assignable.
            self.is_assignable_type(lhs, rhs, false)
        } else if let Type::Set { identifiers, .. } = lhs.resolve(self)? {
            // If the `lhs` is a `Set`, `rhs` has to be its element.
            Ok(identifiers.contains(rhs))
        } else {
            Ok(false)
        }
    }

    pub fn is_assignable_type(
        &self,
        lhs: &Type<Id>,
        rhs: &Type<Id>,
        is_strict: bool,
    ) -> Result<bool, Error<Id>> {
        // Fast path for exact match.
        if lhs == rhs {
            return Ok(true);
        }

        Ok(match (lhs, rhs) {
            (Type::Arrow { lhs: ll, rhs: lr }, Type::Arrow { lhs: rl, rhs: rr }) => {
                self.is_assignable_type(ll, rl, is_strict)?
                    && self.is_assignable_type(lr, rr, is_strict)?
            }
            (
                Type::Set {
                    identifiers: lhs, ..
                },
                Type::Set {
                    identifiers: rhs, ..
                },
            ) => {
                if is_strict {
                    rhs.iter().all(|x| lhs.contains(x))
                } else {
                    rhs.iter().any(|x| lhs.contains(x))
                }
            }
            (Type::TypeReference { .. }, rhs) => {
                self.is_assignable_type(lhs.resolve(self)?, rhs, is_strict)?
            }
            (lhs, Type::TypeReference { .. }) => {
                self.is_assignable_type(lhs, rhs.resolve(self)?, is_strict)?
            }
            _ => false,
        })
    }

    pub fn is_equal_type(
        &self,
        lhs: &Type<Id>,
        rhs: &Type<Id>,
        is_strict: bool,
    ) -> Result<bool, Error<Id>> {
        Ok(self.is_assignable_type(lhs, rhs, is_strict)?
            && self.is_assignable_type(rhs, lhs, is_strict)?)
    }

    pub fn is_symbol(&self, id: &Id) -> bool {
        !(self.resolve_constant(id).is_some() || self.resolve_variable(id).is_some())
    }

    pub fn resolve_constant(&self, identifier: &Id) -> Option<&Constant<Id>> {
        self.constants.iter().find(|x| &x.identifier == identifier)
    }

    pub fn resolve_constant_or_fail(&self, identifier: &Id) -> Result<&Constant<Id>, Error<Id>> {
        let Some(constant) = self.resolve_constant(identifier) else {
            return self.make_error(ErrorReason::UnresolvedConstant {
                identifier: identifier.clone(),
            });
        };
        Ok(constant)
    }

    pub fn rename_node(&mut self, node: &Node<Id>, new_node: &Node<Id>) {
        for edge in &mut self.edges {
            if &edge.lhs == node {
                Arc::make_mut(edge).lhs = new_node.clone();
            }
            if &edge.rhs == node {
                Arc::make_mut(edge).rhs = new_node.clone();
            }
            if edge.label.is_reachability() {
                if let Label::Reachability { lhs, rhs, .. } = &mut Arc::make_mut(edge).label {
                    if lhs == node {
                        *lhs = new_node.clone();
                    }
                    if rhs == node {
                        *rhs = new_node.clone();
                    }
                }
            }
        }

        for pragma in &mut self.pragmas {
            pragma.rename_node(node, new_node);
        }
    }

    pub fn resolve_type_or_fail<'a>(
        &'a self,
        type_: &'a Type<Id>,
    ) -> Result<Option<&'a Typedef<Id>>, Error<Id>> {
        self.typedefs
            .iter()
            .find_map(
                |typedef| match self.is_assignable_type(&typedef.type_, type_, true) {
                    Ok(true) => match self.is_assignable_type(type_, &typedef.type_, true) {
                        Ok(true) => Some(Ok(typedef)),
                        Ok(_) => None,
                        Err(error) => Some(Err(error)),
                    },
                    Ok(_) => None,
                    Err(error) => Some(Err(error)),
                },
            )
            .transpose()
    }

    pub fn resolve_typedef(&self, identifier: &Id) -> Option<&Typedef<Id>> {
        self.typedefs.iter().find(|x| &x.identifier == identifier)
    }

    pub fn resolve_typedef_or_fail(&self, identifier: &Id) -> Result<&Typedef<Id>, Error<Id>> {
        let Some(typedef) = self.resolve_typedef(identifier) else {
            return self.make_error(ErrorReason::UnresolvedType {
                identifier: identifier.clone(),
            });
        };
        Ok(typedef)
    }

    pub fn resolve_variable(&self, identifier: &Id) -> Option<&Variable<Id>> {
        self.variables.iter().find(|x| &x.identifier == identifier)
    }

    pub fn resolve_variable_or_fail(&self, identifier: &Id) -> Result<&Variable<Id>, Error<Id>> {
        let Some(variable) = self.resolve_variable(identifier) else {
            return self.make_error(ErrorReason::UnresolvedVariable {
                identifier: identifier.clone(),
            });
        };
        Ok(variable)
    }
}

#[derive(Eq, PartialEq)]
pub enum ReachabilityCheckResult {
    Loop,
    Reachable,
    Unreachable,
}

impl ReachabilityCheckResult {
    pub fn is_reachable(&self) -> bool {
        *self == Self::Reachable
    }
}

impl<Id: Ord> Game<Id> {
    /// It works only if `self.edges` are sorted by `cmp_outgoing`.
    pub fn add_edge_sorted(&mut self, edge: Arc<Edge<Id>>) {
        if let Err(index) = self.edges.binary_search_by(|x| x.cmp_outgoing(&edge)) {
            self.edges.insert(index, edge);
        }
    }

    pub fn add_pragma(&mut self, pragma: Pragma<Id>) {
        self.pragmas.sort_unstable();
        if let Err(index) = self.pragmas.binary_search(&pragma) {
            self.pragmas.insert(index, pragma);
        }
    }

    pub fn make_check_reachability(
        &self,
        detect_loops: bool,
    ) -> impl Fn(&Node<Id>, &Node<Id>) -> ReachabilityCheckResult + '_ {
        let next_edges = self.next_edges();
        move |a: &Node<_>, b: &Node<_>| -> ReachabilityCheckResult {
            let mut seen = BTreeSet::new();
            let mut queue = vec![a];
            let mut result = ReachabilityCheckResult::Unreachable;

            if detect_loops {
                seen.insert(a);
            }

            while let Some(lhs) = queue.pop() {
                if let Some(edges) = next_edges.get(lhs) {
                    for edge in edges {
                        if detect_loops {
                            if let Label::Reachability { lhs, .. } = &edge.label {
                                if seen.contains(lhs) {
                                    return ReachabilityCheckResult::Loop;
                                }
                            }
                        }

                        if !seen.contains(&edge.rhs) {
                            if edge.rhs == *b {
                                result = ReachabilityCheckResult::Reachable;
                                if !detect_loops {
                                    return result;
                                }
                            }

                            seen.insert(&edge.rhs);
                            queue.push(&edge.rhs);
                        }
                    }
                }
            }

            result
        }
    }

    pub fn nodes(&self) -> BTreeSet<&Node<Id>> {
        self.edges
            .iter()
            .flat_map(|edge| [&edge.lhs, &edge.rhs])
            .collect()
    }

    pub fn next_edges(&self) -> BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>> {
        self.edges
            .iter()
            .fold(BTreeMap::new(), |mut next_edges, edge| {
                next_edges.entry(&edge.lhs).or_default().insert(edge);
                next_edges
            })
    }

    pub fn next_nodes(&self) -> BTreeMap<&Node<Id>, BTreeSet<&Node<Id>>> {
        self.edges
            .iter()
            .fold(BTreeMap::new(), |mut next_nodes, edge| {
                next_nodes.entry(&edge.lhs).or_default().insert(&edge.rhs);
                next_nodes
            })
    }

    pub fn next_edges_idx(&self) -> BTreeMap<&Node<Id>, BTreeSet<usize>> {
        self.edges
            .iter()
            .enumerate()
            .fold(BTreeMap::new(), |mut next_edges, (idx, edge)| {
                next_edges.entry(&edge.lhs).or_default().insert(idx);
                next_edges
            })
    }

    pub fn next_edges_with_idx(&self) -> BTreeMap<&Node<Id>, SetWithIdx<Arc<Edge<Id>>>> {
        self.edges
            .iter()
            .enumerate()
            .fold(BTreeMap::new(), |mut next_edges, (idx, edge)| {
                next_edges.entry(&edge.lhs).or_default().insert((idx, edge));
                next_edges
            })
    }

    pub fn prev_edges(&self) -> BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>> {
        self.edges
            .iter()
            .fold(BTreeMap::new(), |mut prev_edges, edge| {
                prev_edges.entry(&edge.rhs).or_default().insert(edge);
                prev_edges
            })
    }

    pub fn prev_edges_idx(&self) -> BTreeMap<&Node<Id>, BTreeSet<usize>> {
        self.edges
            .iter()
            .enumerate()
            .fold(BTreeMap::new(), |mut next_edges, (idx, edge)| {
                next_edges.entry(&edge.rhs).or_default().insert(idx);
                next_edges
            })
    }

    pub fn prev_edges_with_idx(&self) -> BTreeMap<&Node<Id>, SetWithIdx<Arc<Edge<Id>>>> {
        self.edges
            .iter()
            .enumerate()
            .fold(BTreeMap::new(), |mut next_edges, (idx, edge)| {
                next_edges.entry(&edge.rhs).or_default().insert((idx, edge));
                next_edges
            })
    }

    pub fn reachability_targets(&self) -> BTreeSet<&Node<Id>> {
        self.edges
            .iter()
            .filter_map(|edge| {
                if let Label::Reachability { lhs, rhs, .. } = &edge.label {
                    Some([lhs, rhs])
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    /// It works only if `self.edges` are sorted by `cmp_outgoing`.
    pub fn sorted_outgoing_edges<'a>(
        &'a self,
        node: &'a Node<Id>,
    ) -> impl Iterator<Item = &'a Arc<Edge<Id>>> {
        self.edges
            .binary_search_by(|x| x.lhs.cmp(node))
            .map_or_else(
                |_| self.edges[0..0].iter(),
                |index| {
                    let mut from = index;
                    while from > 0
                        && self
                            .edges
                            .get(from.saturating_sub(1))
                            .is_some_and(|edge| edge.lhs == *node)
                    {
                        from = from.saturating_sub(1);
                    }

                    let mut to = index;
                    while self
                        .edges
                        .get(to.saturating_add(1))
                        .is_some_and(|edge| edge.lhs == *node)
                    {
                        to = to.saturating_add(1);
                    }

                    self.edges[from..=to].iter()
                },
            )
    }
}

impl Game<Arc<str>> {
    pub fn to_stats(&self) -> Stats {
        let edges = self.edges.len();
        let nodes = self.nodes().len();
        let constants = self.constants.len();
        let variables = self.variables.len();
        let state_size = self
            .variables
            .iter()
            .map(|variable| variable.type_.memory_size(self).unwrap_or(0))
            .sum::<usize>();
        let typedefs = self.typedefs.len();
        let reachability_subautomatons = self
            .edges
            .iter()
            .filter_map(|e| {
                if let Label::Reachability { lhs, .. } = &e.label {
                    Some(lhs)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>()
            .len();
        let main_automaton_nodes = self
            .analyse::<ReachableNodes>(false)
            .values()
            .filter(|reachable| **reachable)
            .count();
        let branchings: Vec<_> = self.next_edges().values().map(BTreeSet::len).collect();
        let max_branching = branchings.iter().max().copied().unwrap_or(0);
        let average_branching = branchings.iter().sum::<usize>() as f64 / branchings.len() as f64;
        let repeat_nodes = self
            .pragmas
            .iter()
            .filter_map(|pragma| {
                if let Pragma::Repeat { nodes, .. } = pragma {
                    Some(nodes)
                } else {
                    None
                }
            })
            .flatten()
            .collect::<BTreeSet<_>>();
        let unique_nodes = self
            .pragmas
            .iter()
            .filter_map(|pragma| {
                if let Pragma::Unique { nodes, .. } = pragma {
                    Some(nodes)
                } else {
                    None
                }
            })
            .flatten()
            .collect::<BTreeSet<_>>();
        let repeat_or_unique_nodes = repeat_nodes.union(&unique_nodes).collect::<BTreeSet<_>>();

        Stats {
            edges,
            nodes,
            main_automaton_nodes,
            reachability_subautomatons,
            max_branching,
            average_branching,
            constants,
            variables,
            typedefs,
            state_size,
            repeat_nodes: repeat_nodes.len(),
            unique_nodes: unique_nodes.len(),
            repeat_or_unique_nodes: repeat_or_unique_nodes.len(),
        }
    }
}

impl<Id: PartialEq> Game<Id> {
    pub fn add_edge(&mut self, edge: Arc<Edge<Id>>) {
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }

    pub fn are_connected(&self, lhs: &Node<Id>, rhs: &Node<Id>) -> bool {
        self.edges
            .iter()
            .any(|edge| &edge.lhs == lhs && &edge.rhs == rhs)
    }

    pub fn is_reachability_target(&self, node: &Node<Id>) -> bool {
        self.edges.iter().any(|edge| matches!(&edge.label, Label::Reachability { lhs, rhs, .. } if lhs == node || rhs == node))
    }

    /// Returns the only edge ending at `node` or `None` if there are multiple or no such edges.
    pub fn incoming_edge<'a>(&'a self, node: &'a Node<Id>) -> Option<&'a Arc<Edge<Id>>> {
        let mut iterator = self.incoming_edges(node);
        iterator.next().filter(|_| iterator.next().is_none())
    }

    pub fn incoming_edges<'a>(
        &'a self,
        node: &'a Node<Id>,
    ) -> impl Iterator<Item = &'a Arc<Edge<Id>>> {
        self.edges.iter().filter(move |edge| &edge.rhs == node)
    }

    /// Returns the only edge starting from `node` or `None` if there are multiple or no such edges.
    pub fn outgoing_edge<'a>(&'a self, node: &'a Node<Id>) -> Option<&'a Arc<Edge<Id>>> {
        let mut iterator = self.outgoing_edges(node);
        iterator.next().filter(|_| iterator.next().is_none())
    }

    pub fn outgoing_edges<'a>(
        &'a self,
        node: &'a Node<Id>,
    ) -> impl Iterator<Item = &'a Arc<Edge<Id>>> {
        self.edges.iter().filter(move |edge| &edge.lhs == node)
    }

    pub fn remove_edge(&mut self, edge: &Edge<Id>) {
        self.edges.retain(|x| x.as_ref() != edge);
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Pragma<Id> {
    ArtificialTag {
        #[serde(skip)]
        span: Span,
        tags: Vec<Id>,
    },
    Disjoint {
        #[serde(skip)]
        span: Span,
        #[serde(rename = "edgeName")]
        node: Node<Id>,
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
    },
    DisjointExhaustive {
        #[serde(skip)]
        span: Span,
        #[serde(rename = "edgeName")]
        node: Node<Id>,
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
    },
    Repeat {
        #[serde(skip)]
        span: Span,
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
        #[serde(rename = "identifiers")]
        identifiers: Vec<Id>,
    },
    SimpleApply {
        #[serde(skip)]
        span: Span,
        lhs: Node<Id>,
        rhs: Node<Id>,
        tags: Vec<PragmaTag<Id>>,
        assignments: Vec<PragmaAssignment<Id>>,
    },
    SimpleApplyExhaustive {
        #[serde(skip)]
        span: Span,
        lhs: Node<Id>,
        rhs: Node<Id>,
        tags: Vec<PragmaTag<Id>>,
        assignments: Vec<PragmaAssignment<Id>>,
    },
    TagIndex {
        #[serde(skip)]
        span: Span,
        index: usize,
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
    },
    TagMaxIndex {
        #[serde(skip)]
        span: Span,
        index: usize,
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
    },
    TranslatedFromRbg {
        #[serde(skip)]
        span: Span,
    },
    Unique {
        #[serde(skip)]
        span: Span,
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
    },
}

impl<Id> Pragma<Id> {
    pub fn nodes(&self) -> Box<dyn Iterator<Item = &Node<Id>> + '_> {
        match self {
            Self::ArtificialTag { .. } => Box::new(None.into_iter()),
            Self::Disjoint { node, nodes, .. } | Self::DisjointExhaustive { node, nodes, .. } => {
                Box::new(Some(node).into_iter().chain(nodes))
            }
            Self::Repeat { nodes, .. }
            | Self::TagIndex { nodes, .. }
            | Self::TagMaxIndex { nodes, .. }
            | Self::Unique { nodes, .. } => Box::new(nodes.iter()),
            Self::SimpleApply { lhs, rhs, .. } | Self::SimpleApplyExhaustive { lhs, rhs, .. } => {
                Box::new(Some(lhs).into_iter().chain(Some(rhs)))
            }
            Self::TranslatedFromRbg { .. } => Box::new([].into_iter()),
        }
    }
}

impl<Id: Clone + PartialEq> Pragma<Id> {
    pub fn rename_node(&mut self, old_node: &Node<Id>, new_node: &Node<Id>) {
        match self {
            Self::ArtificialTag { .. } => {}
            Self::Disjoint { node, nodes, .. } | Self::DisjointExhaustive { node, nodes, .. } => {
                if node == old_node {
                    *node = new_node.clone();
                }
                for node in nodes {
                    if node == old_node {
                        *node = new_node.clone();
                    }
                }
            }
            Self::Repeat { nodes, .. }
            | Self::TagIndex { nodes, .. }
            | Self::TagMaxIndex { nodes, .. }
            | Self::Unique { nodes, .. } => {
                for node in nodes {
                    if node == old_node {
                        *node = new_node.clone();
                    }
                }
            }
            Self::SimpleApply { lhs, rhs, .. } | Self::SimpleApplyExhaustive { lhs, rhs, .. } => {
                if lhs == old_node {
                    *lhs = new_node.clone();
                }
                if rhs == old_node {
                    *rhs = new_node.clone();
                }
            }
            Self::TranslatedFromRbg { .. } => {}
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PragmaAssignment<Id> {
    pub lhs: Arc<Expression<Id>>,
    pub rhs: Arc<Expression<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PragmaTag<Id> {
    Symbol {
        symbol: Id,
    },
    Variable {
        identifier: Id,
        type_: Arc<Type<Id>>,
    },
}

impl<Id> PragmaTag<Id> {
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable { .. })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Stats {
    edges: usize,
    nodes: usize,
    main_automaton_nodes: usize,
    reachability_subautomatons: usize,
    max_branching: usize,
    average_branching: f64,
    constants: usize,
    variables: usize,
    typedefs: usize,
    state_size: usize,
    repeat_or_unique_nodes: usize,
    repeat_nodes: usize,
    unique_nodes: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Type<Id> {
    Arrow {
        lhs: Arc<Self>,
        rhs: Arc<Self>,
    },
    Set {
        #[serde(skip)]
        span: Span,
        identifiers: Vec<Id>,
    },
    TypeReference {
        identifier: Id,
    },
}

impl<Id> Type<Id> {
    pub fn as_singleton(&self) -> Option<&Id> {
        match self {
            Self::Set { identifiers, .. } if identifiers.len() == 1 => identifiers.first(),
            _ => None,
        }
    }

    pub fn as_type_reference(&self) -> Option<&Id> {
        match self {
            Self::TypeReference { identifier } => Some(identifier),
            _ => None,
        }
    }

    pub fn is_arrow(&self) -> bool {
        matches!(self, Self::Arrow { .. })
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Self::TypeReference { .. })
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set { .. })
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Arrow { .. } => todo!(),
            Self::Set { identifiers, .. } => identifiers.len(),
            Self::TypeReference { .. } => 1,
        }
    }

    pub fn new(identifier: Id) -> Self {
        Self::TypeReference { identifier }
    }
}

impl<Id: Clone + PartialEq> Type<Id> {
    /// Used to determine how much memory a variable of this type could take.
    fn memory_size(&self, game: &Game<Id>) -> Result<usize, Error<Id>> {
        match self {
            Self::Arrow { lhs, rhs } => {
                let lhs = lhs.resolve(game).and_then(|lhs| lhs.memory_size(game));
                let rhs = rhs.resolve(game).and_then(|rhs| rhs.memory_size(game));
                lhs.and_then(|lhs| rhs.map(|rhs| lhs * rhs))
            }
            Self::Set { identifiers, .. } => Ok(identifiers.len()),
            Self::TypeReference { identifier } => game
                .resolve_typedef_or_fail(identifier)
                .and_then(|type_| type_.type_.memory_size(game)),
        }
    }

    pub fn resolve<'a>(&'a self, game: &'a Game<Id>) -> Result<&'a Self, Error<Id>> {
        if let Self::TypeReference { identifier } = self {
            game.resolve_typedef_or_fail(identifier)?
                .type_
                .resolve(game)
        } else {
            Ok(self)
        }
    }

    pub fn values(&self, game: &Game<Id>) -> Result<Vec<Id>, Error<Id>> {
        match self {
            Self::Arrow { .. } => todo!(),
            Self::Set { identifiers, .. } => Ok(identifiers.clone()),
            Self::TypeReference { identifier } => {
                game.resolve_typedef_or_fail(identifier)?.type_.values(game)
            }
        }
    }
}

impl<Id: Ord> Type<Id> {
    pub fn type_references(&self) -> Option<BTreeSet<&Id>> {
        let mut type_references = None;
        self.type_references_mut(&mut type_references);
        type_references
    }

    fn type_references_mut<'a>(&'a self, type_references: &mut Option<BTreeSet<&'a Id>>) {
        match self {
            Self::Arrow { lhs, rhs } => {
                lhs.type_references_mut(type_references);
                rhs.type_references_mut(type_references);
            }
            Self::Set { .. } => {}
            Self::TypeReference { identifier } => {
                type_references.get_or_insert_default().insert(identifier);
            }
        }
    }
}

impl<Id: PartialEq> Type<Id> {
    pub fn contains(&self, identifier: &Id) -> bool {
        matches!(self, Self::Set { identifiers, .. } if identifiers.contains(identifier))
    }

    pub fn is_identifier(&self, identifier: &Id) -> bool {
        matches!(self, Self::TypeReference { identifier: x } if x == identifier)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "TypeDeclaration", tag = "kind")]
pub struct Typedef<Id> {
    #[serde(skip)]
    pub span: Span,
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Arc<Type<Id>>,
}

impl<Id> Typedef<Id> {
    pub fn new(span: Span, identifier: Id, type_: Arc<Type<Id>>) -> Self {
        Self {
            span,
            identifier,
            type_,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Value<Id> {
    Element {
        identifier: Id,
    },
    Map {
        #[serde(skip)]
        span: Span,
        entries: Vec<ValueEntry<Id>>,
    },
}

impl<Id> Value<Id> {
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map { .. })
    }

    pub fn new(identifier: Id) -> Self {
        Self::Element { identifier }
    }

    pub fn new_empty(default_value: Arc<Self>) -> Self {
        Self::Map {
            span: Span::none(),
            entries: vec![ValueEntry::new_default(default_value)],
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Element { .. } => 1,
            Self::Map { entries, .. } => entries
                .iter()
                .map(|value_entry| value_entry.value.size())
                .sum::<usize>()
                .saturating_add(1),
        }
    }

    pub fn to_identifier(&self) -> Option<&Id> {
        match self {
            Self::Element { identifier } => Some(identifier),
            Self::Map { .. } => None,
        }
    }
}

impl<Id: Clone> Value<Id> {
    pub fn as_identifier(self) -> Option<Id> {
        match self {
            Self::Element { identifier } => Some(identifier),
            Self::Map { .. } => None,
        }
    }
}

impl<Id: PartialEq> Value<Id> {
    pub fn get_entry(&self, identifier: &Id) -> Option<&Self> {
        match self {
            Self::Element { .. } => None,
            Self::Map { entries, .. } => entries
                .iter()
                .find(|entry| entry.identifier.as_ref() == Some(identifier))
                .or_else(|| entries.iter().find(|entry| entry.identifier.is_none()))
                .map(|entry| entry.value.as_ref()),
        }
    }
}

impl<Id: Ord> Value<Id> {
    pub fn from_pairs(pairs: Vec<(Id, Arc<Self>)>) -> Self {
        let most_common_value = pairs
            .iter()
            .fold(BTreeMap::<_, usize>::new(), |mut counts, (_, value)| {
                *counts.entry(value.clone()).or_default() += 1;
                counts
            })
            .into_iter()
            .max_by(|(v1, c1), (v2, c2)| {
                // Count, size, key.
                c1.cmp(c2)
                    .then_with(|| v1.size().cmp(&v2.size()).reverse())
                    .then_with(|| v1.cmp(v2))
            })
            .expect("Value::from_pairs require at least one pair.")
            .0;

        Self::Map {
            span: Span::none(),
            entries: Some(ValueEntry::new_default(most_common_value.clone()))
                .into_iter()
                .chain(
                    pairs
                        .into_iter()
                        .filter(|(_, value)| *value != most_common_value)
                        .map(|(key, value)| ValueEntry::new(Span::none(), Some(key), value)),
                )
                .collect(),
        }
    }

    pub fn from_pairs_iter(iter: impl Iterator<Item = (Id, Arc<Self>)>) -> Self {
        Self::from_pairs(iter.collect())
    }

    /// Test:
    /// ```
    /// use rg::ast::{Value, ValueEntry};
    /// use std::collections::BTreeSet;
    /// let value = Value::Map { span: Default::default(), entries: vec![
    ///     ValueEntry::new(Default::default(), Some("a"), Value::Element { identifier: "b" }.into()),
    ///     ValueEntry::new(Default::default(), None, Value::Element { identifier: "d" }.into()),
    /// ]};
    /// let mut vars = BTreeSet::new();
    /// vars.insert(&"a");
    /// vars.insert(&"b");
    /// vars.insert(&"d");
    /// assert_eq!(value.identifiers(), vars);
    /// ```
    pub fn identifiers(&self) -> BTreeSet<&Id> {
        let mut vars = BTreeSet::new();
        match self {
            Self::Element { identifier } => {
                vars.insert(identifier);
            }
            Self::Map { entries, .. } => {
                for entry in entries {
                    entry.identifier.as_ref().map(|x| vars.insert(x));
                    vars.extend(entry.value.identifiers());
                }
            }
        }
        vars
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct ValueEntry<Id> {
    #[serde(skip)]
    pub span: Span,
    pub identifier: Option<Id>,
    pub value: Arc<Value<Id>>,
}

impl<Id> ValueEntry<Id> {
    pub fn new(span: Span, identifier: Option<Id>, value: Arc<Value<Id>>) -> Self {
        Self {
            span,
            identifier,
            value,
        }
    }

    pub fn new_default(value: Arc<Value<Id>>) -> Self {
        Self {
            span: Span::none(),
            identifier: None,
            value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "VariableDeclaration", tag = "kind")]
pub struct Variable<Id> {
    #[serde(skip)]
    pub span: Span,
    #[serde(rename = "defaultValue")]
    pub default_value: Arc<Value<Id>>,
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Arc<Type<Id>>,
}

impl<Id> Variable<Id> {
    pub fn new(
        span: Span,
        identifier: Id,
        type_: Arc<Type<Id>>,
        default_value: Arc<Value<Id>>,
    ) -> Self {
        Self {
            span,
            default_value,
            identifier,
            type_,
        }
    }
}
