use std::sync::Arc;
use utils::position::Span;

pub struct Identifier {
    pub span: Span,
    pub identifier: String,
}

pub enum Statement<Id> {
    Assignment {
        identifier: Id,
        accessiors: Vec<Arc<Expression<Id>>>,
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
        tpe: Type<Id>,
        body: Vec<Statement<Id>>,
    },
    Function {
        name: Id,
        args: Vec<FunctionArg<Id>>,
        body: Vec<Statement<Id>>,
    },
    Loop {
        body: Vec<Statement<Id>>,
    },
    Pragma {
        identifier: Id,
    },
    Tag {
        symbol: String,
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

pub struct FunctionArg<Id> {
    pub identifier: Id,
    pub tpe: Arc<Type<Id>>,
}

pub struct DomainDeclaration<Id> {
    pub identifier: Id,
    pub elements: Vec<DomainElement<Id>>,
}

pub enum DomainElement<Id> {
    Generator {
        identifier: Id,
        args: Vec<String>,
        values: Vec<DomainValue<Id>>,
    },
}

pub enum DomainValue<Id> {
    Range {
        identifier: Id,
        min: String,
        max: String,
    },
    Set {
        identifier: Id,
        values: Vec<String>,
    },
}

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
    Identifier {
        identifier: Id,
    },
    If {
        condition: Arc<Expression<Id>>,
        then: Arc<Expression<Id>>,
        else_: Arc<Expression<Id>>,
    },
    Literal {
        identifier: String,
    },
    Map {
        pattern: Arc<Pattern<Id>>,
        expression: Arc<Expression<Id>>,
        domains: Vec<DomainValue<Id>>,
    },
}

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

pub struct TypeDeclaration<Id> {
    pub identifier: Id,
    pub tpe: Type<Id>,
}

pub enum Type<Id> {
    Function {
        lhs: Arc<Type<Id>>,
        rhs: Arc<Type<Id>>,
    },
    Name {
        identifier: Id,
    },
}

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

pub struct ValueMapEntry<Id> {
    pub key: Arc<Value<Id>>,
    pub value: Arc<Value<Id>>,
}

pub struct VariableDeclaration<Id> {
    pub identifier: Id,
    pub tpe: Type<Id>,
    pub default_value: Option<Arc<Expression<Id>>>,
}
