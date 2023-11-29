use std::collections::HashMap;

use rg::position::Positioned;
use tower_lsp::lsp_types::{
    self as l, Diagnostic, GotoDefinitionResponse, Hover, Location, PrepareRenameResponse,
    TextEdit, WorkspaceEdit,
};
use tower_lsp::lsp_types::{DocumentSymbolResponse, SymbolInformation, Url};

use crate::rg::ast_features::hover_signature;
use crate::rg::symbol_table::*;
use crate::utils::ToRgPosition;
use rg::ast::{Game, Identifier};
use rg::parsing::error::Error;

use super::utils::ToLspRange;

#[allow(deprecated)]
pub fn document_symbol(uri: &Url, symbol_table: &SymbolTable) -> Option<DocumentSymbolResponse> {
    let symbols = symbol_table
        .symbols
        .iter()
        .filter(|symbol| !symbol.pos.is_none())
        .map(|symbol| SymbolInformation {
            name: symbol.id.clone(),
            kind: (&symbol.flag).into(),
            location: l::Location {
                uri: uri.clone(),
                range: symbol.pos.to_lsp(),
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
    position: &l::Position,
    symbol_table: &SymbolTable,
) -> Option<Vec<Location>> {
    let enclosing_symbol = symbol_table.get_symbol_at(&position.to_rg())?;
    let sym_idx = symbol_table.sym_idx(enclosing_symbol)?;
    let all_occurrences = symbol_table.all_symbol_occurences(sym_idx);
    let all_occurrences: Vec<Location> = all_occurrences
        .into_iter()
        .filter(|occ| !occ.span().equal_span(&enclosing_symbol.span()))
        .map(|occ| occ.pos.to_location(uri))
        .collect();
    Some(all_occurrences)
}

pub fn definitions(
    uri: &Url,
    position: &l::Position,
    symbol_table: &SymbolTable,
) -> Option<GotoDefinitionResponse> {
    let enclosing_symbol = symbol_table.get_symbol_at(&position.to_rg())?;
    enclosing_symbol.safe_pos().map(|pos| {
        let location = pos.to_location(uri);
        GotoDefinitionResponse::Scalar(location)
    })
}

pub fn document_highlight(
    position: &l::Position,
    symbol_table: &SymbolTable,
) -> Option<Vec<l::DocumentHighlight>> {
    let enclosing_symbol = symbol_table.get_symbol_at(&position.to_rg())?;
    let sym_idx = symbol_table.sym_idx(enclosing_symbol)?;
    let all_occurrences = symbol_table.all_symbol_occurences(sym_idx);
    Some(
        all_occurrences
            .iter()
            .map(|occ| l::DocumentHighlight {
                range: occ.pos.to_lsp(),
                kind: None,
            })
            .collect(),
    )
}

pub fn prepare_rename(
    position: &l::Position,
    symbol_table: &SymbolTable,
) -> Option<PrepareRenameResponse> {
    let enclosing_occ = symbol_table.get_occ_at(&position.to_rg())?;
    let symbol = symbol_table.get_occ_symbol(enclosing_occ)?;
    symbol
        .safe_pos()
        .map(|_| PrepareRenameResponse::RangeWithPlaceholder {
            range: enclosing_occ.pos.to_lsp(),
            placeholder: symbol.id.clone(),
        })
}

pub fn rename(
    uri: &Url,
    position: &l::Position,
    symbol_table: &SymbolTable,
    new_name: String,
) -> Option<WorkspaceEdit> {
    let symbol = symbol_table.get_symbol_at(&position.to_rg())?;
    let sym_idx = symbol_table.sym_idx(symbol)?;
    symbol.safe_pos().map(|_| {
        let text_edits = symbol_table
            .all_symbol_occurences(sym_idx)
            .iter()
            .map(|occ| TextEdit {
                range: occ.pos.to_lsp(),
                new_text: new_name.clone(),
            })
            .collect();
        let changes = HashMap::from([(uri.clone(), text_edits)]);
        WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }
    })
}

pub fn diagnostics(errors: Vec<Error>) -> Vec<Diagnostic> {
    errors
        .iter()
        .map(|Error { span, message, .. }| l::Diagnostic {
            range: span.to_lsp(),
            severity: Some(l::DiagnosticSeverity::ERROR),
            source: Some("rg-lsp".into()),
            message: message.clone(),
            ..Diagnostic::default()
        })
        .collect()
}

pub fn hover(
    position: &l::Position,
    symbol_table: &SymbolTable,
    game: &Game<Identifier>,
) -> Option<l::Hover> {
    let occ = symbol_table.get_occ_at(&position.to_rg())?;
    let pos = &occ.pos;
    let enclosing_symbol = symbol_table.get_occ_symbol(occ)?;
    let str = hover_signature(game, enclosing_symbol)?;
    let contents = l::HoverContents::Array(vec![l::MarkedString::from_language_code(
        "rg".to_string(),
        str,
    )]);
    Some(Hover {
        contents,
        range: Some(pos.to_lsp()),
    })
}
