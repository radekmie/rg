mod display;

use map_id::MapId;
use map_id_macro::MapId;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Statement<Id> {
    Assignment {
        identifier: Id,
        accessors: Vec<Arc<Expression<Id>>>,
        expression: Arc<Expression<Id>>,
    },
    AssignmentAny {
        identifier: Id,
        accessors: Vec<Arc<Expression<Id>>>,
        type_: Arc<Type<Id>>,
    },
    Branch {
        arms: Vec<Vec<Statement<Id>>>,
    },
    Call {
        identifier: Id,
        args: Vec<Arc<Expression<Id>>>,
    },
    If {
        expression: Arc<Expression<Id>>,
        then: Vec<Statement<Id>>,
        else_: Option<Vec<Statement<Id>>>,
    },
    Loop {
        body: Vec<Statement<Id>>,
    },
    Repeat {
        count: usize,
        body: Vec<Statement<Id>>,
    },
    Tag {
        symbol: Id,
    },
    TagVariable {
        symbol: Id,
    },
    While {
        expression: Arc<Expression<Id>>,
        body: Vec<Statement<Id>>,
    },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Function<Id> {
    pub reusable: bool,
    pub name: Id,
    pub args: Vec<FunctionArg<Id>>,
    pub body: Vec<Statement<Id>>,
}

impl<Id: PartialEq> Function<Id> {
    pub fn arg_index(&self, identifier: &Id) -> Option<usize> {
        self.args
            .iter()
            .position(|arg| arg.identifier == *identifier)
    }
}

impl Function<Arc<str>> {
    pub fn nth_arg_variable(&self, index: usize) -> Arc<str> {
        Arc::from(format!(
            "{}_arg{index}_{}",
            self.name, self.args[index].identifier
        ))
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FunctionArg<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FunctionDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
    pub cases: Vec<FunctionCase<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FunctionCase<Id> {
    pub identifier: Id,
    pub args: Vec<Arc<Pattern<Id>>>,
    pub body: Arc<Expression<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DomainDeclaration<Id> {
    pub identifier: Id,
    pub elements: Vec<DomainElement<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DomainElement<Id> {
    Generator {
        identifier: Id,
        args: Vec<DomainElementPattern<Id>>,
        values: Vec<DomainValue<Id>>,
    },
    Literal {
        identifier: Id,
    },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DomainElementPattern<Id> {
    Literal { identifier: Id },
    Variable { identifier: Id },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DomainValue<Id> {
    Range {
        identifier: Id,
        min: usize,
        max: usize,
    },
    Set {
        identifier: Id,
        elements: Vec<Id>,
    },
}

impl<Id> DomainValue<Id> {
    pub fn identifier(&self) -> &Id {
        match self {
            Self::Range { identifier, .. } => identifier,
            Self::Set { identifier, .. } => identifier,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error<Id> {
    DuplicatedDomainValue {
        identifier: Id,
    },
    DuplicatedMapKey {
        key: Value<Id>,
    },
    EmptyMap,
    FunctionCaseNotCovered {
        identifier: Id,
        args: Vec<Value<Id>>,
    },
    IncomparableValues {
        lhs: Value<Id>,
        rhs: Value<Id>,
    },
    IncorrectNumberOfArguments {
        identifier: Id,
        expected: usize,
        got: usize,
    },
    InvalidCondition {
        expression: Expression<Id>,
    },
    NotImplemented {
        message: &'static str,
    },
    UnknownAutomatonFunction {
        identifier: Id,
    },
    UnknownFunction {
        identifier: Id,
    },
    UnknownType {
        identifier: Id,
    },
    UnknownVariable {
        identifier: Id,
    },
}

// TODO: Implement MapId for trivial enums
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
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
        default_value: Option<Arc<Expression<Id>>>,
        parts: Vec<ExpressionMapPart<Id>>,
    },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ExpressionMapPart<Id> {
    pub pattern: Arc<Pattern<Id>>,
    pub expression: Arc<Expression<Id>>,
    pub domains: Vec<DomainValue<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Pattern<Id> {
    Constructor {
        identifier: Id,
        args: Vec<Arc<Pattern<Id>>>,
    },
    Literal {
        identifier: Id,
    },
    Variable {
        identifier: Id,
    },
    Wildcard,
}

impl<Id: std::fmt::Display> Pattern<Id> {
    /// Literals do not start with an uppercase letter.
    pub fn is_literal(id: &Id) -> bool {
        !id.to_string()
            .chars()
            .next()
            .is_some_and(char::is_uppercase)
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TypeDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Type<Id> {
    Function {
        lhs: Arc<Type<Id>>,
        rhs: Arc<Type<Id>>,
    },
    Name {
        identifier: Id,
    },
}

impl<Id> Type<Id> {
    pub fn new(identifier: Id) -> Self {
        Self::Name { identifier }
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Value<Id> {
    Constructor {
        identifier: Id,
        args: Vec<Arc<Value<Id>>>,
    },
    Element {
        identifier: Id,
    },
    Map {
        default_value: Option<Arc<Value<Id>>>,
        entries: Vec<ValueMapEntry<Id>>,
    },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ValueMapEntry<Id> {
    pub key: Arc<Value<Id>>,
    pub value: Arc<Value<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct VariableDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
    pub default_value: Option<Arc<Expression<Id>>>,
}

#[derive(Clone, Debug, Default, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Game<Id> {
    pub automaton: Vec<Function<Id>>,
    pub domains: Vec<DomainDeclaration<Id>>,
    pub functions: Vec<FunctionDeclaration<Id>>,
    pub variables: Vec<VariableDeclaration<Id>>,
    pub types: Vec<TypeDeclaration<Id>>,
}
