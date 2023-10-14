use crate::ast::*;
use crate::position::*;

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
        match &self {
            Pragma::Any { span, .. } => *span,
            Pragma::MultiAny { span, .. } => *span,
            Pragma::Unique { span, .. } => *span,
            Pragma::Disjoint { span, .. } => *span,
        }
    }
}
