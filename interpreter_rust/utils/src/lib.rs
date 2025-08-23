pub mod cartesian;
pub mod display;
pub mod interner;
pub mod parser;
pub mod position;

use parser::Input;
use position::{Positioned, Span};
use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
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

    pub fn is_numeric(&self) -> bool {
        self.identifier.chars().all(char::is_numeric)
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
pub struct ParserError {
    pub span: Span,
    pub message: String,
}

impl ParserError {
    pub fn new(span: Span, message: String) -> Self {
        Self { span, message }
    }

    pub fn new_unknown_identifier(identifier: &Identifier) -> Self {
        Self::new(
            identifier.span,
            format!("Unknown identifier: {}", identifier.identifier),
        )
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.span, self.message)
    }
}
