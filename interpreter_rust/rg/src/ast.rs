use map_id::MapId;
use map_id_macro::MapId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

pub type Mapping<Id> = BTreeMap<Id, Id>;
pub type Binding<'a, Id> = (&'a Id, &'a Rc<Type<Id>>);

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct ConstantDeclaration<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
    pub value: Rc<Value<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct EdgeDeclaration<Id> {
    pub label: Rc<EdgeLabel<Id>>,
    pub lhs: Rc<EdgeName<Id>>,
    pub rhs: Rc<EdgeName<Id>>,
}

impl<Id: Ord> EdgeDeclaration<Id> {
    pub fn bindings(&self) -> BTreeSet<Binding<Id>> {
        let mut bindings = self.lhs.bindings();
        bindings.extend(self.rhs.bindings());
        bindings
    }
}

impl EdgeDeclaration<String> {
    pub fn substitute_bindings(&self, mapping: &Mapping<String>) -> Self {
        Self {
            label: Rc::new(self.label.substitute_bindings(mapping)),
            lhs: Rc::new(self.lhs.substitute_bindings(mapping)),
            rhs: Rc::new(self.rhs.substitute_bindings(mapping)),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum EdgeLabel<Id> {
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
        lhs: Rc<EdgeName<Id>>,
        rhs: Rc<EdgeName<Id>>,
        negated: bool,
    },
    Skip,
}

impl<Id: PartialEq> EdgeLabel<Id> {
    pub fn is_self_assignment(&self) -> bool {
        matches!(self, EdgeLabel::Assignment { lhs, rhs } if lhs.is_equal_reference(rhs))
    }
}

impl EdgeLabel<String> {
    pub fn substitute_bindings(&self, mapping: &Mapping<String>) -> Self {
        match self {
            Self::Assignment { lhs, rhs } => Self::Assignment {
                lhs: Rc::new(lhs.substitute_bindings(mapping)),
                rhs: Rc::new(rhs.substitute_bindings(mapping)),
            },
            Self::Comparison { lhs, rhs, negated } => Self::Comparison {
                lhs: Rc::new(lhs.substitute_bindings(mapping)),
                rhs: Rc::new(rhs.substitute_bindings(mapping)),
                negated: *negated,
            },
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct EdgeName<Id> {
    pub parts: Vec<Rc<EdgeNamePart<Id>>>,
}

impl<Id: Ord> EdgeName<Id> {
    pub fn bindings(&self) -> BTreeSet<Binding<Id>> {
        self.parts
            .iter()
            .flat_map(|edge_name_part| edge_name_part.binding())
            .collect()
    }
}

impl EdgeName<String> {
    pub fn substitute_bindings(&self, mapping: &Mapping<String>) -> Self {
        let identifier = self
            .parts
            .iter()
            .map(|edge_name_part| match &**edge_name_part {
                EdgeNamePart::Binding { identifier, .. } => mapping.get(identifier).unwrap(),
                EdgeNamePart::Literal { identifier } => identifier,
            })
            .cloned()
            .collect::<Vec<_>>()
            .join("__bind__");
        Self {
            parts: vec![Rc::new(EdgeNamePart::Literal { identifier })],
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

#[derive(Debug)]
pub struct Error<Id> {
    pub game_declaration: GameDeclaration<Id>,
    pub reason: ErrorReason<Id>,
}

#[derive(Debug)]
pub enum ErrorReason<Id> {
    EmptySetType {
        identifier: Id,
    },
    SetTypeExpected {
        identifier: Id,
    },
    TypeDeclarationMismatch {
        expected: Rc<Type<Id>>,
        identifier: Id,
        resolved: Rc<Type<Id>>,
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

impl<Id: PartialEq> Expression<Id> {
    pub fn is_equal_reference(&self, other: &Self) -> bool {
        match (self, other) {
            (Expression::Cast { rhs: x, .. }, y) => x.is_equal_reference(y),
            (x, Expression::Cast { rhs: y, .. }) => x.is_equal_reference(y),
            (
                Expression::Access {
                    lhs: x_lhs,
                    rhs: x_rhs,
                },
                Expression::Access {
                    lhs: y_lhs,
                    rhs: y_rhs,
                },
            ) => x_lhs.is_equal_reference(y_lhs) && x_rhs.is_equal_reference(y_rhs),
            (Expression::Reference { identifier: x }, Expression::Reference { identifier: y }) => {
                x == y
            }
            _ => false,
        }
    }
}

impl Expression<String> {
    pub fn substitute_bindings(&self, bindings: &Mapping<String>) -> Self {
        match self {
            Self::Access { lhs, rhs } => Self::Access {
                lhs: Rc::new(lhs.substitute_bindings(bindings)),
                rhs: Rc::new(rhs.substitute_bindings(bindings)),
            },
            Self::Cast { lhs, rhs } => Self::Cast {
                lhs: lhs.clone(),
                rhs: Rc::new(rhs.substitute_bindings(bindings)),
            },
            Self::Reference { identifier } => Self::Reference {
                identifier: bindings.get(identifier).unwrap_or(identifier).clone(),
            },
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
#[serde(tag = "kind")]
pub struct GameDeclaration<Id> {
    pub constants: Vec<Rc<ConstantDeclaration<Id>>>,
    pub edges: Vec<Rc<EdgeDeclaration<Id>>>,
    pub pragmas: Vec<Rc<Pragma<Id>>>,
    pub types: Vec<Rc<TypeDeclaration<Id>>>,
    pub variables: Vec<Rc<VariableDeclaration<Id>>>,
}

impl<Id: Clone + PartialEq> GameDeclaration<Id> {
    pub fn is_assignable_type(
        &self,
        lhs: &Rc<Type<Id>>,
        rhs: &Rc<Type<Id>>,
        is_strict: bool,
    ) -> Result<bool, Error<Id>> {
        let lhs = self.resolve_type_reference(lhs)?;
        let rhs = self.resolve_type_reference(rhs)?;

        match (&**lhs, &**rhs) {
            (
                Type::Arrow {
                    lhs: lhs_lhs,
                    rhs: lhs_rhs,
                },
                Type::Arrow {
                    lhs: rhs_lhs,
                    rhs: rhs_rhs,
                },
            ) => Ok(self.is_assignable_type(lhs_rhs, rhs_rhs, is_strict)?
                && self.is_assignable_type(
                    &Rc::new(Type::TypeReference {
                        identifier: lhs_lhs.clone(),
                    }),
                    &Rc::new(Type::TypeReference {
                        identifier: rhs_lhs.clone(),
                    }),
                    is_strict,
                )?),
            (Type::Set { identifiers: lhs }, Type::Set { identifiers: rhs }) => {
                if is_strict {
                    Ok(rhs.iter().all(|x| lhs.contains(x)))
                } else {
                    Ok(rhs.iter().any(|x| lhs.contains(x)))
                }
            }
            _ => Ok(false),
        }
    }

    pub fn is_equal_type(
        &self,
        lhs: &Rc<Type<Id>>,
        rhs: &Rc<Type<Id>>,
        is_strict: bool,
    ) -> Result<bool, Error<Id>> {
        Ok(self.is_assignable_type(lhs, rhs, is_strict)?
            && self.is_assignable_type(rhs, lhs, is_strict)?)
    }

    pub fn resolve_type(&self, identifier: &Id) -> Result<&Rc<TypeDeclaration<Id>>, Error<Id>> {
        self.types
            .iter()
            .find(|type_declaration| &type_declaration.identifier == identifier)
            .ok_or_else(|| Error {
                game_declaration: self.clone(),
                reason: ErrorReason::UnresolvedType {
                    identifier: identifier.clone(),
                },
            })
    }

    pub fn resolve_type_reference<'a>(
        &'a self,
        type_: &'a Rc<Type<Id>>,
    ) -> Result<&'a Rc<Type<Id>>, Error<Id>> {
        match &**type_ {
            Type::TypeReference { identifier } => {
                self.resolve_type_reference(&self.resolve_type(identifier)?.type_)
            }
            _ => Ok(type_),
        }
    }

    pub fn resolve_variable(
        &self,
        identifier: &Id,
    ) -> Result<&Rc<VariableDeclaration<Id>>, Error<Id>> {
        self.variables
            .iter()
            .find(|variable_declaration| &variable_declaration.identifier == identifier)
            .ok_or_else(|| Error {
                game_declaration: self.clone(),
                reason: ErrorReason::UnresolvedVariable {
                    identifier: identifier.clone(),
                },
            })
    }

    pub fn type_values(&self, type_: &Rc<Type<Id>>) -> Result<Vec<Id>, Error<Id>> {
        match &**type_ {
            Type::Arrow { .. } => todo!(),
            Type::Set { identifiers } => Ok(identifiers.clone()),
            Type::TypeReference { identifier } => {
                self.type_values(&self.resolve_type(identifier)?.type_)
            }
        }
    }
}

impl GameDeclaration<String> {
    pub fn create_mappings(
        &self,
        bindings: BTreeSet<Binding<String>>,
    ) -> Vec<BTreeMap<String, String>> {
        let mut mappings = vec![BTreeMap::default()];
        for (identifier, type_) in bindings {
            // FIXME: Error handling.
            let values = self.type_values(type_).unwrap();
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

        mappings
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Pragma<Id> {
    Disjoint {
        #[serde(rename = "edgeName")]
        edge_name: Rc<EdgeName<Id>>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Type<Id> {
    Arrow { lhs: Id, rhs: Rc<Self> },
    Set { identifiers: Vec<Id> },
    TypeReference { identifier: Id },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct TypeDeclaration<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Value<Id> {
    Element { identifier: Id },
    Map { entries: Vec<Rc<ValueEntry<Id>>> },
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
#[serde(tag = "kind")]
pub struct VariableDeclaration<Id> {
    #[serde(rename = "defaultValue")]
    pub default_value: Rc<Value<Id>>,
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
}
