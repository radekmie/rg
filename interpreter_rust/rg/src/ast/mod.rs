mod display;
mod transforms;
mod validators;

use map_id::MapId;
use map_id_macro::MapId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::sync::Arc;
use utils::position::Span;

pub type Binding<'a, Id> = (&'a Id, &'a Arc<Type<Id>>);
pub type Mapping<Id> = BTreeMap<Id, Id>;

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
    pub fn new(span: Span, lhs: Node<Id>, rhs: Node<Id>, label: Label<Id>) -> Self {
        Self {
            span,
            label,
            lhs,
            rhs,
        }
    }

    pub fn has_bindings(&self) -> bool {
        self.lhs.has_bindings() || self.rhs.has_bindings()
    }
}

impl<Id: Clone + Ord> Edge<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        Self {
            span: self.span,
            label: self.label.rename_variables(mapping),
            lhs: self.lhs.rename_variables(mapping),
            rhs: self.rhs.rename_variables(mapping),
        }
    }
}

impl<Id: Ord> Edge<Id> {
    pub fn bindings(&self) -> BTreeSet<Binding<Id>> {
        self.lhs.bindings().chain(self.rhs.bindings()).collect()
    }
}

impl<Id: PartialEq> Edge<Id> {
    pub fn get_binding(&self, identifier: &Id) -> Option<Binding<Id>> {
        self.lhs
            .bindings()
            .chain(self.rhs.bindings())
            .find(|binding| binding.0 == identifier)
    }

    pub fn has_binding(&self, identifier: &Id) -> bool {
        self.get_binding(identifier).is_some()
    }
}

impl Edge<Arc<str>> {
    pub fn substitute_bindings(&self, mapping: &Mapping<Arc<str>>) -> Self {
        Self {
            span: self.span,
            label: self.label.rename_variables(mapping),
            lhs: self.lhs.substitute_bindings(mapping),
            rhs: self.rhs.substitute_bindings(mapping),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "EdgeLabel", tag = "kind")]
pub enum Label<Id> {
    Assignment {
        lhs: Arc<Expression<Id>>,
        rhs: Arc<Expression<Id>>,
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
}

impl<Id> Default for Label<Id> {
    fn default() -> Self {
        Self::Skip { span: Span::none() }
    }
}

impl<Id> Label<Id> {
    pub fn is_assignment(&self) -> bool {
        matches!(self, Self::Assignment { .. })
    }

    pub fn is_tag(&self) -> bool {
        matches!(self, Self::Tag { .. })
    }

    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip { .. })
    }
}

impl<Id: Clone + Ord> Label<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        match self {
            Self::Assignment { lhs, rhs } => Self::Assignment {
                lhs: Arc::new(lhs.rename_variables(mapping)),
                rhs: Arc::new(rhs.rename_variables(mapping)),
            },
            Self::Comparison { lhs, rhs, negated } => Self::Comparison {
                lhs: Arc::new(lhs.rename_variables(mapping)),
                rhs: Arc::new(rhs.rename_variables(mapping)),
                negated: *negated,
            },
            Self::Tag { symbol } => Self::Tag {
                symbol: mapping.get(symbol).unwrap_or(symbol).clone(),
            },
            _ => self.clone(),
        }
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
}

impl<Id: PartialEq> Label<Id> {
    pub fn has_variable(&self, identifier: &Id) -> bool {
        matches!(self, Self::Assignment { lhs, rhs } | Self::Comparison { lhs, rhs, .. } if lhs.has_variable(identifier) || rhs.has_variable(identifier))
    }

    pub fn is_self_assignment(&self) -> bool {
        matches!(self, Self::Assignment { lhs, rhs } if lhs.is_equal_reference(rhs))
    }

    pub fn is_self_comparison(&self) -> bool {
        matches!(self, Self::Comparison { lhs, rhs, negated } if !negated && lhs.is_equal_reference(rhs))
    }
}

impl Label<Arc<str>> {
    pub fn is_player_assignment(&self) -> bool {
        matches!(self, Self::Assignment { lhs, .. } if lhs.uncast().is_player_reference())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "EdgeName", tag = "kind")]
pub struct Node<Id> {
    #[serde(skip)]
    pub span: Span,
    pub parts: Vec<NodePart<Id>>,
}

