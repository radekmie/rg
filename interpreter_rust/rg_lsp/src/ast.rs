use std::rc::Rc;

use crate::position::Span;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Constant<'a> {
    pub span: Span,
    pub identifier: Identifier<'a>,
    pub type_: Rc<Type<'a>>,
    pub value: Rc<Value<'a>>,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge<'a> {
    pub span: Span,
    pub label: EdgeLabel<'a>,
    pub lhs: EdgeName<'a>,
    pub rhs: EdgeName<'a>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeLabel<'a> {
    Assignment {
        lhs: Rc<Expression<'a>>,
        rhs: Rc<Expression<'a>>,
    },
    Comparison {
        lhs: Rc<Expression<'a>>,
        rhs: Rc<Expression<'a>>,
        negated: bool,
    },
    Reachability {
        span: Span,
        lhs: EdgeName<'a>,
        rhs: EdgeName<'a>,
        negated: bool,
    },
    Skip {
        span: Span
    },
    Tag {
        symbol: Identifier<'a>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EdgeName<'a> {
    pub span: Span,
    pub parts: Vec<EdgeNamePart<'a>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeNamePart<'a> {
    Binding {
        span: Span,
        identifier: Identifier<'a>,
        type_: Rc<Type<'a>>,
    },
    Literal {
        identifier: Identifier<'a>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Type<'a> {
    Arrow {
        lhs: Rc<Self>,
        rhs: Rc<Self>,
    },
    Set {
        span: Span,
        identifiers: Vec<Identifier<'a>>,
    },
    TypeReference {
        identifier: Identifier<'a>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Expression<'a> {
    Access {
        span: Span,
        lhs: Rc<Self>,
        rhs: Rc<Self>,
    },
    Cast {
        span: Span,
        lhs: Rc<Type<'a>>,
        rhs: Rc<Self>,
    },
    Reference {
        identifier: Identifier<'a>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Value<'a> {
    Element {
        identifier: Identifier<'a>,
    },
    Map {
        span: Span,
        entries: Vec<ValueEntry<'a>>,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValueEntry<'a> {
    pub span: Span,
    pub identifier: Option<Identifier<'a>>,
    pub value: Rc<Value<'a>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Variable<'a> {
    pub span: Span,
    pub default_value: Rc<Value<'a>>,
    pub identifier: Identifier<'a>,
    pub type_: Rc<Type<'a>>,
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Typedef<'a> {
    pub span: Span,
    pub identifier: Identifier<'a>,
    pub type_: Rc<Type<'a>>,
}



#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Identifier<'a> {
    pub span: Span,
    pub identifier: &'a str,
}


#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Pragma<'a> {
    Any {
        span: Span,
        edge_name: EdgeName<'a>,
    },
    Disjoint {
        span: Span,
        edge_name: EdgeName<'a>,
    },
    MultiAny {
        span: Span,
        edge_name: EdgeName<'a>,
    },
    Unique {
        span: Span,
        edge_name: EdgeName<'a>,
    },
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Game<'a> {
    pub constants: Vec<Constant<'a>>,
    pub edges: Vec<Edge<'a>>,
    pub pragmas: Vec<Pragma<'a>>,
    pub typedefs: Vec<Typedef<'a>>,
    pub variables: Vec<Variable<'a>>,
}