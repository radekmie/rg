use crate::ast::*;
use crate::position::{Span as Position, *};
use nom_locate::LocatedSpan;
use std::rc::Rc;

type Span<'a> = LocatedSpan<&'a str>;

impl<'a>
    From<(
        Span<'a>,
        (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
        Span<'a>,
    )> for Constant<'a>
{
    fn from(
        (start, (identifier, type_, value), end): (
            Span<'a>,
            (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
            Span<'a>,
        ),
    ) -> Self {
        let span = Position::from((start, end));
        Self {
            span,
            identifier,
            type_,
            value,
        }
    }
}

impl<'a> From<Vec<EdgeNamePart<'a>>> for EdgeName<'a> {
    fn from(parts: Vec<EdgeNamePart<'a>>) -> Self {
        let (first, last) = (parts.first().unwrap(), parts.last().unwrap());
        let span = Position::new(first.start().clone(), last.end().clone());
        Self { span, parts }
    }
}

impl<'a> From<Identifier<'a>> for EdgeName<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::from(vec![EdgeNamePart::from(identifier)])
    }
}

impl<'a> From<Identifier<'a>> for EdgeNamePart<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::Literal { identifier }
    }
}

impl<'a> From<(Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>)> for EdgeNamePart<'a> {
    fn from(
        (start, (identifier, type_), end): (Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>),
    ) -> Self {
        let span = Position::from((start, end));
        Self::Binding {
            span,
            identifier,
            type_,
        }
    }
}

impl<'a> From<(EdgeName<'a>, EdgeName<'a>, EdgeLabel<'a>, Span<'a>)> for Edge<'a> {
    fn from((lhs, rhs, label, end): (EdgeName<'a>, EdgeName<'a>, EdgeLabel<'a>, Span<'a>)) -> Self {
        let span = Position::from((end, end)).with_start(lhs.start().clone());
        Self {
            span,
            label,
            lhs,
            rhs,
        }
    }
}

impl<'a> From<(Identifier<'a>)> for EdgeLabel<'a> {
    fn from(symbol: Identifier<'a>) -> Self {
        EdgeLabel::Tag { symbol }
    }
}

impl<'a> From<(Rc<Expression<'a>>, Rc<Expression<'a>>)> for EdgeLabel<'a> {
    fn from((lhs, rhs): (Rc<Expression<'a>>, Rc<Expression<'a>>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl<'a> From<(Rc<Expression<'a>>, bool, Rc<Expression<'a>>)> for EdgeLabel<'a> {
    fn from((lhs, negated, rhs): (Rc<Expression<'a>>, bool, Rc<Expression<'a>>)) -> Self {
        Self::Comparison { lhs, rhs, negated }
    }
}

impl<'a> From<(Span<'a>, EdgeName<'a>, EdgeName<'a>)> for EdgeLabel<'a> {
    fn from((tag, lhs, rhs): (Span<'a>, EdgeName<'a>, EdgeName<'a>)) -> Self {
        let negated = *tag.fragment() == "!";
        let span = Position::from((tag, tag)).with_end(rhs.span().end);
        Self::Reachability {
            span,
            lhs,
            rhs,
            negated,
        }
    }
}

impl<'a> From<Span<'a>> for Identifier<'a> {
    fn from(value: Span<'a>) -> Self {
        Identifier {
            span: Position::from(value),
            identifier: value.fragment(),
        }
    }
}

impl<'a> From<Identifier<'a>> for Type<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::TypeReference { identifier }
    }
}

impl<'a> From<(Span<'a>, Vec<Identifier<'a>>, Span<'a>)> for Type<'a> {
    fn from((start, identifiers, end): (Span<'a>, Vec<Identifier<'a>>, Span<'a>)) -> Self {
        let span = Position::from((start, end));
        Self::Set { span, identifiers }
    }
}

impl<'a> From<(Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>)> for Typedef<'a> {
    fn from(
        (start, (identifier, type_), end): (Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>),
    ) -> Self {
        let span = Position::from((start, end));
        Self {
            span,
            identifier,
            type_,
        }
    }
}

impl<'a> From<(Span<'a>, Vec<ValueEntry<'a>>, Span<'a>)> for Value<'a> {
    fn from((start, entries, end): (Span<'a>, Vec<ValueEntry<'a>>, Span<'a>)) -> Self {
        let span = Position::from((start, end));
        Self::Map { span, entries }
    }
}

impl<'a> From<(Identifier<'a>)> for Value<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::Element { identifier }
    }
}

impl<'a> From<(Option<Identifier<'a>>, Rc<Value<'a>>)> for ValueEntry<'a> {
    fn from((identifier, value): (Option<Identifier<'a>>, Rc<Value<'a>>)) -> Self {
        let span = match identifier {
            Some(identifier) => Position::new(identifier.span.start, value.as_ref().span().end),
            None => value.as_ref().span().clone(),
        };
        Self {
            span,
            identifier,
            value,
        }
    }
}

impl<'a>
    From<(
        Span<'a>,
        (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
        Span<'a>,
    )> for Variable<'a>
{
    fn from(
        (start, (identifier, type_, value), end): (
            Span<'a>,
            (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
            Span<'a>,
        ),
    ) -> Self {
        let span = Position::from((start, end));
        Self {
            span,
            identifier,
            default_value: value,
            type_,
        }
    }
}
