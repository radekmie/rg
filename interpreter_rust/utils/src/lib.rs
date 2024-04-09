pub mod interner;
pub mod parser;
pub mod position;

use parser::Input;
use position::{Positioned, Span};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Default)]
pub struct Identifier {
    pub span: Span,
    pub identifier: String,
}

impl Identifier {
    pub fn new(span: Span, identifier: String) -> Self {
        Self { span, identifier }
    }

    pub fn none(span: Span) -> Self {
        Self {
            span,
            identifier: String::from("<none>"),
        }
    }

    pub fn is_none(&self) -> bool {
        self.identifier == "<none>"
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.identifier)
    }
}

impl From<Input<'_>> for Identifier {
    fn from(value: Input) -> Self {
        Self::new(Span::from(&value), (*value.fragment()).to_string())
    }
}

impl Positioned for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub span: Span,
    pub message: String,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    ParseError,
    UnknownIdentifier,
}

impl Error {
    pub fn parser_error(span: Span, message: String) -> Self {
        Self {
            span,
            message,
            kind: ErrorKind::ParseError,
        }
    }

    pub fn symbol_table_error(identifier: &str, span: &Span) -> Self {
        Self {
            span: *span,
            message: format!("Unknown identifier: {}", identifier),
            kind: ErrorKind::UnknownIdentifier,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.span, self.message)
    }
}
