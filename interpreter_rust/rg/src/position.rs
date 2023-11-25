use map_id::MapId;
use nom_locate::LocatedSpan;

use crate::ast::*;

#[derive(Copy, Clone, Debug, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

// Fake implementations to ignore `Span` in AST transformations. In Rg-lsp use `equals_span` and `is_after` instead.
impl PartialEq for Span {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(_other))
    }
}

impl Eq for Span {}
impl Ord for Span {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl<OldId, NewId> MapId<Span, OldId, NewId> for Span {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Span {
        *self
    }
}

impl<'a, T> From<&LocatedSpan<&'a str, T>> for Span {
    fn from(span: &LocatedSpan<&'a str, T>) -> Self {
        Self {
            start: Position {
                line: span.location_line() as usize,
                column: span.get_column(),
            },
            end: Position {
                line: span.location_line() as usize,
                column: span.get_column() + span.fragment().len(),
            },
        }
    }
}

impl<'a, T> From<LocatedSpan<&'a str, T>> for Span {
    fn from(span: LocatedSpan<&'a str, T>) -> Self {
        Self {
            start: Position {
                line: span.location_line() as usize,
                column: span.get_column(),
            },
            end: Position {
                line: span.location_line() as usize,
                column: span.get_column() + span.fragment().len(),
            },
        }
    }
}

impl<'a, T> From<(LocatedSpan<&'a str, T>, LocatedSpan<&'a str, T>)> for Span {
    fn from((start, end): (LocatedSpan<&'a str, T>, LocatedSpan<&'a str, T>)) -> Self {
        Self {
            start: Position {
                line: start.location_line() as usize,
                column: start.get_column(),
            },
            end: Position {
                line: end.location_line() as usize,
                column: end.get_column() + end.fragment().len(),
            },
        }
    }
}

impl<'a, T> From<(&LocatedSpan<&'a str, T>, &LocatedSpan<&'a str, T>)> for Span {
    fn from((start, end): (&LocatedSpan<&'a str, T>, &LocatedSpan<&'a str, T>)) -> Self {
        Self {
            start: Position {
                line: start.location_line() as usize,
                column: start.get_column(),
            },
            end: Position {
                line: end.location_line() as usize,
                column: end.get_column() + end.fragment().len(),
            },
        }
    }
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn with_start(self, start: Position) -> Self {
        Self {
            start,
            end: self.end,
        }
    }

    pub fn with_end(self, end: Position) -> Self {
        Self {
            start: self.start,
            end,
        }
    }

    pub fn none() -> Self {
        Self {
            start: Position::none(),
            end: Position::none(),
        }
    }

    pub fn is_none(&self) -> bool {
        self.start.is_none() || self.end.is_none()
    }

    pub fn encloses_pos(&self, other: &Position) -> bool {
        self.start.line <= other.line
            && other.line <= self.end.line
            && (self.start.line < other.line || self.start.column <= other.column)
            && (other.line < self.end.line || other.column <= self.end.column)
    }

    pub fn encloses_span(&self, other: &Span) -> bool {
        self.encloses_pos(&other.start) && self.encloses_pos(&other.end)
    }

    pub fn focus_start(self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }

    pub fn focus_end(self) -> Self {
        Self {
            start: self.end,
            end: self.end,
        }
    }

    pub fn equal_span(&self, other: &Span) -> bool {
        self.start.equal_pos(&other.start) && self.end.equal_pos(&other.end)
    }

    pub fn is_after(&self, other: &Span) -> bool {
        self.start.is_after(&other.start)
    }
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn none() -> Self {
        Self { line: 0, column: 0 }
    }

    pub fn is_none(&self) -> bool {
        self.line == 0 && self.column == 0
    }

    pub fn equal_pos(&self, other: &Position) -> bool {
        self.line == other.line && self.column == other.column
    }

    pub fn is_after(&self, other: &Position) -> bool {
        self.line > other.line || (self.line == other.line && self.column > other.column)
    }
}

pub trait Positioned {
    fn span(&self) -> Span;

    fn start(&self) -> Position {
        self.span().start
    }

    fn end(&self) -> Position {
        self.span().end
    }
}

impl<Id> Positioned for Constant<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id> Positioned for Edge<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id: Positioned> Positioned for EdgeLabel<Id> {
    fn span(&self) -> Span {
        match self {
            EdgeLabel::Assignment { lhs, rhs } | EdgeLabel::Comparison { lhs, rhs, .. } => {
                lhs.span().with_end(rhs.span().end)
            }
            EdgeLabel::Reachability { span, .. } | EdgeLabel::Skip { span } => *span,
            EdgeLabel::Tag { symbol } => symbol.span(),
        }
    }
}
impl<Id> Positioned for EdgeName<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id: Positioned> Positioned for EdgeNamePart<Id> {
    fn span(&self) -> Span {
        match &self {
            EdgeNamePart::Binding { span, .. } => *span,
            EdgeNamePart::Literal { identifier } => identifier.span(),
        }
    }
}

impl<Id: Positioned> Positioned for Type<Id> {
    fn span(&self) -> Span {
        match &self {
            Type::Arrow { lhs, rhs } => lhs.span().with_end(rhs.span().end),
            Type::Set { span, .. } => *span,
            Type::TypeReference { identifier } => identifier.span(),
        }
    }
}

impl<Id: Positioned> Positioned for Expression<Id> {
    fn span(&self) -> Span {
        match &self {
            Expression::Access { span, .. } | Expression::Cast { span, .. } => *span,
            Expression::Reference { identifier } => identifier.span(),
        }
    }
}

impl<Id: Positioned> Positioned for Value<Id> {
    fn span(&self) -> Span {
        match &self {
            Value::Element { identifier } => identifier.span(),
            Value::Map { span, .. } => *span,
        }
    }
}

impl<Id> Positioned for ValueEntry<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id> Positioned for Variable<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id> Positioned for Typedef<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl Positioned for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id> Positioned for Pragma<Id> {
    fn span(&self) -> Span {
        match self {
            Self::Any { span, .. }
            | Self::Disjoint { span, .. }
            | Self::MultiAny { span, .. }
            | Self::Unique { span, .. } => *span,
        }
    }
}
