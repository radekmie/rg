use map_id::MapId;
use map_id_macro::MapId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::rc::Rc;

pub type Mapping<Id> = BTreeMap<Id, Id>;
pub type Binding<'a, Id> = (&'a Id, &'a Rc<Type<Id>>);

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "ConstantDeclaration", tag = "kind")]
pub struct Constant<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
    pub value: Rc<Value<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "EdgeDeclaration", tag = "kind")]
pub struct Edge<Id> {
    pub label: EdgeLabel<Id>,
    pub lhs: EdgeName<Id>,
    pub rhs: EdgeName<Id>,
}

impl<Id> Edge<Id> {
    pub fn has_bindings(&self) -> bool {
        self.lhs.has_bindings() || self.rhs.has_bindings()
    }
}

impl<Id: Clone + Ord> Edge<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        Self {
            label: self.label.rename_variables(mapping),
            lhs: self.lhs.rename_variables(mapping),
            rhs: self.rhs.rename_variables(mapping),
        }
    }
}

impl<Id: PartialEq> Edge<Id> {
    pub fn bindings(&self) -> Vec<Binding<Id>> {
        self.lhs.bindings().chain(self.rhs.bindings()).fold(
            Vec::default(),
            |mut bindings, binding| {
                if !bindings.contains(&binding) {
                    bindings.push(binding);
                }

                bindings
            },
        )
    }

    pub fn get_binding(&self, identifier: &Id) -> Option<Binding<Id>> {
        self.lhs
            .bindings()
            .find(|binding| binding.0 == identifier)
            .or_else(|| self.rhs.bindings().find(|binding| binding.0 == identifier))
    }
}

impl Edge<Rc<str>> {
    pub fn substitute_bindings(&self, mapping: &Mapping<Rc<str>>) -> Self {
        Self {
            label: self.label.rename_variables(mapping),
            lhs: self.lhs.substitute_bindings(mapping),
            rhs: self.rhs.substitute_bindings(mapping),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum EdgeLabel<Id> {
    Any {
        lhs: EdgeName<Id>,
        rhs: EdgeName<Id>,
    },
    Assignment {
        lhs: Rc<Expression<Id>>,
        rhs: Rc<Expression<Id>>,
    },
    Comparison {
        lhs: Rc<Expression<Id>>,
        rhs: Rc<Expression<Id>>,
        negated: bool,
    },
    Reachability {
        lhs: EdgeName<Id>,
        rhs: EdgeName<Id>,
        negated: bool,
    },
    Skip,
}

impl<Id> EdgeLabel<Id> {
    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip)
    }
}

impl<Id: Clone + Ord> EdgeLabel<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        match self {
            Self::Assignment { lhs, rhs } => Self::Assignment {
                lhs: Rc::new(lhs.rename_variables(mapping)),
                rhs: Rc::new(rhs.rename_variables(mapping)),
            },
            Self::Comparison { lhs, rhs, negated } => Self::Comparison {
                lhs: Rc::new(lhs.rename_variables(mapping)),
                rhs: Rc::new(rhs.rename_variables(mapping)),
                negated: *negated,
            },
            _ => self.clone(),
        }
    }
}

impl<Id: PartialEq> EdgeLabel<Id> {
    pub fn is_self_assignment(&self) -> bool {
        matches!(self, EdgeLabel::Assignment { lhs, rhs } if lhs.is_equal_reference(rhs))
    }
}

impl EdgeLabel<Rc<str>> {
    pub fn is_player_assignment(&self) -> bool {
        matches!(self, Self::Assignment { lhs, .. } if matches!(&**lhs, Expression::Reference { identifier } if &**identifier == "Player"))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct EdgeName<Id> {
    pub parts: Vec<EdgeNamePart<Id>>,
}

impl<Id> EdgeName<Id> {
    pub fn bindings(&self) -> impl Iterator<Item = Binding<Id>> {
        self.parts
            .iter()
            .flat_map(|edge_name_part| edge_name_part.binding())
    }

    pub fn has_bindings(&self) -> bool {
        self.bindings().next().is_some()
    }
}

impl<Id: Clone + Ord> EdgeName<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        Self {
            parts: self
                .parts
                .iter()
                .map(|edge_name| edge_name.rename_variables(mapping))
                .collect(),
        }
    }
}

impl EdgeName<Rc<str>> {
    pub fn is_begin(&self) -> bool {
        matches!(&self.parts[..], [EdgeNamePart::Literal { identifier }] if &**identifier == "begin")
    }

