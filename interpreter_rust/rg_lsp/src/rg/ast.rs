use std::sync::Arc;

use crate::rg::position::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Constant {
    span: Span,
    pub identifier: Identifier,
    pub type_: Arc<Type>,
    pub value: Arc<Value>,
}

impl Constant {
    pub fn new(span: Span, identifier: Identifier, type_: Arc<Type>, value: Arc<Value>) -> Self {
        Self {
            span,
            identifier,
            type_,
            value,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge {
    span: Span,
    pub lhs: EdgeName,
    pub rhs: EdgeName,
    pub label: EdgeLabel,
}

impl Edge {
    pub fn new(span: Span, lhs: EdgeName, rhs: EdgeName, label: EdgeLabel) -> Self {
        Self {
            span,
            lhs,
            rhs,
            label,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeLabel {
    Assignment {
        lhs: Arc<Expression>,
        rhs: Arc<Expression>,
    },
    Comparison {
        lhs: Arc<Expression>,
        rhs: Arc<Expression>,
        negated: bool,
    },
    Reachability {
        span: Span,
        lhs: EdgeName,
        rhs: EdgeName,
        negated: bool,
    },
    Skip {
        span: Span,
    },
    Tag {
        symbol: Identifier,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EdgeName {
    span: Span,
    pub parts: Vec<EdgeNamePart>,
}

impl EdgeName {
    pub fn new(span: Span, parts: Vec<EdgeNamePart>) -> Self {
        Self { span, parts }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EdgeNamePart {
    Binding {
        span: Span,
        identifier: Identifier,
        type_: Arc<Type>,
    },
    Literal {
        identifier: Identifier,
    },
}

impl EdgeNamePart {
    pub fn identifier(&self) -> &Identifier {
        match self {
            EdgeNamePart::Binding { identifier, .. } => identifier,
            EdgeNamePart::Literal { identifier } => identifier,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Type {
    Arrow {
        lhs: Arc<Self>,
        rhs: Arc<Self>,
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
        lhs: Arc<Self>,
        rhs: Arc<Self>,
    },
    Cast {
        span: Span,
        lhs: Arc<Type>,
        rhs: Arc<Self>,
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
    span: Span,
    pub identifier: Option<Identifier>,
    pub value: Arc<Value>,
}

impl ValueEntry {
    pub fn new(span: Span, identifier: Option<Identifier>, value: Arc<Value>) -> Self {
        Self {
            span,
            identifier,
            value,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Variable {
    span: Span,
    pub default_value: Arc<Value>,
    pub identifier: Identifier,
    pub type_: Arc<Type>,
}

impl Variable {
    pub fn new(
        span: Span,
        default_value: Arc<Value>,
        identifier: Identifier,
        type_: Arc<Type>,
    ) -> Self {
        Self {
            span,
            default_value,
            identifier,
            type_,
        }
    }
}
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Typedef {
    span: Span,
    pub identifier: Identifier,
    pub type_: Arc<Type>,
}

impl Typedef {
    pub fn new(span: Span, identifier: Identifier, type_: Arc<Type>) -> Self {
        Self {
            span,
            identifier,
            type_,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Identifier {
    span: Span,
    pub identifier: String,
}

impl Identifier {
    pub fn new(span: Span, identifier: String) -> Self {
        Self { span, identifier }
    }

    pub fn none(span: Span) -> Self {
        Self {
            span,
            identifier: String::from("<none>"),
        }
    }

    pub fn is_none(&self) -> bool {
        self.identifier == "<none>"
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Pragma {
    span: Span,
    pub kind: PragmaKind,
    pub edge_name: EdgeName,
}

impl Pragma {
    pub fn new(span: Span, kind: PragmaKind, edge_name: EdgeName) -> Self {
        Self {
            span,
            kind,
            edge_name,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PragmaKind {
    Any,
    Disjoint,
    MultiAny,
    Unique,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Stat {
    Constant(Constant),
    Edge(Edge),
    Pragma(Pragma),
    Typedef(Typedef),
    Variable(Variable),
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Game {
    pub stats: Vec<Stat>,
}

impl Positioned for Constant {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for Edge {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for EdgeLabel {
    fn span(&self) -> Span {
        match self {
            EdgeLabel::Assignment { lhs, rhs } => lhs.span().with_end(rhs.span().end),
            EdgeLabel::Comparison { lhs, rhs, .. } => lhs.span().with_end(rhs.span().end),
            EdgeLabel::Reachability { span, .. } => *span,
            EdgeLabel::Tag { symbol } => symbol.span(),
            EdgeLabel::Skip { span } => *span,
        }
    }
}
impl Positioned for EdgeName {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for EdgeNamePart {
    fn span(&self) -> Span {
        match &self {
            EdgeNamePart::Binding { span, .. } => *span,
            EdgeNamePart::Literal { identifier } => identifier.span(),
        }
    }
}

impl Positioned for Type {
    fn span(&self) -> Span {
        match &self {
            Type::Arrow { lhs, rhs } => lhs.span().with_end(rhs.span().end),
            Type::Set { span, .. } => *span,
            Type::TypeReference { identifier } => identifier.span(),
        }
    }
}

impl Positioned for Expression {
    fn span(&self) -> Span {
        match &self {
            Expression::Access { span, .. } => *span,
            Expression::Cast { span, .. } => *span,
            Expression::Reference { identifier } => identifier.span(),
        }
    }
}

impl Positioned for Value {
    fn span(&self) -> Span {
        match &self {
            Value::Element { identifier } => identifier.span(),
            Value::Map { span, .. } => *span,
        }
    }
}

impl Positioned for ValueEntry {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for Variable {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for Typedef {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for Pragma {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for Stat {
    fn span(&self) -> Span {
        match self {
            Stat::Constant(constant) => constant.span(),
            Stat::Edge(edge) => edge.span(),
            Stat::Pragma(pragma) => pragma.span(),
            Stat::Typedef(typedef) => typedef.span(),
            Stat::Variable(variable) => variable.span(),
        }
    }
}
