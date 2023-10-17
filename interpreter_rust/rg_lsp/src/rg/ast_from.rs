use crate::rg::{
    ast::*,
    parser::Span,
    position::{Span as Position, *},
};

impl From<(Span<'_>, (Identifier, Box<Type>, Box<Value>))> for Constant {
    fn from(
        (start, (identifier, type_, value)): (Span, (Identifier, Box<Type>, Box<Value>)),
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

impl From<(Identifier, Box<Type>)> for EdgeNamePart {
    fn from((identifier, type_): (Identifier, Box<Type>)) -> Self {
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
        let span = Position::new(lhs.start(), rhs.end());
        Self::new(span, lhs, rhs, label)
    }
}

impl From<Identifier> for EdgeLabel {
    fn from(symbol: Identifier) -> Self {
        EdgeLabel::Tag { symbol }
    }
}

impl From<(Box<Expression>, Box<Expression>)> for EdgeLabel {
    fn from((lhs, rhs): (Box<Expression>, Box<Expression>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl From<(Box<Expression>, bool, Box<Expression>)> for EdgeLabel {
    fn from((lhs, negated, rhs): (Box<Expression>, bool, Box<Expression>)) -> Self {
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

impl From<Option<Vec<Identifier>>> for Type {
    fn from(identifiers: Option<Vec<Identifier>>) -> Self {
        match identifiers {
            Some(identifiers) => {
                let (first, last) = (identifiers.first().unwrap(), identifiers.last().unwrap());
                let span = Position::new(first.start(), last.end());
                Self::Set { span, identifiers }
            }
            None => Self::Set {
                span: Position::none(),
                identifiers: vec![],
            },
        }
    }
}

impl From<(Span<'_>, Identifier, Box<Type>)> for Typedef {
    fn from((start, identifier, type_): (Span, Identifier, Box<Type>)) -> Self {
        let span = Position::from(start).with_end(type_.span().end);
        Self::new(span, identifier, type_)
    }
}

impl From<Option<Vec<ValueEntry>>> for Value {
    fn from(entries: Option<Vec<ValueEntry>>) -> Self {
        match entries {
            Some(entries) => {
                let (first, last) = (entries.first().unwrap(), entries.last().unwrap());
                let span = Position::new(first.start(), last.end());
                Self::Map { span, entries }
            }
            None => Self::Map {
                span: Position::none(),
                entries: vec![],
            },
        }
    }
}

impl From<Identifier> for Value {
    fn from(identifier: Identifier) -> Self {
        Self::Element { identifier }
    }
}

impl From<(Option<Identifier>, Box<Value>)> for ValueEntry {
    fn from((identifier, value): (Option<Identifier>, Box<Value>)) -> Self {
        let span = match &identifier {
            Some(identifier) => Position::new(identifier.span().start, value.as_ref().span().end),
            None => value.as_ref().span().clone(),
        };
        Self::new(span, identifier, value)
    }
}

impl From<(Span<'_>, Identifier, Box<Type>, Box<Value>)> for Variable {
    fn from((start, identifier, type_, value): (Span, Identifier, Box<Type>, Box<Value>)) -> Self {
        let span = Position::from(start).with_end(value.end());
        Self::new(span, value, identifier, type_)
    }
}