    pub fn substitute_bindings(&self, mapping: &Mapping<Rc<str>>) -> Self {
        let identifier = self
            .parts
            .iter()
            .map(|edge_name_part| match edge_name_part {
                EdgeNamePart::Binding { identifier, .. } => mapping.get(identifier).unwrap(),
                EdgeNamePart::Literal { identifier } => identifier,
            })
            .cloned()
            .collect::<Vec<_>>()
            .join("__bind__");
        Self {
            parts: vec![EdgeNamePart::Literal {
                identifier: Rc::from(identifier),
            }],
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum EdgeNamePart<Id> {
    Binding {
        identifier: Id,
        #[serde(rename = "type")]
        type_: Rc<Type<Id>>,
    },
    Literal {
        identifier: Id,
    },
}

impl<Id> EdgeNamePart<Id> {
    pub fn binding(&self) -> Option<Binding<Id>> {
        match self {
            Self::Binding { identifier, type_ } => Some((identifier, type_)),
            _ => None,
        }
    }
}

impl<Id: Clone + Ord> EdgeNamePart<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        if let Self::Binding { identifier, type_ } = self {
            if let Some(identifier) = mapping.get(identifier) {
                return Self::Binding {
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

#[derive(Debug)]
pub enum ErrorReason<Id> {
    ArrowTypeExpected {
        got: Rc<Type<Id>>,
    },
    AssignmentTypeMismatch {
        lhs: Rc<Type<Id>>,
        rhs: Rc<Type<Id>>,
    },
    ComparisonTypeMismatch {
        lhs: Rc<Type<Id>>,
        rhs: Rc<Type<Id>>,
    },
    EmptySetType {
        identifier: Id,
    },
    SetTypeExpected {
        got: Rc<Type<Id>>,
    },
    TypeDeclarationMismatch {
        expected: Rc<Type<Id>>,
        identifier: Id,
        resolved: Rc<Type<Id>>,
    },
    Unreachable {
        lhs: EdgeName<Id>,
        rhs: EdgeName<Id>,
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
        expected: Rc<Type<Id>>,
        identifier: Id,
        resolved: Rc<Type<Id>>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Expression<Id> {
    Access { lhs: Rc<Self>, rhs: Rc<Self> },
    Cast { lhs: Rc<Type<Id>>, rhs: Rc<Self> },
    Reference { identifier: Id },
}

impl<Id: Clone + PartialEq> Expression<Id> {
    pub fn infer(
        &self,
        game: &Game<Id>,
        edge: Option<&Edge<Id>>,
    ) -> Result<Rc<Type<Id>>, Error<Id>> {
        match self {
            Self::Access { lhs, rhs } => {
                let accessed_type = lhs.infer(game, edge)?;
                if let Type::Arrow {
                    lhs: key_type,
                    rhs: value_type,
                } = accessed_type.resolve(game)?
                {
                    let accessor_type = rhs.infer(game, edge)?;
                    if !accessor_type.resolve(game)?.is_set() {
                        return game
                            .make_error(ErrorReason::SetTypeExpected { got: accessor_type });
                    }

                    if !game.is_assignable_type(key_type, &accessor_type, false)? {
                        return game.make_error(ErrorReason::AssignmentTypeMismatch {
                            lhs: key_type.clone(),
                            rhs: accessor_type,
                        });
                    }

                    Ok(value_type.clone())
                } else {
                    game.make_error(ErrorReason::ArrowTypeExpected { got: accessed_type })
                }
            }
            Self::Cast { lhs, rhs } => {
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
}

impl<Id: Clone + Ord> Expression<Id> {
    pub fn rename_variables(&self, mapping: &Mapping<Id>) -> Self {
        match self {
            Self::Access { lhs, rhs } => Self::Access {
                lhs: Rc::new(lhs.rename_variables(mapping)),
                rhs: Rc::new(rhs.rename_variables(mapping)),
            },
            Self::Cast { lhs, rhs } => Self::Cast {
                lhs: lhs.clone(),
                rhs: Rc::new(rhs.rename_variables(mapping)),
            },
            Self::Reference { identifier } => Self::Reference {
                identifier: mapping.get(identifier).unwrap_or(identifier).clone(),
            },
        }
    }
}

impl<Id: PartialEq> Expression<Id> {
    pub fn is_equal_reference(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Cast { rhs: x, .. }, y) => x.is_equal_reference(y),
            (x, Self::Cast { rhs: y, .. }) => x.is_equal_reference(y),
            (
                Self::Access {
                    lhs: x_lhs,
                    rhs: x_rhs,
                },
                Self::Access {
                    lhs: y_lhs,
                    rhs: y_rhs,
                },
            ) => x_lhs.is_equal_reference(y_lhs) && x_rhs.is_equal_reference(y_rhs),
            (Self::Reference { identifier: x }, Self::Reference { identifier: y }) => x == y,
            _ => false,
        }
    }
}

#[cfg(test)]
mod expression {
    mod is_equal_reference {
        use crate::parser::expression;
        use nom::combinator::all_consuming;

        fn check(lhs: &str, rhs: &str, expected: bool) {
            let (_, lhs) = all_consuming(expression)(lhs).expect("Incorrect lhs.");
            let (_, rhs) = all_consuming(expression)(rhs).expect("Incorrect rhs.");
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

impl<Id: Clone + Ord> Game<Id> {
    pub fn create_mappings(
        &self,
        bindings: Vec<Binding<Id>>,
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
    pub fn infer(&self, identifier: &Id, edge: Option<&Edge<Id>>) -> Rc<Type<Id>> {
        if let Some((_, type_)) = edge.and_then(|edge| edge.get_binding(identifier)) {
            return type_.clone();
        }

        if let Ok(constant) = self.resolve_constant(identifier) {
            return constant.type_.clone();
        }

        if let Ok(variable) = self.resolve_variable(identifier) {
            return variable.type_.clone();
        }

        Rc::new(Type::from(vec![identifier.clone()]))
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
            (Type::Set { identifiers: lhs }, Type::Set { identifiers: rhs }) => {
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

    pub fn resolve_constant(&self, identifier: &Id) -> Result<&Constant<Id>, Error<Id>> {
        self.constants
            .iter()
            .find(|constant| &constant.identifier == identifier)
            .map_or_else(
                || {
                    self.make_error(ErrorReason::UnresolvedConstant {
                        identifier: identifier.clone(),
                    })
                },
                Ok,
            )
    }

    pub fn resolve_typedef(&self, identifier: &Id) -> Result<&Typedef<Id>, Error<Id>> {
        self.typedefs
            .iter()
            .find(|typedef| &typedef.identifier == identifier)
            .map_or_else(
                || {
                    self.make_error(ErrorReason::UnresolvedType {
                        identifier: identifier.clone(),
                    })
                },
                Ok,
            )
    }

    pub fn resolve_typedef_by_type<'a>(
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

    pub fn resolve_variable(&self, identifier: &Id) -> Result<&Variable<Id>, Error<Id>> {
        self.variables
            .iter()
            .find(|variable| &variable.identifier == identifier)
            .map_or_else(
                || {
                    self.make_error(ErrorReason::UnresolvedVariable {
                        identifier: identifier.clone(),
                    })
                },
                Ok,
            )
    }
}

impl<Id: PartialEq> Game<Id> {
    pub fn are_connected(&self, lhs: &EdgeName<Id>, rhs: &EdgeName<Id>) -> bool {
        self.edges
            .iter()
            .any(|edge| &edge.lhs == lhs && &edge.rhs == rhs)
    }

    pub fn is_reachability_target(&self, edge_name: &EdgeName<Id>) -> bool {
        self.edges.iter().any(|edge| matches!(&edge.label, EdgeLabel::Any { lhs, rhs } | EdgeLabel::Reachability { lhs, rhs, .. } if lhs == edge_name || rhs == edge_name))
    }

    pub fn incoming_edges<'a>(
        &'a self,
        edge_name: &'a EdgeName<Id>,
    ) -> impl Iterator<Item = &'a Edge<Id>> {
        self.edges.iter().filter(move |edge| &edge.rhs == edge_name)
    }

    pub fn outgoing_edges<'a>(
        &'a self,
        edge_name: &'a EdgeName<Id>,
    ) -> impl Iterator<Item = &'a Edge<Id>> {
        self.edges.iter().filter(move |edge| &edge.lhs == edge_name)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Pragma<Id> {
    Disjoint {
        #[serde(rename = "edgeName")]
        edge_name: EdgeName<Id>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Type<Id> {
    Arrow { lhs: Rc<Self>, rhs: Rc<Self> },
    Set { identifiers: Vec<Id> },
    TypeReference { identifier: Id },
}

impl<Id> Type<Id> {
    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set { .. })
    }
}

impl<Id: Clone + PartialEq> Type<Id> {
    pub fn resolve<'a>(&'a self, game: &'a Game<Id>) -> Result<&'a Self, Error<Id>> {
        if let Self::TypeReference { identifier } = self {
            game.resolve_typedef(identifier)?.type_.resolve(game)
        } else {
            Ok(self)
        }
    }

    pub fn values(&self, game: &Game<Id>) -> Result<Vec<Id>, Error<Id>> {
        match self {
            Self::Arrow { .. } => todo!(),
            Self::Set { identifiers } => Ok(identifiers.clone()),
            Self::TypeReference { identifier } => {
                game.resolve_typedef(identifier)?.type_.values(game)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "TypeDeclaration", tag = "kind")]
pub struct Typedef<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Value<Id> {
    Element { identifier: Id },
    Map { entries: Vec<ValueEntry<Id>> },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum ValueEntry<Id> {
    DefaultEntry {
        value: Rc<Value<Id>>,
    },
    NamedEntry {
        identifier: Id,
        value: Rc<Value<Id>>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "VariableDeclaration", tag = "kind")]
pub struct Variable<Id> {
    #[serde(rename = "defaultValue")]
    pub default_value: Rc<Value<Id>>,
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
}
