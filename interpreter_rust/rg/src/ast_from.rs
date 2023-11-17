use crate::{
    ast::*,
    parser::Span,
    position::{Span as Position, *},
};
use std::sync::Arc;

impl<Id: Positioned> From<(Span<'_>, (Id, Arc<Type<Id>>, Arc<Value<Id>>))> for Constant<Id> {
    fn from(
        (start, (identifier, type_, value)): (Span, (Id, Arc<Type<Id>>, Arc<Value<Id>>)),
    ) -> Self {
        let span = Position::from(start).with_end(value.end());
        Self {
            span,
            identifier,
            type_,
            value,
        }
    }
}

impl<Id: Positioned> From<(EdgeName<Id>, EdgeName<Id>, EdgeLabel<Id>)> for Edge<Id> {
    fn from((lhs, rhs, label): (EdgeName<Id>, EdgeName<Id>, EdgeLabel<Id>)) -> Self {
        let span = Position::new(lhs.start(), label.end());
        Self {
            span,
            label,
            lhs,
            rhs,
        }
    }
}

impl<Id> From<Id> for EdgeLabel<Id> {
    fn from(symbol: Id) -> Self {
        Self::Tag { symbol }
    }
}

impl<Id> From<(Arc<Expression<Id>>, Arc<Expression<Id>>)> for EdgeLabel<Id> {
    fn from((lhs, rhs): (Arc<Expression<Id>>, Arc<Expression<Id>>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl<Id> From<(Arc<Expression<Id>>, bool, Arc<Expression<Id>>)> for EdgeLabel<Id> {
    fn from((lhs, negated, rhs): (Arc<Expression<Id>>, bool, Arc<Expression<Id>>)) -> Self {
        Self::Comparison { lhs, rhs, negated }
    }
}

impl<Id: Positioned> From<(Span<'_>, EdgeName<Id>, EdgeName<Id>)> for EdgeLabel<Id> {
    fn from((tag, lhs, rhs): (Span, EdgeName<Id>, EdgeName<Id>)) -> Self {
        let negated = *tag.fragment() == "!";
        let span = Position::from(tag).with_end(rhs.span().end);
        Self::Reachability {
            span,
            lhs,
            rhs,
            negated,
        }
    }
}

impl<Id: Positioned> From<Vec<EdgeNamePart<Id>>> for EdgeName<Id> {
    fn from(parts: Vec<EdgeNamePart<Id>>) -> Self {
        let (first, last) = (parts.first().unwrap(), parts.last().unwrap());
        let span = Position::new(first.start().clone(), last.end().clone());
        Self { span, parts }
    }
}

impl<Id: Positioned> From<Id> for EdgeName<Id> {
    fn from(identifier: Id) -> Self {
        Self::from(vec![EdgeNamePart::from(identifier)])
    }
}

impl<Id: Positioned> From<(Span<'_>, Id, Arc<Type<Id>>, Span<'_>)> for EdgeNamePart<Id> {
    fn from((start, identifier, type_, end): (Span, Id, Arc<Type<Id>>, Span)) -> Self {
        let span = Position::from((start, end));
        Self::Binding {
            span,
            identifier,
            type_,
        }
    }
}

impl<Id> From<Id> for EdgeNamePart<Id> {
    fn from(identifier: Id) -> Self {
        Self::Literal { identifier }
    }
}

impl From<Span<'_>> for Identifier {
    fn from(value: Span) -> Self {
        Self::new(Position::from(&value), value.fragment().to_string())
    }
}

impl<Id: Positioned> From<(Span<'_>, Vec<Id>, Span<'_>)> for Type<Id> {
    fn from((start, identifiers, end): (Span, Vec<Id>, Span)) -> Self {
        let span = Position::from((start, end));
        Self::Set { span, identifiers }
    }
}

impl<Id> From<Id> for Type<Id> {
    fn from(identifier: Id) -> Self {
        Self::TypeReference { identifier }
    }
}

impl<Id: Positioned> From<(Span<'_>, Id, Arc<Type<Id>>)> for Typedef<Id> {
    fn from((start, identifier, type_): (Span, Id, Arc<Type<Id>>)) -> Self {
        let span = Position::from(start).with_end(type_.span().end);
        Self {
            span,
            identifier,
            type_,
        }
    }
}

impl From<(Span<'_>, Span<'_>)> for Value<Identifier> {
    fn from((start, end): (Span<'_>, Span<'_>)) -> Self {
        let start = Position::from(start);
        let end = Position::from(end);
        let span = Position::new(start.start, end.end);
        Self::Map {
            span,
            entries: vec![],
        }
    }
}

impl<Id> From<Id> for Value<Id> {
    fn from(identifier: Id) -> Self {
        Self::Element { identifier }
    }
}

impl<Id> From<(Span<'_>, Vec<Option<ValueEntry<Id>>>, Span<'_>)> for Value<Id> {
    fn from((start, entries, end): (Span<'_>, Vec<Option<ValueEntry<Id>>>, Span<'_>)) -> Self {
        let start = Position::from(start);
        let end = Position::from(end);
        let span = Position::new(start.start, end.end);
        let entries = entries.into_iter().flatten().collect();
        Self::Map { span, entries }
    }
}

impl<Id> From<(Position, Option<Id>, Arc<Value<Id>>)> for ValueEntry<Id> {
    fn from((span, identifier, value): (Position, Option<Id>, Arc<Value<Id>>)) -> Self {
        Self {
            span,
            identifier,
            value,
        }
    }
}

impl<Id: Positioned> From<(Span<'_>, Id, Arc<Type<Id>>, Arc<Value<Id>>)> for Variable<Id> {
    fn from(
        (start, identifier, type_, default_value): (Span, Id, Arc<Type<Id>>, Arc<Value<Id>>),
    ) -> Self {
        let span = Position::from(start).with_end(default_value.end());
        Self {
            span,
            default_value,
            identifier,
            type_,
        }
    }
}
