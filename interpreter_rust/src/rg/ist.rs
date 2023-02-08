use crate::utils::map_id::MapId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

// Interned strings that the interpreter relies on.
pub type RuntimeId = u16;
pub const LABEL_BEGIN: RuntimeId = 0;
pub const LABEL_END: RuntimeId = 1;
pub const LABEL_GOALS: RuntimeId = 2;
pub const LABEL_KEEPER: RuntimeId = 3;
pub const LABEL_PLAYER: RuntimeId = 4;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub struct Edge<Id: Ord> {
    pub label: EdgeLabel<Id>,
    pub next: Id,
}

impl<OldId: Ord, NewId: Ord> MapId<Edge<NewId>, OldId, NewId> for Edge<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Edge<NewId> {
        Edge {
            label: self.label.map_id(map),
            next: map(&self.next),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum EdgeLabel<Id: Ord> {
    Assignment {
        lhs: Expression<Id>,
        rhs: Expression<Id>,
    },
    Comparison {
        lhs: Expression<Id>,
        rhs: Expression<Id>,
        negated: bool,
    },
    Reachability {
        lhs: Id,
        rhs: Id,
        negated: bool,
    },
    Skip,
}

impl<OldId: Ord, NewId: Ord> MapId<EdgeLabel<NewId>, OldId, NewId> for EdgeLabel<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> EdgeLabel<NewId> {
        match self {
            EdgeLabel::Assignment { lhs, rhs } => EdgeLabel::Assignment {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            EdgeLabel::Comparison { lhs, rhs, negated } => EdgeLabel::Comparison {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
                negated: *negated,
            },
            EdgeLabel::Reachability { lhs, rhs, negated } => EdgeLabel::Reachability {
                lhs: map(lhs),
                rhs: map(rhs),
                negated: *negated,
            },
            EdgeLabel::Skip => EdgeLabel::Skip,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum Expression<Id: Ord> {
    Access { lhs: Rc<Self>, rhs: Rc<Self> },
    ConstantReference { identifier: Id },
    Literal { value: Rc<Value<Id>> },
    VariableReference { identifier: Id },
}

impl Expression<RuntimeId> {
    pub fn is_player_reference(&self) -> bool {
        matches!(self, Self::VariableReference { identifier } if *identifier == LABEL_PLAYER)
    }
}

impl<OldId: Ord, NewId: Ord> MapId<Expression<NewId>, OldId, NewId> for Expression<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Expression<NewId> {
        match self {
            Expression::Access { lhs, rhs } => Expression::Access {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            Expression::ConstantReference { identifier } => Expression::ConstantReference {
                identifier: map(identifier),
            },
            Expression::Literal { value } => Expression::Literal {
                value: value.map_id(map),
            },
            Expression::VariableReference { identifier } => Expression::VariableReference {
                identifier: map(identifier),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub struct Game<Id: Ord> {
    pub constants: BTreeMap<Id, Rc<Value<Id>>>,
    pub edges: BTreeMap<Id, Vec<Edge<Id>>>,
    pub types: BTreeMap<Id, Type<Id>>,
    pub variables: BTreeMap<Id, Variable<Id>>,
}

impl<OldId: Ord, NewId: Ord> MapId<Game<NewId>, OldId, NewId> for Game<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Game<NewId> {
        Game {
            constants: self.constants.map_id(map),
            edges: self.edges.map_id(map),
            types: self.types.map_id(map),
            variables: self.variables.map_id(map),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum Type<Id: Ord> {
    Arrow { lhs: Rc<Self>, rhs: Rc<Self> },
    Set { values: Vec<Value<Id>> },
}

impl<OldId: Ord, NewId: Ord> MapId<Type<NewId>, OldId, NewId> for Type<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Type<NewId> {
        match self {
            Type::Arrow { lhs, rhs } => Type::Arrow {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            Type::Set { values } => Type::Set {
                values: values.map_id(map),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum Value<Id: Ord> {
    Element {
        value: Id,
    },
    Map {
        #[serde(rename = "defaultValue")]
        default: Rc<Self>,
        values: Rc<BTreeMap<Id, Rc<Self>>>,
    },
}

impl Value<RuntimeId> {
    pub fn is_keeper(&self) -> bool {
        matches!(self, Self::Element { value } if *value == LABEL_KEEPER)
    }
}

impl<Id: fmt::Display + Ord> fmt::Display for Value<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Element { value } => write!(f, "{}", value),
            Self::Map { default, values } => {
                write!(f, "{{ :{}", default)?;
                for (key, value) in values.iter() {
                    write!(f, ", {}: {}", key, value)?;
                }
                write!(f, " }}")
            }
        }
    }
}

impl<OldId: Ord, NewId: Ord> MapId<Value<NewId>, OldId, NewId> for Value<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Value<NewId> {
        match self {
            Value::Element { value } => Value::Element { value: map(value) },
            Value::Map { default, values } => Value::Map {
                default: default.map_id(map),
                values: values.map_id(map),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub struct Variable<Id: Ord> {
    #[serde(rename = "defaultValue")]
    pub default: Rc<Value<Id>>,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
}

impl<OldId: Ord, NewId: Ord> MapId<Variable<NewId>, OldId, NewId> for Variable<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Variable<NewId> {
        Variable {
            default: self.default.map_id(map),
            type_: self.type_.map_id(map),
        }
    }
}
