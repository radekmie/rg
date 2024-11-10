mod display;

use map_id::MapId;
use map_id_macro::MapId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "AutomatonStatement", tag = "kind")]
pub enum Statement<Id> {
    #[serde(rename = "AutomatonAssignment")]
    Assignment {
        identifier: Id,
        accessors: Vec<Arc<Expression<Id>>>,
        expression: Arc<Expression<Id>>,
    },
    #[serde(rename = "AutomatonBranch")]
    Branch { arms: Vec<Vec<Statement<Id>>> },
    #[serde(rename = "AutomatonCall")]
    Call {
        identifier: Id,
        args: Vec<Arc<Expression<Id>>>,
    },
    #[serde(rename = "AutomatonForall")]
    Forall {
        identifier: Id,
        #[serde(rename = "type")]
        type_: Arc<Type<Id>>,
        body: Vec<Statement<Id>>,
    },
    #[serde(rename = "AutomatonIf")]
    If {
        expression: Arc<Expression<Id>>,
        body: Vec<Statement<Id>>,
    },
    #[serde(rename = "AutomatonLoop")]
    Loop { body: Vec<Statement<Id>> },
    #[serde(rename = "AutomatonTag")]
    Tag { symbol: Id },
    #[serde(rename = "AutomatonWhile")]
    While {
        expression: Arc<Expression<Id>>,
        body: Vec<Statement<Id>>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "AutomatonFunction", tag = "kind")]
pub struct Function<Id> {
    pub name: Id,
    pub args: Vec<FunctionArg<Id>>,
    pub body: Vec<Statement<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename = "AutomatonFunctionArgument", tag = "kind")]
pub struct FunctionArg<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Arc<Type<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct FunctionDeclaration<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Arc<Type<Id>>,
    pub cases: Vec<FunctionCase<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct FunctionCase<Id> {
    pub identifier: Id,
    pub args: Vec<Arc<Pattern<Id>>>,
    pub body: Arc<Expression<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct DomainDeclaration<Id> {
    pub identifier: Id,
    pub elements: Vec<DomainElement<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum DomainElement<Id> {
    #[serde(rename = "DomainGenerator")]
    Generator {
        identifier: Id,
        args: Vec<Id>,
        values: Vec<DomainValue<Id>>,
    },
    #[serde(rename = "DomainLiteral")]
    Literal { identifier: Id },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum DomainValue<Id> {
    #[serde(rename = "DomainRange")]
    Range {
        identifier: Id,
        min: usize,
        max: usize,
    },
    #[serde(rename = "DomainSet")]
    Set { identifier: Id, elements: Vec<Id> },
}

impl<Id> DomainValue<Id> {
    pub fn identifier(&self) -> &Id {
        match self {
            Self::Range { identifier, .. } => identifier,
            Self::Set { identifier, .. } => identifier,
        }
    }
}

// TODO: Implement MapId for trivial enums
#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Binop {
    Add,
    And,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Mod,
    Ne,
    Or,
    Sub,
}

impl Binop {
    pub fn precedence(&self) -> usize {
        match self {
            Self::Or => 0,
            Self::And => 1,
            Self::Eq | Self::Gt | Self::Gte | Self::Lt | Self::Lte | Self::Ne => 2,
            Self::Add | Self::Mod | Self::Sub => 3,
        }
    }
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for Binop {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Expression<Id> {
    #[serde(rename = "ExpressionAccess")]
    Access {
        lhs: Arc<Expression<Id>>,
        rhs: Arc<Expression<Id>>,
    },
    BinExpr {
        lhs: Arc<Expression<Id>>,
        op: Binop,
        rhs: Arc<Expression<Id>>,
    },
    #[serde(rename = "ExpressionCall")]
    Call {
        expression: Arc<Expression<Id>>,
        args: Vec<Arc<Expression<Id>>>,
    },
    #[serde(rename = "ExpressionConstructor")]
    Constructor {
        identifier: Id,
        args: Vec<Arc<Expression<Id>>>,
    },
    #[serde(rename = "ExpressionIf")]
    If {
        cond: Arc<Expression<Id>>,
        then: Arc<Expression<Id>>,
        #[serde(rename = "else")]
        else_: Arc<Expression<Id>>,
    },
    #[serde(rename = "ExpressionLiteral")]
    Literal { identifier: Id },
    #[serde(rename = "ExpressionMap")]
    Map {
        pattern: Arc<Pattern<Id>>,
        expression: Arc<Expression<Id>>,
        domains: Vec<DomainValue<Id>>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Pattern<Id> {
    #[serde(rename = "PatternConstructor")]
    Constructor {
        identifier: Id,
        args: Vec<Arc<Pattern<Id>>>,
    },
    #[serde(rename = "PatternLiteral")]
    Literal { identifier: Id },
    #[serde(rename = "PatternVariable")]
    Variable { identifier: Id },
    #[serde(rename = "PatternWildcard")]
    Wildcard,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct TypeDeclaration<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Arc<Type<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Type<Id> {
    #[serde(rename = "TypeFunction")]
    Function {
        lhs: Arc<Type<Id>>,
        rhs: Arc<Type<Id>>,
    },
    #[serde(rename = "TypeName")]
    Name { identifier: Id },
}

impl<Id> Type<Id> {
    pub fn new(identifier: Id) -> Self {
        Self::Name { identifier }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub enum Value<Id> {
    #[serde(rename = "ValueConstructor")]
    Constructor {
        identifier: Id,
        args: Vec<Arc<Value<Id>>>,
    },
    #[serde(rename = "ValueElement")]
    Element { identifier: Id },
    #[serde(rename = "ValueMap")]
    Map { entries: Vec<ValueMapEntry<Id>> },
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct ValueMapEntry<Id> {
    pub key: Arc<Value<Id>>,
    pub value: Arc<Value<Id>>,
}

#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind")]
pub struct VariableDeclaration<Id> {
    pub identifier: Id,
    #[serde(rename = "type")]
    pub type_: Arc<Type<Id>>,
    #[serde(rename = "defaultValue")]
    pub default_value: Option<Arc<Expression<Id>>>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename = "GameDeclaration")]
pub struct Game<Id> {
    pub automaton: Vec<Function<Id>>,
    pub domains: Vec<DomainDeclaration<Id>>,
    pub functions: Vec<FunctionDeclaration<Id>>,
    pub variables: Vec<VariableDeclaration<Id>>,
    #[serde(skip_serializing, default = "Vec::new")]
    pub types: Vec<TypeDeclaration<Id>>,
}