impl<Id> Node<Id> {
    pub fn new(identifier: Id) -> Self {
        Self {
            span: Span::none(),
            parts: vec![NodePart::new(identifier)],
        }
    }

    pub fn bindings(&self) -> impl Iterator<Item = Binding<Id>> {
        self.parts
            .iter()
            .filter_map(|node_part| node_part.binding())
    }

    pub fn has_bindings(&self) -> bool {
        self.bindings().next().is_some()
    }
}

impl<Id: PartialEq> Node<Id> {
    pub fn has_binding(&self, identifier: &Id) -> bool {
        self.bindings().any(|binding| binding.0 == identifier)
    }

    pub fn has_equal_bindings(&self, other: &Self) -> bool {
        self.bindings().eq(other.bindings())
    }
}

impl<Id: Clone + Ord> Node<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        Self {
            span: self.span,
            parts: self
                .parts
                .iter()
                .map(|node_part| node_part.rename_variables(mapping))
                .collect(),
        }
    }
}

impl Node<Arc<str>> {
    pub fn is_begin(&self) -> bool {
        matches!(&self.parts[..], [NodePart::Literal { identifier }] if &**identifier == "begin")
    }

    pub fn substitute_bindings(&self, mapping: &Mapping<Arc<str>>) -> Self {
        let identifier = self
            .parts
            .iter()
            .map(|node_part| match node_part {
                NodePart::Binding { identifier, .. } => mapping.get(identifier).unwrap(),
                NodePart::Literal { identifier } => identifier,
            })
            .cloned()
            .collect::<Vec<_>>()
            .join("__bind__");
        Self {
            span: self.span,
            parts: vec![NodePart::new(Arc::from(identifier))],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "EdgeNamePart", tag = "kind")]
pub enum NodePart<Id> {
    Binding {
        #[serde(skip)]
        span: Span,
        identifier: Id,
        #[serde(rename = "type")]
        type_: Arc<Type<Id>>,
    },
    Literal {
        identifier: Id,
    },
}

impl<Id> NodePart<Id> {
    pub fn binding(&self) -> Option<Binding<Id>> {
        match self {
            Self::Binding {
                identifier, type_, ..
            } => Some((identifier, type_)),
            Self::Literal { .. } => None,
        }
    }

    pub fn identifier(&self) -> &Id {
        match self {
            Self::Binding { identifier, .. } => identifier,
            Self::Literal { identifier } => identifier,
        }
    }

    pub fn new(identifier: Id) -> Self {
        Self::Literal { identifier }
    }

    pub fn type_(&self) -> Option<Arc<Type<Id>>> {
        match self {
            EdgeNamePart::Binding { type_, .. } => Some(type_.clone()),
            EdgeNamePart::Literal { .. } => None,
        }
    }
}

impl<Id: Clone + Ord> NodePart<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        if let Self::Binding {
            identifier, type_, ..
        } = self
        {
            if let Some(identifier) = mapping.get(identifier) {
                return Self::Binding {
                    span: Span::none(),
                    identifier: identifier.clone(),
                    type_: type_.clone(),
                };
            }
        }

        self.clone()
    }
}

#[derive(Debug)]
pub struct Error<Id> {
    pub game: Game<Id>,
    pub reason: ErrorReason<Id>,
}

// Simplify error handling in WASM modules.
impl<Id: Display> From<Error<Id>> for String {
    fn from(error: Error<Id>) -> Self {
        format!("{error}")
    }
}

#[derive(Debug)]
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
    DuplicatedMapKey {
        key: Option<Id>,
        value: Value<Id>,
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
    pub fn new(identifier: Id) -> Self {
        Self::Reference { identifier }
    }

    pub fn is_cast_and(&self, fn_: impl FnOnce(&Arc<Type<Id>>, &Arc<Self>) -> bool) -> bool {
        matches!(self, Self::Cast { lhs, rhs, .. } if fn_(lhs, rhs))
    }

    pub fn uncast(&self) -> &Self {
        match self {
            Self::Cast { rhs, .. } => rhs.uncast(),
            _ => self,
        }
    }
}

