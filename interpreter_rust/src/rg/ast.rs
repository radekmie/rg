use crate::utils::map_id::MapId;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<ConstantDeclaration<NewId>, OldId, NewId> for ConstantDeclaration<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> ConstantDeclaration<NewId> {
        ConstantDeclaration {
            identifier: map(&self.identifier),
            type_: self.type_.map_id(map),
            value: self.value.map_id(map),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<EdgeDeclaration<NewId>, OldId, NewId> for EdgeDeclaration<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> EdgeDeclaration<NewId> {
        EdgeDeclaration {
            label: self.label.map_id(map),
            lhs: self.lhs.map_id(map),
            rhs: self.rhs.map_id(map),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<EdgeLabel<NewId>, OldId, NewId> for EdgeLabel<OldId> {
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
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
                negated: *negated,
            },
            EdgeLabel::Skip => EdgeLabel::Skip,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct EdgeName<Id> {
    pub parts: Vec<Rc<EdgeNamePart<Id>>>,
}

impl<Id> From<Vec<Rc<EdgeNamePart<Id>>>> for EdgeName<Id> {
    fn from(parts: Vec<Rc<EdgeNamePart<Id>>>) -> Self {
        Self { parts }
    }
}

impl<OldId, NewId> MapId<EdgeName<NewId>, OldId, NewId> for EdgeName<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> EdgeName<NewId> {
        EdgeName {
            parts: self.parts.map_id(map),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<EdgeNamePart<NewId>, OldId, NewId> for EdgeNamePart<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> EdgeNamePart<NewId> {
        match self {
            EdgeNamePart::Binding { identifier, type_ } => EdgeNamePart::Binding {
                identifier: map(identifier),
                type_: type_.map_id(map),
            },
            EdgeNamePart::Literal { identifier } => EdgeNamePart::Literal {
                identifier: map(identifier),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<Expression<NewId>, OldId, NewId> for Expression<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Expression<NewId> {
        match self {
            Expression::Access { lhs, rhs } => Expression::Access {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            Expression::Cast { lhs, rhs } => Expression::Cast {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            Expression::Reference { identifier } => Expression::Reference {
                identifier: map(identifier),
            },
        }
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

impl<OldId, NewId> MapId<GameDeclaration<NewId>, OldId, NewId> for GameDeclaration<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> GameDeclaration<NewId> {
        GameDeclaration {
            constants: self.constants.map_id(map),
            edges: self.edges.map_id(map),
            types: self.types.map_id(map),
            variables: self.variables.map_id(map),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<Type<NewId>, OldId, NewId> for Type<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Type<NewId> {
        match self {
            Type::Arrow { lhs, rhs } => Type::Arrow {
                lhs: map(lhs),
                rhs: rhs.map_id(map),
            },
            Type::Set { identifiers } => Type::Set {
                identifiers: identifiers.iter().map(map).collect(),
            },
            Type::TypeReference { identifier } => Type::TypeReference {
                identifier: map(identifier),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<TypeDeclaration<NewId>, OldId, NewId> for TypeDeclaration<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> TypeDeclaration<NewId> {
        TypeDeclaration {
            identifier: map(&self.identifier),
            type_: self.type_.map_id(map),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<Value<NewId>, OldId, NewId> for Value<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Value<NewId> {
        match self {
            Value::Element { identifier } => Value::Element {
                identifier: map(identifier),
            },
            Value::Map { entries } => Value::Map {
                entries: entries.map_id(map),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<ValueEntry<NewId>, OldId, NewId> for ValueEntry<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> ValueEntry<NewId> {
        match self {
            ValueEntry::DefaultEntry { value } => ValueEntry::DefaultEntry {
                value: value.map_id(map),
            },
            ValueEntry::NamedEntry { identifier, value } => ValueEntry::NamedEntry {
                identifier: map(identifier),
                value: value.map_id(map),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

impl<OldId, NewId> MapId<VariableDeclaration<NewId>, OldId, NewId> for VariableDeclaration<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> VariableDeclaration<NewId> {
        VariableDeclaration {
            default_value: self.default_value.map_id(map),
            identifier: map(&self.identifier),
            type_: self.type_.map_id(map),
        }
    }
}
