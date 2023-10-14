use std::rc::Rc;

use crate::position::Span;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Constant {
    pub span: Span,
    pub identifier: Identifier,
    pub type_: Rc<Type>,
    pub value: Rc<Value>,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge {
    pub span: Span,
    pub label: EdgeLabel,
    pub lhs: EdgeName,
    pub rhs: EdgeName,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeLabel {
    Assignment {
        lhs: Rc<Expression>,
        rhs: Rc<Expression>,
    },
    Comparison {
        lhs: Rc<Expression>,
        rhs: Rc<Expression>,
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
        type_: Rc<Type>,
    },
    Literal {
        identifier: Identifier,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Type {
    Arrow {
        lhs: Rc<Self>,
        rhs: Rc<Self>,
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
        lhs: Rc<Self>,
        rhs: Rc<Self>,
    },
    Cast {
        span: Span,
        lhs: Rc<Type>,
        rhs: Rc<Self>,
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
    pub value: Rc<Value>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Variable {
    pub span: Span,
    pub default_value: Rc<Value>,
    pub identifier: Identifier,
    pub type_: Rc<Type>,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Typedef {
    pub span: Span,
    pub identifier: Identifier,
    pub type_: Rc<Type>,
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