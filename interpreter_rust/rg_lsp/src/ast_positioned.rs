use crate::ast::*;
use crate::position::*;

impl<'a> Positioned for Constant<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Positioned for Edge<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Positioned for EdgeLabel<'a> {
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
impl<'a> Positioned for EdgeName<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Positioned for EdgeNamePart<'a> {
    fn span(&self) -> Span {
        match &self {
            EdgeNamePart::Binding { span, .. } => *span,
            EdgeNamePart::Literal { identifier } => identifier.span(),
        }
    }
}

impl<'a> Positioned for Type<'a> {
    fn span(&self) -> Span {
        match &self {
            Type::Arrow { lhs, rhs } => lhs.span().with_end(rhs.span().end),
            Type::Set { span, .. } => *span,
            Type::TypeReference { identifier } => identifier.span(),
        }
    }
}

impl<'a> Positioned for Expression<'a> {
    fn span(&self) -> Span {
        match &self {
            Expression::Access { span, .. } => *span,
            Expression::Cast { span, .. } => *span,
            Expression::Reference { identifier } => identifier.span(),
        }
    }
}

impl<'a> Positioned for Value<'a> {
    fn span(&self) -> Span {
        match &self {
            Value::Element { identifier } => identifier.span(),
            Value::Map { span, .. } => *span,
        }
    }
}

impl<'a> Positioned for ValueEntry<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Positioned for Variable<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Positioned for Typedef<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Positioned for Identifier<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> Positioned for Pragma<'a> {
    fn span(&self) -> Span {
        match &self {
            Pragma::Any { span, .. } => *span,
            Pragma::MultiAny { span, .. } => *span,
            Pragma::Unique { span, .. } => *span,
            Pragma::Disjoint { span, .. } => *span,
        }
    }
}
