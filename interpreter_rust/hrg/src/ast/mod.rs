mod display;

use std::sync::Arc;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Statement<Id> {
    Assignment {
        identifier: Id,
        accessors: Vec<Arc<Expression<Id>>>,
        expression: Arc<Expression<Id>>,
    },
    Branch {
        arms: Vec<Vec<Statement<Id>>>,
    },
    Call {
        identifier: Id,
        args: Vec<Arc<Expression<Id>>>,
    },
    Forall {
        identifier: Id,
        type_: Arc<Type<Id>>,
        body: Vec<Statement<Id>>,
    },
    Loop {
        body: Vec<Statement<Id>>,
    },
    Pragma {
        identifier: Id,
    },
    Tag {
        symbol: Id,
    },
    When {
        condition: Arc<Expression<Id>>,
        body: Vec<Statement<Id>>,
    },
    While {
        condition: Arc<Expression<Id>>,
        body: Vec<Statement<Id>>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Function<Id> {
    pub name: Id,
    pub args: Vec<FunctionArg<Id>>,
    pub body: Vec<Statement<Id>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FunctionArg<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FunctionDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
    pub cases: Vec<FunctionCase<Id>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct FunctionCase<Id> {
    pub identifier: Id,
    pub args: Vec<Arc<Pattern<Id>>>,
    pub body: Arc<Expression<Id>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DomainDeclaration<Id> {
    pub identifier: Id,
    pub elements: Vec<DomainElement<Id>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DomainElement<Id> {
    Generator {
        identifier: Id,
        args: Vec<Id>,
        values: Vec<DomainValue<Id>>,
    },
    Literal {
        identifier: Id,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DomainValue<Id> {
    Range { identifier: Id, min: Id, max: Id },
    Set { identifier: Id, values: Vec<Id> },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Binop {
    Add,
    And,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Ne,
    Or,
    Sub,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
        condition: Arc<Expression<Id>>,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Pattern<Id> {
    Constructor {
        identifier: Id,
        args: Vec<Arc<Pattern<Id>>>,
    },
    Literal {
        pattern: String,
    },
    Variable {
        identifier: Id,
    },
    Wildcard,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Value<Id> {
    ValueConstructor {
        identifier: Id,
        args: Vec<Arc<Value<Id>>>,
    },
    Element {
        identifier: Id,
    },
    Map {
        entries: Vec<ValueMapEntry<Id>>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValueMapEntry<Id> {
    pub key: Arc<Value<Id>>,
    pub value: Arc<Value<Id>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VariableDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
    pub default_value: Option<Arc<Expression<Id>>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Default)]
pub struct GameDeclaration<Id> {
    pub automaton: Vec<Function<Id>>,
    pub domains: Vec<DomainDeclaration<Id>>,
    pub functions: Vec<FunctionDeclaration<Id>>,
    pub variables: Vec<VariableDeclaration<Id>>,
    pub types: Vec<TypeDeclaration<Id>>,
}
