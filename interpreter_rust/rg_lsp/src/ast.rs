use crate::position::Span;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Constant {
    pub span: Span,
    pub identifier: Identifier,
    pub type_: Box<Type>,
    pub value: Box<Value>,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge {
    pub span: Span,
    pub lhs: EdgeName,
    pub rhs: EdgeName,
    pub label: EdgeLabel,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeLabel {
    Assignment {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Comparison {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        negated: bool,
    },
    Reachability {
        span: Span,
        lhs: EdgeName,
        rhs: EdgeName,
        negated: bool,
    },
    Skip {
        span: Span
    },
    Tag {
        symbol: Identifier,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EdgeName {
    pub span: Span,
    pub parts: Vec<EdgeNamePart>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeNamePart {
    Binding {
        span: Span,
        identifier: Identifier,
        type_: Box<Type>,
    },
    Literal {
        identifier: Identifier,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Type {
    Arrow {
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Set {
        span: Span,
        identifiers: Vec<Identifier>,
    },
    TypeReference {
        identifier: Identifier,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Expression {
    Access {
        span: Span,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Cast {
        span: Span,
        lhs: Box<Type>,
        rhs: Box<Self>,
    },
    Reference {
        identifier: Identifier,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Element {
        identifier: Identifier,
    },
    Map {
        span: Span,
        entries: Vec<ValueEntry>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValueEntry {
    pub span: Span,
    pub identifier: Option<Identifier>,
    pub value: Box<Value>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Variable {
    pub span: Span,
    pub default_value: Box<Value>,
    pub identifier: Identifier,
    pub type_: Box<Type>,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Typedef {
    pub span: Span,
    pub identifier: Identifier,
    pub type_: Box<Type>,
}



#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Identifier {
    pub span: Span,
    pub identifier: String,
}


#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Pragma {
    Any {
        span: Span,
        edge_name: EdgeName,
    },
    Disjoint {
        span: Span,
        edge_name: EdgeName,
    },
    MultiAny {
        span: Span,
        edge_name: EdgeName,
    },
    Unique {
        span: Span,
        edge_name: EdgeName,
    },
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Game {
    pub constants: Vec<Constant>,
    pub edges: Vec<Edge>,
    pub pragmas: Vec<Pragma>,
    pub typedefs: Vec<Typedef>,
    pub variables: Vec<Variable>,
}