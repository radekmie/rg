use tower_lsp::lsp_types::SymbolKind;
use tower_lsp::lsp_types::{self as l, Location, Url};

use crate::rg::symbol::Flag;
use rg::position::*;

// https://microsoft.github.io/monaco-editor/typedoc/enums/languages.SymbolKind.html
impl From<&Flag> for SymbolKind {
    fn from(flag: &Flag) -> Self {
        match flag {
            Flag::Constant => SymbolKind::VARIABLE,
            Flag::Variable => SymbolKind::FUNCTION,
            Flag::Edge => SymbolKind::CLASS,
            Flag::Param => SymbolKind::ARRAY,
            Flag::Type => SymbolKind::PACKAGE,
            Flag::Member => SymbolKind::NULL,
        }
    }
}

pub trait ToLspRange {
    fn to_lsp(&self) -> l::Range;
    fn to_location(&self, url: &Url) -> Location {
        Location {
            uri: url.clone(),
            range: self.to_lsp(),
        }
    }
}

pub trait ToLspPosition {
    fn to_lsp(&self) -> l::Position;
}

pub trait ToRgSpan {
    fn to_rg(&self) -> Span;
}

pub trait ToRgPosition {
    fn to_rg(&self) -> Position;
}

impl ToLspRange for Span {
    fn to_lsp(&self) -> l::Range {
        l::Range {
            start: self.start.to_lsp(),
            end: self.end.to_lsp(),
        }
    }
}

impl ToLspPosition for Position {
    fn to_lsp(&self) -> l::Position {
        if self.is_none() {
            panic!("Called toLsp on a none position")
        } else {
            l::Position {
                line: (self.line - 1) as u32,
                character: (self.column - 1) as u32,
            }
        }
    }
}

impl ToRgPosition for l::Position {
    fn to_rg(&self) -> Position {
        Position {
            line: (self.line + 1) as usize,
            column: (self.character + 1) as usize,
        }
    }
}

impl ToRgSpan for l::Range {
    fn to_rg(&self) -> Span {
        Span {
            start: self.start.to_rg(),
            end: self.end.to_rg(),
        }
    }
}
