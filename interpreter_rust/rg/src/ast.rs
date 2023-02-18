use map_id::MapId;
use map_id_macro::MapId;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

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

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct EdgeName<Id> {
    pub parts: Vec<Rc<EdgeNamePart<Id>>>,
}

impl<Id> From<Vec<Rc<EdgeNamePart<Id>>>> for EdgeName<Id> {
    fn from(parts: Vec<Rc<EdgeNamePart<Id>>>) -> Self {
        Self { parts }
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

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub struct GameDeclaration<Id> {
    pub constants: Vec<Rc<ConstantDeclaration<Id>>>,
    pub edges: Vec<Rc<EdgeDeclaration<Id>>>,
    pub pragmas: Vec<Rc<Pragma<Id>>>,
    pub types: Vec<Rc<TypeDeclaration<Id>>>,
    pub variables: Vec<Rc<VariableDeclaration<Id>>>,
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
