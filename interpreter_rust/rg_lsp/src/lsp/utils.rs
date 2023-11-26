use tower_lsp::lsp_types::SymbolKind;
use tower_lsp::lsp_types::{self as l, Location, Url};

use crate::rg::symbol::Flag;
use rg::position::*;

// https://microsoft.github.io/monaco-editor/typedoc/enums/languages.SymbolKind.html
pub fn flag_to_kind(flag: &Flag) -> SymbolKind {
    match flag {
        Flag::Constant => SymbolKind::VARIABLE,
        Flag::Variable => SymbolKind::FUNCTION,
        Flag::Edge => SymbolKind::CLASS,
        Flag::Param => SymbolKind::ARRAY,
        Flag::Type => SymbolKind::PACKAGE,
        Flag::Member => SymbolKind::NULL,
    }
}

pub fn span_to_lsp(span: &Span) -> l::Range {
    l::Range {
        start: pos_to_lsp(&span.start),
        end: pos_to_lsp(&span.end),
    }
}

pub fn pos_to_lsp(position: &Position) -> l::Position {
    if position.is_none() {
        panic!("Called toLsp on a none position")
    } else {
        l::Position {
            line: (position.line - 1) as u32,
            character: (position.column - 1) as u32,
        }
    }
}

pub fn pos_to_lsp_range(position: &Position) -> l::Range {
    l::Range {
        start: pos_to_lsp(position),
        end: pos_to_lsp(position),
    }
}

pub fn lsp_to_pos(position: &l::Position) -> Position {
    Position {
        line: (position.line + 1) as usize,
        column: (position.character + 1) as usize,
    }
}

pub fn lsp_to_span(range: &l::Range) -> Span {
    Span {
        start: lsp_to_pos(&range.start),
        end: lsp_to_pos(&range.end),
    }
}

pub fn to_location(url: &Url, span: &Span) -> Location {
    Location {
        uri: url.clone(),
        range: span_to_lsp(span),
    }
}
