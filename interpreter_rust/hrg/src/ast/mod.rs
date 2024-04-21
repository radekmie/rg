mod display;

use map_id::MapId;
use map_id_macro::MapId;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
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
    #[serde(rename = "AutomatonLoop")]
    Loop { body: Vec<Statement<Id>> },
    #[serde(rename = "AutomatonPragma")]
    Pragma { identifier: Id },
    #[serde(rename = "AutomatonTag")]
    Tag { symbol: Id },
    #[serde(rename = "AutomatonWhen")]
    When {
        expression: Arc<Expression<Id>>,
        body: Vec<Statement<Id>>,
    },
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
    Range { identifier: Id, min: Id, max: Id },
    #[serde(rename = "DomainSet")]
    Set { identifier: Id, elements: Vec<Id> },
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Binop {
    #[serde(rename = "ExpressionAdd")]
    Add,
    #[serde(rename = "ExpressionAnd")]
    And,
    #[serde(rename = "ExpressionEq")]
    Eq,
    #[serde(rename = "ExpressionGt")]
    Gt,
    #[serde(rename = "ExpressionGte")]
    Gte,
    #[serde(rename = "ExpressionLt")]
    Lt,
    #[serde(rename = "ExpressionLte")]
    Lte,
    #[serde(rename = "ExpressionNe")]
    Ne,
    #[serde(rename = "ExpressionOr")]
    Or,
    #[serde(rename = "ExpressionSub")]
    Sub,
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for Binop {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

// TODO: Implement Deserialize by hand
#[derive(Clone, Debug, Deserialize, Eq, MapId, Ord, PartialEq, PartialOrd)]
#[serde(tag = "kind")]
pub enum Expression<Id> {
    Access {
        lhs: Arc<Expression<Id>>,
        rhs: Arc<Expression<Id>>,
    },
    BinExpr {
        lhs: Arc<Expression<Id>>,
        op: Binop,
        rhs: Arc<Expression<Id>>,
    },
    Call {
        expression: Arc<Expression<Id>>,
        args: Vec<Arc<Expression<Id>>>,
    },
    Constructor {
        identifier: Id,
        args: Vec<Arc<Expression<Id>>>,
    },
    If {
        cond: Arc<Expression<Id>>,
        then: Arc<Expression<Id>>,
        else_: Arc<Expression<Id>>,
    },
    Literal {
        identifier: Id,
    },
    Map {
        pattern: Arc<Pattern<Id>>,
        expression: Arc<Expression<Id>>,
        domains: Vec<DomainValue<Id>>,
    },
}

impl<Id> Serialize for Expression<Id>
where
    Id: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Expression::BinExpr { lhs, op, rhs } => {
                let mut state: <S as Serializer>::SerializeStruct =
                    serializer.serialize_struct("BinExpr", 3)?;
                state.serialize_field("kind", op)?;
                state.serialize_field("lhs", lhs)?;
                state.serialize_field("rhs", rhs)?;
                state.end()
            }
            Expression::Access { lhs, rhs } => {
                let mut state: <S as Serializer>::SerializeStruct =
                    serializer.serialize_struct("Access", 3)?;
                state.serialize_field("kind", "ExpressionAccess")?;
                state.serialize_field("lhs", lhs)?;
                state.serialize_field("rhs", rhs)?;
                state.end()
            }
            Expression::Call { expression, args } => {
                let mut state: <S as Serializer>::SerializeStruct =
                    serializer.serialize_struct("Call", 3)?;
                state.serialize_field("kind", "ExpressionCall")?;
                state.serialize_field("expression", expression)?;
                state.serialize_field("args", args)?;
                state.end()
            }
            Expression::Constructor { identifier, args } => {
                let mut state: <S as Serializer>::SerializeStruct =
                    serializer.serialize_struct("Constructor", 3)?;
                state.serialize_field("kind", "ExpressionConstructor")?;
                state.serialize_field("identifier", identifier)?;
                state.serialize_field("args", args)?;
                state.end()
            }
            Expression::If { cond, then, else_ } => {
                let mut state: <S as Serializer>::SerializeStruct =
                    serializer.serialize_struct("If", 4)?;
                state.serialize_field("kind", "ExpressionIf")?;
                state.serialize_field("cond", cond)?;
                state.serialize_field("then", then)?;
                state.serialize_field("else", else_)?;
                state.end()
            }
            Expression::Literal { identifier } => {
                let mut state: <S as Serializer>::SerializeStruct =
                    serializer.serialize_struct("Literal", 2)?;
                state.serialize_field("kind", "ExpressionLiteral")?;
                state.serialize_field("identifier", identifier)?;
                state.end()
            }
            Expression::Map {
                pattern,
                expression,
                domains,
            } => {
                let mut state: <S as Serializer>::SerializeStruct =
                    serializer.serialize_struct("Map", 4)?;
                state.serialize_field("kind", "ExpressionMap")?;
                state.serialize_field("pattern", pattern)?;
                state.serialize_field("expression", expression)?;
                state.serialize_field("domains", domains)?;
                state.end()
            }
        }
    }
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
#[serde(tag = "kind")]
pub struct GameDeclaration<Id> {
    pub automaton: Vec<Function<Id>>,
    pub domains: Vec<DomainDeclaration<Id>>,
    pub functions: Vec<FunctionDeclaration<Id>>,
    pub variables: Vec<VariableDeclaration<Id>>,
    #[serde(skip)]
    pub types: Vec<TypeDeclaration<Id>>,
}
