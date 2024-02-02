use crate::ast::{
    Constant, Edge, EdgeLabel, EdgeName, EdgeNamePart, Expression, Identifier, Pragma, Type,
    Typedef, Value, ValueEntry, Variable,
};
use map_id::MapId;
use nom_locate::LocatedSpan;
use std::cmp::Ordering;

#[derive(Copy, Clone, Debug, Default, PartialOrd, Ord, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn is_none(&self) -> bool {
        self.line == 0 && self.column == 0
    }

    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn none() -> Self {
        Self { line: 0, column: 0 }
    }

    pub fn with_end(self, end: Self) -> Span {
        Span::new(self, end)
    }

    pub fn with_start(self, start: Self) -> Span {
        Span::new(start, self)
    }
}

impl<'a, T> From<&LocatedSpan<&'a str, T>> for Position {
    fn from(span: &LocatedSpan<&'a str, T>) -> Self {
        Self {
            line: span.location_line() as usize,
            column: span.get_column(),
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn at<T>(input: &LocatedSpan<&str, T>) -> Self {
        let start = Position {
            line: input.location_line() as usize,
            column: input.get_column(),
        };
        Self::new(start, start)
    }

    pub fn encloses_position(&self, other: &Position) -> bool {
        self.start.line <= other.line
            && other.line <= self.end.line
            && (self.start.line < other.line || self.start.column <= other.column)
            && (other.line < self.end.line || other.column <= self.end.column)
    }

    pub fn encloses_span(&self, other: &Self) -> bool {
        self.encloses_position(&other.start) && self.encloses_position(&other.end)
    }

    pub fn equal_span(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }

    pub fn focus_end(self) -> Self {
        Self {
            start: self.end,
            end: self.end,
        }
    }

    pub fn focus_start(self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }

    pub fn is_none(&self) -> bool {
        self.start.is_none() || self.end.is_none()
    }

    pub fn line_at<'a>(&self, text: &'a str) -> Option<&'a str> {
        if self.start.line == self.end.line {
            let line = text.lines().nth(self.start.line - 1)?;
            Some(&line[(self.start.column - 1)..(self.end.column - 1)])
        } else {
            None
        }
    }

    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn none() -> Self {
        Self {
            start: Position::none(),
            end: Position::none(),
        }
    }

    pub fn with_end(self, end: Position) -> Self {
        Self {
            start: self.start,
            end,
        }
    }

    pub fn with_start(self, start: Position) -> Self {
        Self {
            start,
            end: self.end,
        }
    }
}

// Fake implementations to ignore `Span` in AST transformations.
impl Eq for Span {}

impl Ord for Span {
    fn cmp(&self, _: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl PartialEq for Span {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for Span {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

impl<'a, T> From<&LocatedSpan<&'a str, T>> for Span {
    fn from(span: &LocatedSpan<&'a str, T>) -> Self {
        Self {
            start: span.into(),
            end: Position {
                column: span.get_column() + span.fragment().len(),
                line: span.location_line() as usize,
            },
        }
    }
}

impl<'a, T> From<(&LocatedSpan<&'a str, T>, &LocatedSpan<&'a str, T>)> for Span {
    fn from((start, end): (&LocatedSpan<&'a str, T>, &LocatedSpan<&'a str, T>)) -> Self {
        Self {
            start: Position {
                column: start.get_column(),
                line: start.location_line() as usize,
            },
            end: Position {
                column: end.get_column() + end.fragment().len(),
                line: end.location_line() as usize,
            },
        }
    }
}

pub trait Positioned {
    fn end(&self) -> Position {
        self.span().end
    }

    fn span(&self) -> Span;

    fn start(&self) -> Position {
        self.span().start
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
            Self::Assignment { lhs, rhs } | Self::Comparison { lhs, rhs, .. } => {
                lhs.span().with_end(rhs.span().end)
            }
            Self::Reachability { span, .. } | Self::Skip { span } => *span,
            Self::Tag { symbol } => symbol.span(),
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
            Self::Binding { span, .. } => *span,
            Self::Literal { identifier } => identifier.span(),
        }
    }
}

impl<Id: Positioned> Positioned for Expression<Id> {
    fn span(&self) -> Span {
        match &self {
            Self::Access { span, .. } | Self::Cast { span, .. } => *span,
            Self::Reference { identifier } => identifier.span(),
        }
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
            Self::Disjoint { span, .. }
            | Self::DisjointExhaustive { span, .. }
            | Self::Repeat { span, .. }
            | Self::SimpleApply { span, .. }
            | Self::TagIndex { span, .. }
            | Self::TagMaxIndex { span, .. }
            | Self::Unique { span, .. } => *span,
        }
    }
}

impl<Id: Positioned> Positioned for Type<Id> {
    fn span(&self) -> Span {
        match &self {
            Self::Arrow { lhs, rhs } => lhs.span().with_end(rhs.span().end),
            Self::Set { span, .. } => *span,
            Self::TypeReference { identifier } => identifier.span(),
        }
    }
}

impl<Id> Positioned for Typedef<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id: Positioned> Positioned for Value<Id> {
    fn span(&self) -> Span {
        match &self {
            Self::Element { identifier } => identifier.span(),
            Self::Map { span, .. } => *span,
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
