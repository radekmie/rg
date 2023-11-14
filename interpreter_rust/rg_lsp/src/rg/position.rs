use nom_locate::LocatedSpan;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl<'a, T> From<&LocatedSpan<&'a str, T>> for Span {
    fn from(span: &LocatedSpan<&'a str, T>) -> Self {
        Self {
            start: Position {
                line: span.location_line() as usize,
                column: span.get_column() as usize,
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
                column: span.get_column() as usize,
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
                column: start.get_column() as usize,
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
                column: start.get_column() as usize,
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
        self.start <= *other && *other <= self.end
    }

    pub fn encloses_span(&self, other: &Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }

    pub fn focus_start(&self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }

    pub fn focus_end(&self) -> Self {
        Self {
            start: self.end,
            end: self.end,
        }
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
