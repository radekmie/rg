use crate::ast::*;
use crate::parser::Span;
use crate::position::{Span as Position, *};
use nom_locate::LocatedSpan;
use std::rc::Rc;

impl From<(Span<'_>, (Identifier, Rc<Type>, Rc<Value>), Span<'_>)> for Constant {
    fn from(
        (start, (identifier, type_, value), end): (Span, (Identifier, Rc<Type>, Rc<Value>), Span),
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

impl From<Vec<EdgeNamePart>> for EdgeName {
    fn from(parts: Vec<EdgeNamePart>) -> Self {
        let (first, last) = (parts.first().unwrap(), parts.last().unwrap());
        let span = Position::new(first.start().clone(), last.end().clone());
        Self { span, parts }
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

impl From<(Span<'_>, (Identifier, Rc<Type>), Span<'_>)> for EdgeNamePart {
    fn from((start, (identifier, type_), end): (Span, (Identifier, Rc<Type>), Span)) -> Self {
        let span = Position::from((start, end));
        Self::Binding {
            span,
            identifier,
            type_,
        }
    }
}

impl From<(EdgeName, EdgeName, EdgeLabel, Span<'_>)> for Edge {
    fn from((lhs, rhs, label, end): (EdgeName, EdgeName, EdgeLabel, Span)) -> Self {
        let span = Position::from(end).with_start(lhs.start().clone());
        Self {
            span,
            label,
            lhs,
            rhs,
        }
    }
}

impl From<(Identifier)> for EdgeLabel {
    fn from(symbol: Identifier) -> Self {
        EdgeLabel::Tag { symbol }
    }
}

impl From<(Rc<Expression>, Rc<Expression>)> for EdgeLabel {
    fn from((lhs, rhs): (Rc<Expression>, Rc<Expression>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl From<(Rc<Expression>, bool, Rc<Expression>)> for EdgeLabel {
    fn from((lhs, negated, rhs): (Rc<Expression>, bool, Rc<Expression>)) -> Self {
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
        Identifier {
            span: Position::from(&value),
            identifier: value.fragment().to_string(),
        }
    }
}

impl From<Identifier> for Type {
    fn from(identifier: Identifier) -> Self {
        Self::TypeReference { identifier }
    }
}

impl From<(Span<'_>, Vec<Identifier>, Span<'_>)> for Type {
    fn from((start, identifiers, end): (Span, Vec<Identifier>, Span)) -> Self {
        let span = Position::from((start, end));
        Self::Set { span, identifiers }
    }
}

impl From<(Span<'_>, (Identifier, Rc<Type>), Span<'_>)> for Typedef {
    fn from((start, (identifier, type_), end): (Span, (Identifier, Rc<Type>), Span)) -> Self {
        let span = Position::from((start, end));
        Self {
            span,
            identifier,
            type_,
        }
    }
}

impl From<(Span<'_>, Vec<ValueEntry>, Span<'_>)> for Value {
    fn from((start, entries, end): (Span, Vec<ValueEntry>, Span)) -> Self {
        let span = Position::from((start, end));
        Self::Map { span, entries }
    }
}

impl From<(Identifier)> for Value {
    fn from(identifier: Identifier) -> Self {
        Self::Element { identifier }
    }
}

impl From<(Option<Identifier>, Rc<Value>)> for ValueEntry {
    fn from((identifier, value): (Option<Identifier>, Rc<Value>)) -> Self {
        let span = match &identifier {
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

impl From<(Span<'_>, (Identifier, Rc<Type>, Rc<Value>), Span<'_>)> for Variable {
    fn from(
        (start, (identifier, type_, value), end): (Span, (Identifier, Rc<Type>, Rc<Value>), Span),
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
