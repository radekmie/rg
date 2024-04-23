use super::symbol::Flag;
use tower_lsp::lsp_types::{Location, Position as LspPosition, Range, SymbolKind, Url};
use utils::position::{Position, Span};

// https://microsoft.github.io/monaco-editor/typedoc/enums/languages.SymbolKind.html
impl From<&Flag> for SymbolKind {
    fn from(flag: &Flag) -> Self {
        match flag {
            Flag::Constant => Self::VARIABLE,
            Flag::Function => Self::CLASS,
            Flag::Member => Self::NULL,
            Flag::Param => Self::ARRAY,
            Flag::Type => Self::PACKAGE,
            Flag::Variable => Self::FUNCTION,
        }
    }
}

pub trait ToLspRange {
    fn to_lsp(&self) -> Range;
    fn to_location(&self, url: &Url) -> Location {
        Location {
            uri: url.clone(),
            range: self.to_lsp(),
        }
    }
}

pub trait ToLspPosition {
    fn to_lsp(&self) -> LspPosition;
}

pub trait ToSpan {
    fn to_rg(&self) -> Span;
}

pub trait ToPosition {
    fn to_rg(&self) -> Position;
}

impl ToLspRange for Span {
    fn to_lsp(&self) -> Range {
        Range {
            start: self.start.to_lsp(),
            end: self.end.to_lsp(),
        }
    }
}

impl ToLspPosition for Position {
    fn to_lsp(&self) -> LspPosition {
        if self.is_none() {
            panic!("Called toLsp on a none position")
        } else {
            LspPosition {
                line: (self.line - 1) as u32,
                character: (self.column - 1) as u32,
            }
        }
    }
}

impl ToPosition for LspPosition {
    fn to_rg(&self) -> Position {
        Position {
            line: (self.line + 1) as usize,
            column: (self.character + 1) as usize,
        }
    }
}

impl ToSpan for Range {
    fn to_rg(&self) -> Span {
        Span {
            start: self.start.to_rg(),
            end: self.end.to_rg(),
        }
    }
}
