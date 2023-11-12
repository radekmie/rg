use std::collections::HashMap;

use tower_lsp::lsp_types::{
    self as l, Diagnostic, GotoDefinitionResponse, Hover, Location, PrepareRenameResponse,
    TextEdit, WorkspaceEdit,
};
use tower_lsp::lsp_types::{DocumentSymbolResponse, SymbolInformation, Url};

use crate::rg::ast::Game;
use crate::rg::error::Error;
use crate::rg::symbol_table::*;

use super::utils::*;

#[allow(deprecated)]
pub fn document_symbol(uri: &Url, symbol_table: &SymbolTable) -> Option<DocumentSymbolResponse> {
    let symbols = symbol_table
        .symbols
        .iter()
        .filter(|symbol| !symbol.pos.is_none())
        .map(|symbol| SymbolInformation {
            name: symbol.id.clone(),
            kind: flag_to_kind(&symbol.flag),
            location: l::Location {
                uri: uri.clone(),
                range: symbol.pos.into(),
            },
            deprecated: None,
            container_name: None,
            tags: None,
        })
        .collect();
    Some(DocumentSymbolResponse::Flat(symbols))
}

pub fn references(
    uri: &Url,
    position: l::Position,
    symbol_table: &SymbolTable,
) -> Option<Vec<Location>> {
    let enclosing_symbol = symbol_table.symbol_enclosing_pos(position)?;
    let sym_idx = symbol_table.sym_idx(enclosing_symbol)?;
    let mut all_occurrences = symbol_table.all_symbol_occurences(sym_idx);
    if !all_occurrences.is_empty() && all_occurrences[0].pos == enclosing_symbol.pos {
        // first occurrence is the definition
        all_occurrences.remove(0);
    }
    Some(
        all_occurrences
            .iter()
            .map(|occ| to_location(uri, occ.pos))
            .collect(),
    )
}

pub fn definitions(
    uri: &Url,
    position: l::Position,
    symbol_table: &SymbolTable,
) -> Option<GotoDefinitionResponse> {
    let enclosing_symbol = symbol_table.symbol_enclosing_pos(position)?;
    if enclosing_symbol.pos.is_none() {
        None
    } else {
        let location = to_location(uri, enclosing_symbol.pos);
        Some(GotoDefinitionResponse::Scalar(location))
    }
}

pub fn document_highlight(
    position: l::Position,
    symbol_table: &SymbolTable,
) -> Option<Vec<l::DocumentHighlight>> {
    let enclosing_symbol = symbol_table.symbol_enclosing_pos(position)?;
    let sym_idx = symbol_table.sym_idx(enclosing_symbol)?;
    let all_occurrences = symbol_table.all_symbol_occurences(sym_idx);
    Some(
        all_occurrences
            .iter()
            .map(|occ| l::DocumentHighlight {
                range: occ.pos.into(),
                kind: None,
            })
            .collect(),
    )
}

pub fn prepare_rename(
    position: l::Position,
    symbol_table: &SymbolTable,
) -> Option<PrepareRenameResponse> {
    let enclosing_occ = symbol_table.occ_enclosing_pos(position)?;
    let symbol = symbol_table.get_occ_symbol(enclosing_occ)?;
    if symbol.pos.is_none() {
        return None;
    }
    Some(PrepareRenameResponse::RangeWithPlaceholder {
        range: enclosing_occ.pos.into(),
        placeholder: symbol.id.clone(),
    })
}

pub fn rename(
    uri: &Url,
    position: l::Position,
    symbol_table: &SymbolTable,
    new_name: String,
) -> Option<WorkspaceEdit> {
    let symbol = symbol_table.symbol_enclosing_pos(position)?;
    let sym_idx = symbol_table.sym_idx(symbol)?;
    if symbol.pos.is_none() {
        return None;
    }
    let text_edits = symbol_table
        .all_symbol_occurences(sym_idx)
        .iter()
        .map(|occ| TextEdit {
            range: occ.pos.into(),
            new_text: new_name.clone(),
        })
        .collect();
    let mut changes = HashMap::new();
    changes.insert(uri.clone(), text_edits);
    Some(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

pub fn diagnostics(errors: Vec<Error>) -> Vec<Diagnostic> {
    errors
        .iter()
        .map(|Error { span, message, .. }| l::Diagnostic {
            range: span.start.into(),
            severity: Some(l::DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: Some("rg-lsp".into()),
            message: message.clone(),
            related_information: None,
            tags: None,
            data: None,
        })
        .collect()
}

pub fn hover(position: l::Position, symbol_table: &SymbolTable, game: &Game) -> Option<l::Hover> {
    let occ = symbol_table.occ_enclosing_pos(position)?;
    let pos = occ.pos;
    let enclosing_symbol = symbol_table.get_occ_symbol(occ)?;
    let str = game.hover_signature(enclosing_symbol)?;
    let contents = l::HoverContents::Array(vec![l::MarkedString::from_language_code(
        "rg".to_string(),
        str,
    )]);
    Some(Hover {
        contents,
        range: Some(pos.into()),
    })
}
