use tower_lsp::lsp_types::SymbolKind;
use tower_lsp::lsp_types::{self as l, Location, Url};

use crate::rg::position::*;
use crate::rg::symbol::Flag;

pub fn flag_to_kind(flag: &Flag) -> SymbolKind {
    match flag {
        Flag::Constant => SymbolKind::CONSTANT,
        Flag::Variable => SymbolKind::VARIABLE,
        Flag::Edge => SymbolKind::METHOD,
        Flag::Param => SymbolKind::PROPERTY,
        Flag::Type => SymbolKind::CLASS,
        Flag::Member => SymbolKind::FIELD,
    }
}

impl From<Span> for l::Range {
    fn from(span: Span) -> Self {
        l::Range {
            start: span.start.into(),
            end: span.end.into(),
        }
    }
}

impl From<Position> for l::Position {
    fn from(position: Position) -> Self {
        if position.is_none() {
            panic!("Called toLsp on a none position")
        } else {
            l::Position {
                line: (position.line - 1) as u32,
                character: (position.column - 1) as u32,
            }
        }
    }
}

impl From<Position> for l::Range {
    fn from(position: Position) -> Self {
        if position.is_none() {
            panic!("Called toLsp on a none position")
        } else {
            l::Range {
                start: position.clone().into(),
                end: position.into(),
            }
        }
    }
}

impl From<l::Position> for Position {
    fn from(position: l::Position) -> Self {
        Position {
            line: (position.line + 1) as usize,
            column: (position.character + 1) as usize,
        }
    }
}

impl From<l::Range> for Span {
    fn from(range: l::Range) -> Self {
        Self {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

pub fn to_location(url: &Url, span: Span) -> Location {
    Location {
        uri: url.clone(),
        range: span.clone().into(),
    }
}
