use crate::ast::Identifier;
use crate::position::{Positioned, Span};
use std::fmt::Display;

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

    pub fn symbol_table_error(identifier: &Identifier) -> Self {
        Self {
            span: identifier.span(),
            message: format!("Unknown identifier: {}", identifier.identifier),
            kind: ErrorKind::UnknownIdentifier,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({},{}): {}",
            self.span.start, self.span.end, self.message
        )
    }
}
