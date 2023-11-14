use std::sync::Arc;

use crate::rg::{
    ast::*,
    parser::Span,
    position::{Span as Position, *},
};

impl From<(Span<'_>, (Identifier, Arc<Type>, Arc<Value>))> for Constant {
    fn from(
        (start, (identifier, type_, value)): (Span, (Identifier, Arc<Type>, Arc<Value>)),
    ) -> Self {
        let span = Position::from(start).with_end(value.end());
        Self::new(span, identifier, type_, value)
    }
}

impl From<Vec<EdgeNamePart>> for EdgeName {
    fn from(parts: Vec<EdgeNamePart>) -> Self {
        let (first, last) = (parts.first().unwrap(), parts.last().unwrap());
        let span = Position::new(first.start().clone(), last.end().clone());
        Self::new(span, parts)
    }
}

impl From<Identifier> for EdgeName {
    fn from(identifier: Identifier) -> Self {
        Self::from(vec![EdgeNamePart::from(identifier)])
    }
}

impl From<Identifier> for EdgeNamePart {
    fn from(identifier: Identifier) -> Self {
        Self::Literal { identifier }
    }
}

impl From<(Identifier, Arc<Type>)> for EdgeNamePart {
    fn from((identifier, type_): (Identifier, Arc<Type>)) -> Self {
        let span = Position::new(identifier.start(), type_.end());
        Self::Binding {
            span,
            identifier,
            type_,
        }
    }
}

impl From<(EdgeName, EdgeName, EdgeLabel)> for Edge {
    fn from((lhs, rhs, label): (EdgeName, EdgeName, EdgeLabel)) -> Self {
        let span = Position::new(lhs.start(), label.end());
        Self::new(span, lhs, rhs, label)
    }
}

impl From<Identifier> for EdgeLabel {
    fn from(symbol: Identifier) -> Self {
        EdgeLabel::Tag { symbol }
    }
}

impl From<(Arc<Expression>, Arc<Expression>)> for EdgeLabel {
    fn from((lhs, rhs): (Arc<Expression>, Arc<Expression>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl From<(Arc<Expression>, bool, Arc<Expression>)> for EdgeLabel {
    fn from((lhs, negated, rhs): (Arc<Expression>, bool, Arc<Expression>)) -> Self {
        Self::Comparison { lhs, rhs, negated }
    }
}

impl From<(Span<'_>, EdgeName, EdgeName)> for EdgeLabel {
    fn from((tag, lhs, rhs): (Span, EdgeName, EdgeName)) -> Self {
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

impl From<Span<'_>> for Identifier {
    fn from(value: Span) -> Self {
        Self::new(Position::from(&value), value.fragment().to_string())
    }
}

impl From<Identifier> for Type {
    fn from(identifier: Identifier) -> Self {
        Self::TypeReference { identifier }
    }
}

impl From<Vec<Identifier>> for Type {
    fn from(identifiers: Vec<Identifier>) -> Self {
        let (first, last) = (identifiers.first().unwrap(), identifiers.last().unwrap());
        let span = Position::new(first.start(), last.end());
        Self::Set { span, identifiers }
    }
}

impl From<(Span<'_>, Identifier, Arc<Type>)> for Typedef {
    fn from((start, identifier, type_): (Span, Identifier, Arc<Type>)) -> Self {
        let span = Position::from(start).with_end(type_.span().end);
        Self::new(span, identifier, type_)
    }
}

impl From<(Span<'_>, Vec<Option<ValueEntry>>, Span<'_>)> for Value {
    fn from((start, entries, end): (Span<'_>, Vec<Option<ValueEntry>>, Span<'_>)) -> Self {
        let start = Position::from(start);
        let end = Position::from(end);
        let span = Position::new(start.start, end.end);
        let entries = entries.into_iter().flatten().collect();
        Self::Map { span, entries }
    }
}

impl From<(Span<'_>, Span<'_>)> for Value {
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

impl From<Identifier> for Value {
    fn from(identifier: Identifier) -> Self {
        Self::Element { identifier }
    }
}

impl From<(Option<Identifier>, Option<Arc<Value>>)> for ValueEntry {
    fn from((identifier, value): (Option<Identifier>, Option<Arc<Value>>)) -> Self {
        match value {
            Some(value) => {
                let span = match &identifier {
                    Some(identifier) => {
                        Position::new(identifier.span().start, value.as_ref().span().end)
                    }
                    None => value.as_ref().span().clone(),
                };
                Self::new(span, identifier, value)
            }
            None => {
                let span = match &identifier {
                    Some(identifier) => identifier.span().clone(),
                    None => Position::none(),
                };
                Self::new(
                    span,
                    identifier,
                    Arc::new(Value::Element {
                        identifier: Identifier::none(Position::none()),
                    }),
                )
            }
        }
    }
}

impl From<(Span<'_>, Identifier, Arc<Type>, Arc<Value>)> for Variable {
    fn from((start, identifier, type_, value): (Span, Identifier, Arc<Type>, Arc<Value>)) -> Self {
        let span = Position::from(start).with_end(value.end());
        Self::new(span, value, identifier, type_)
    }
}