impl<Id: Clone + PartialEq> Expression<Id> {
    pub fn infer(
        &self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
    ) -> Result<Arc<Type<Id>>, Error<Id>> {
        match self {
            Self::Access { lhs, rhs, .. } => {
                let accessed_type = lhs.infer(game, edge)?;
                let Type::Arrow {
                    lhs: key_type,
                    rhs: value_type,
                } = accessed_type.resolve(game)?
                else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: accessed_type });
                };

                let accessor_type = rhs.infer(game, edge)?;
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
                let rhs = rhs.infer(game, edge)?;
                if !game.is_assignable_type(lhs, &rhs, false)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch {
                        lhs: lhs.clone(),
                        rhs,
                    });
                }

                Ok(lhs.clone())
            }
            Self::Reference { identifier } => Ok(game.infer(identifier, edge)),
        }
    }

    pub fn remove_casts(&self, identifier: &Id) -> Self {
        match self {
            Self::Access { lhs, rhs, .. } => Self::Access {
                span: Span::none(),
                lhs: Arc::new(lhs.remove_casts(identifier)),
                rhs: Arc::new(rhs.remove_casts(identifier)),
            },
            Self::Cast { lhs, rhs, .. } if lhs.is_reference(identifier) => {
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
}

impl<Id: Clone + Ord> Expression<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        match self {
            Self::Access { lhs, rhs, .. } => Self::Access {
                span: Span::none(),
                lhs: Arc::new(lhs.rename_variables(mapping)),
                rhs: Arc::new(rhs.rename_variables(mapping)),
            },
            Self::Cast { lhs, rhs, .. } => Self::Cast {
                span: Span::none(),
                lhs: lhs.clone(),
                rhs: Arc::new(rhs.rename_variables(mapping)),
            },
            Self::Reference { identifier } => Self::Reference {
                identifier: mapping.get(identifier).unwrap_or(identifier).clone(),
            },
        }
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
    pub edges: Vec<Edge<Id>>,
    pub pragmas: Vec<Pragma<Id>>,
    #[serde(rename = "types")]
    pub typedefs: Vec<Typedef<Id>>,
    pub variables: Vec<Variable<Id>>,
}

impl<Id: Clone> Game<Id> {
    pub fn make_error<T>(&self, reason: ErrorReason<Id>) -> Result<T, Error<Id>> {
        Err(Error {
            game: self.clone(),
            reason,
        })
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
                        mapping.insert(identifier.clone(), value.clone());
                        mapping
                    })
                })
                .collect();
        }

        Ok(mappings)
    }
}

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn infer(&self, identifier: &Id, edge: Option<&Edge<Id>>) -> Arc<Type<Id>> {
        self.infer_or_none(identifier, edge)
            .cloned()
            .unwrap_or_else(|| {
                Arc::new(Type::Set {
                    span: Span::none(),
                    identifiers: vec![identifier.clone()],
                })
            })
    }

    pub fn infer_or_none<'a>(
        &'a self,
        identifier: &Id,
        edge: Option<&'a Edge<Id>>,
    ) -> Option<&'a Arc<Type<Id>>> {
        edge.and_then(|edge| edge.get_binding(identifier))
            .map(|(_, type_)| type_)
            .or_else(|| self.resolve_constant(identifier).map(|x| &x.type_))
            .or_else(|| self.resolve_variable(identifier).map(|x| &x.type_))
    }

    pub fn is_assignable_identifier(
        &self,
        lhs: &Type<Id>,
        rhs: &Id,
        is_strict: bool,
    ) -> Result<bool, Error<Id>> {
        if let Some(rhs) = self.infer_or_none(rhs, None) {
            self.is_assignable_type(lhs, rhs, is_strict)
        } else if let Type::Set { identifiers, .. } = lhs.resolve(self)? {
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

impl<Id: Ord> Game<Id> {
    pub fn nodes(&self) -> BTreeSet<&Node<Id>> {
        self.edges
            .iter()
            .flat_map(|edge| [&edge.lhs, &edge.rhs])
            .collect()
    }

    pub fn next_edges(&self) -> BTreeMap<&Node<Id>, BTreeSet<&Edge<Id>>> {
        self.edges
            .iter()
            .fold(BTreeMap::new(), |mut next_edges, edge| {
                next_edges.entry(&edge.lhs).or_default().insert(edge);
                next_edges
            })
    }

    pub fn prev_edges(&self) -> BTreeMap<&Node<Id>, BTreeSet<&Edge<Id>>> {
        self.edges
            .iter()
            .fold(BTreeMap::new(), |mut prev_edges, edge| {
                prev_edges.entry(&edge.rhs).or_default().insert(edge);
                prev_edges
            })
    }
}

impl<Id: PartialEq> Game<Id> {
    pub fn add_edge(&mut self, edge: Edge<Id>) {
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
    pub fn incoming_edge<'a>(&'a self, node: &'a Node<Id>) -> Option<&'a Edge<Id>> {
        let mut iterator = self.incoming_edges(node);
        iterator.next().filter(|_| iterator.next().is_none())
    }

    pub fn incoming_edges<'a>(&'a self, node: &'a Node<Id>) -> impl Iterator<Item = &'a Edge<Id>> {
        self.edges.iter().filter(move |edge| &edge.rhs == node)
    }

    /// Returns the only edge starting from `node` or `None` if there are multiple or no such edges.
    pub fn outgoing_edge<'a>(&'a self, node: &'a Node<Id>) -> Option<&'a Edge<Id>> {
        let mut iterator = self.outgoing_edges(node);
        iterator.next().filter(|_| iterator.next().is_none())
    }

    pub fn outgoing_edges<'a>(&'a self, node: &'a Node<Id>) -> impl Iterator<Item = &'a Edge<Id>> {
        self.edges.iter().filter(move |edge| &edge.lhs == node)
    }

    pub fn remove_edge(&mut self, edge: &Edge<Id>) {
        self.edges.retain(|x| x != edge);
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Pragma<Id> {
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
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
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
    Unique {
        #[serde(skip)]
        span: Span,
        #[serde(rename = "edgeNames")]
        nodes: Vec<Node<Id>>,
    },
}

impl<Id> Pragma<Id> {
    pub fn nodes(&self) -> impl Iterator<Item = &Node<Id>> {
        match self {
            Self::Disjoint { node, nodes, .. } | Self::DisjointExhaustive { node, nodes, .. } => {
                Some(node).into_iter().chain(nodes)
            }
            Self::Repeat { nodes, .. }
            | Self::SimpleApply { nodes, .. }
            | Self::TagIndex { nodes, .. }
            | Self::TagMaxIndex { nodes, .. }
            | Self::Unique { nodes, .. } => None.into_iter().chain(nodes),
        }
    }
}

impl<Id: Ord> Pragma<Id> {
    pub fn bindings(&self) -> BTreeSet<Binding<Id>> {
        self.nodes().flat_map(Node::bindings).collect()
    }
}

impl Pragma<Arc<str>> {
    pub fn substitute_bindings_mut(&mut self, mappings: &[Mapping<Arc<str>>]) {
        if let Self::Disjoint { node, .. } | Self::DisjointExhaustive { node, .. } = self {
            let mut node_variants = mappings
                .iter()
                .map(|mapping| node.substitute_bindings(mapping))
                .collect::<BTreeSet<_>>();
            assert_eq!(
                node_variants.len(),
                1,
                "Cannot `substitute_bindings_mut` of a `Pragma` with bindings in `node`."
            );
            *node = node_variants.pop_first().unwrap();
        };

        match self {
            Self::Disjoint { nodes, .. }
            | Self::DisjointExhaustive { nodes, .. }
            | Self::Repeat { nodes, .. }
            | Self::SimpleApply { nodes, .. }
            | Self::TagIndex { nodes, .. }
            | Self::TagMaxIndex { nodes, .. }
            | Self::Unique { nodes, .. } => {
                *nodes = mappings
                    .iter()
                    .flat_map(|mapping| nodes.iter().map(|node| node.substitute_bindings(mapping)))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect();
            }
        }
    }
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

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set { .. })
    }

    pub fn new(identifier: Id) -> Self {
        Self::TypeReference { identifier }
    }
}

impl<Id: Clone + PartialEq> Type<Id> {
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

impl<Id: PartialEq> Type<Id> {
    pub fn is_reference(&self, identifier: &Id) -> bool {
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
    pub fn new(identifier: Id) -> Self {
        Self::Element { identifier }
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
