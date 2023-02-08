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

impl<Id> From<(Id, Rc<Type<Id>>, Rc<Value<Id>>)> for ConstantDeclaration<Id> {
    fn from((identifier, type_, value): (Id, Rc<Type<Id>>, Rc<Value<Id>>)) -> Self {
        Self {
            identifier,
            type_,
            value,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct EdgeDeclaration<Id> {
    pub label: Rc<EdgeLabel<Id>>,
    pub lhs: Rc<EdgeName<Id>>,
    pub rhs: Rc<EdgeName<Id>>,
}

impl<Id> From<(Rc<EdgeName<Id>>, Rc<EdgeName<Id>>, Rc<EdgeLabel<Id>>)> for EdgeDeclaration<Id> {
    fn from((lhs, rhs, label): (Rc<EdgeName<Id>>, Rc<EdgeName<Id>>, Rc<EdgeLabel<Id>>)) -> Self {
        Self { label, lhs, rhs }
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

impl<Id> From<(Rc<Expression<Id>>, Rc<Expression<Id>>)> for EdgeLabel<Id> {
    fn from((lhs, rhs): (Rc<Expression<Id>>, Rc<Expression<Id>>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl<Id> From<(Rc<Expression<Id>>, bool, Rc<Expression<Id>>)> for EdgeLabel<Id> {
    fn from((lhs, negated, rhs): (Rc<Expression<Id>>, bool, Rc<Expression<Id>>)) -> Self {
        Self::Comparison { lhs, rhs, negated }
    }
}

impl<Id> From<(bool, Rc<EdgeName<Id>>, Rc<EdgeName<Id>>)> for EdgeLabel<Id> {
    fn from((negated, lhs, rhs): (bool, Rc<EdgeName<Id>>, Rc<EdgeName<Id>>)) -> Self {
        Self::Reachability { lhs, rhs, negated }
    }
}

impl<Id> From<()> for EdgeLabel<Id> {
    fn from(_: ()) -> Self {
        Self::Skip
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

impl<Id> From<(Id, Rc<Type<Id>>)> for EdgeNamePart<Id> {
    fn from((identifier, type_): (Id, Rc<Type<Id>>)) -> Self {
        Self::Binding { identifier, type_ }
    }
}

impl<Id> From<Id> for EdgeNamePart<Id> {
    fn from(identifier: Id) -> Self {
        Self::Literal { identifier }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Expression<Id> {
    Access { lhs: Rc<Self>, rhs: Rc<Self> },
    Cast { lhs: Rc<Type<Id>>, rhs: Rc<Self> },
    Reference { identifier: Id },
}

impl<Id> From<(Rc<Expression<Id>>, Rc<Expression<Id>>)> for Expression<Id> {
    fn from((lhs, rhs): (Rc<Expression<Id>>, Rc<Expression<Id>>)) -> Self {
        Self::Access { lhs, rhs }
    }
}

impl<Id> From<(Id, Rc<Expression<Id>>)> for Expression<Id> {
    fn from((identifier, rhs): (Id, Rc<Expression<Id>>)) -> Self {
        Self::Cast {
            lhs: Type::TypeReference { identifier }.into(),
            rhs,
        }
    }
}

impl<Id> From<Id> for Expression<Id> {
    fn from(identifier: Id) -> Self {
        Self::Reference { identifier }
    }
}

#[derive(Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub struct GameDeclaration<Id> {
    pub constants: Vec<Rc<ConstantDeclaration<Id>>>,
    pub edges: Vec<Rc<EdgeDeclaration<Id>>>,
    pub types: Vec<Rc<TypeDeclaration<Id>>>,
    pub variables: Vec<Rc<VariableDeclaration<Id>>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Type<Id> {
    Arrow { lhs: Id, rhs: Rc<Self> },
    Set { identifiers: Vec<Id> },
    TypeReference { identifier: Id },
}

impl<Id> From<(Id, Rc<Type<Id>>)> for Type<Id> {
    fn from((lhs, rhs): (Id, Rc<Type<Id>>)) -> Self {
        Self::Arrow { lhs, rhs }
    }
}

impl<Id> From<Vec<Id>> for Type<Id> {
    fn from(identifiers: Vec<Id>) -> Self {
        Self::Set { identifiers }
    }
}

impl<Id> From<Id> for Type<Id> {
    fn from(identifier: Id) -> Self {
        Self::TypeReference { identifier }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct TypeDeclaration<Id> {
    identifier: Id,
    #[serde(rename = "type")]
    type_: Rc<Type<Id>>,
}

impl<Id> From<(Id, Rc<Type<Id>>)> for TypeDeclaration<Id> {
    fn from((identifier, type_): (Id, Rc<Type<Id>>)) -> Self {
        Self { identifier, type_ }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Value<Id> {
    Element { identifier: Id },
    Map { entries: Vec<Rc<ValueEntry<Id>>> },
}

impl<Id> From<Id> for Value<Id> {
    fn from(identifier: Id) -> Self {
        Self::Element { identifier }
    }
}

impl<Id> From<Vec<Rc<ValueEntry<Id>>>> for Value<Id> {
    fn from(entries: Vec<Rc<ValueEntry<Id>>>) -> Self {
        Self::Map { entries }
    }
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

impl<Id> From<(Option<Id>, Rc<Value<Id>>)> for ValueEntry<Id> {
    fn from((identifier, value): (Option<Id>, Rc<Value<Id>>)) -> Self {
        match identifier {
            None => Self::DefaultEntry { value },
            Some(identifier) => Self::NamedEntry { identifier, value },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct VariableDeclaration<Id> {
    #[serde(rename = "defaultValue")]
    pub default_value: Rc<Value<Id>>,
    identifier: Id,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
}

impl<Id> From<(Id, Rc<Type<Id>>, Rc<Value<Id>>)> for VariableDeclaration<Id> {
    fn from((identifier, type_, default_value): (Id, Rc<Type<Id>>, Rc<Value<Id>>)) -> Self {
        Self {
            default_value,
            identifier,
            type_,
        }
    }
}
