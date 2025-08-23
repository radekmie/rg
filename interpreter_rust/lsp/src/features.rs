use super::common::utils::ToLspRange;
use crate::common::symbol_table::SymbolTable;
use crate::common::utils::ToPosition;
use crate::rg::ast_features::hover_signature;
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, DocumentHighlight, GotoDefinitionResponse, Hover,
    HoverContents, Location, MarkedString, Position, PrepareRenameResponse, TextEdit,
    WorkspaceEdit,
};
use tower_lsp::lsp_types::{DocumentSymbolResponse, SymbolInformation, Url};
use utils::position::Positioned;
use utils::ParserError;

#[expect(deprecated)]
pub fn document_symbol(uri: &Url, symbol_table: &SymbolTable) -> Option<DocumentSymbolResponse> {
    let symbols = symbol_table
        .symbols
        .iter()
        .filter(|symbol| !symbol.pos.is_none())
        .map(|symbol| SymbolInformation {
            name: symbol.id.clone(),
            kind: (&symbol.flag).into(),
            location: Location {
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
    position: Position,
    symbol_table: &SymbolTable,
) -> Option<Vec<Location>> {
    let (sym_idx, enclosing_symbol) = symbol_table.get_symbol_at(&position.to_rg())?;
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
    position: Position,
    symbol_table: &SymbolTable,
) -> Option<GotoDefinitionResponse> {
    let enclosing_symbol = symbol_table.get_symbol_at(&position.to_rg())?.1;
    enclosing_symbol.safe_pos().map(|pos| {
        let location = pos.to_location(uri);
        GotoDefinitionResponse::Scalar(location)
    })
}

pub fn document_highlight(
    position: Position,
    symbol_table: &SymbolTable,
) -> Option<Vec<DocumentHighlight>> {
    let sym_idx = symbol_table.get_symbol_at(&position.to_rg())?.0;
    let all_occurrences = symbol_table.all_symbol_occurences(sym_idx);
    Some(
        all_occurrences
            .iter()
            .map(|occ| DocumentHighlight {
                range: occ.pos.to_lsp(),
                kind: None,
            })
            .collect(),
    )
}

pub fn prepare_rename(
    position: Position,
    symbol_table: &SymbolTable,
) -> Option<PrepareRenameResponse> {
    let enclosing_occ = symbol_table.get_occ_at(&position.to_rg())?;
    let symbol = symbol_table.get_occ_symbol(enclosing_occ)?.1;
    symbol
        .safe_pos()
        .map(|_| PrepareRenameResponse::RangeWithPlaceholder {
            range: enclosing_occ.pos.to_lsp(),
            placeholder: symbol.id.clone(),
        })
}

pub fn rename(
    uri: &Url,
    position: Position,
    symbol_table: &SymbolTable,
    new_name: &str,
) -> Option<WorkspaceEdit> {
    let (sym_idx, symbol) = symbol_table.get_symbol_at(&position.to_rg())?;
    symbol.safe_pos().map(|_| {
        let text_edits = symbol_table
            .all_symbol_occurences(sym_idx)
            .iter()
            .map(|occ| TextEdit {
                range: occ.pos.to_lsp(),
                new_text: new_name.to_string(),
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

pub fn diagnostics(errors: &[ParserError]) -> Vec<Diagnostic> {
    errors
        .iter()
        .map(|ParserError { span, message, .. }| Diagnostic {
            range: span.to_lsp(),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("rg-lsp".into()),
            message: message.clone(),
            ..Diagnostic::default()
        })
        .collect()
}

pub fn hover(position: Position, symbol_table: &SymbolTable) -> Option<Hover> {
    let occ = symbol_table.get_occ_at(&position.to_rg())?;
    let pos = &occ.pos;
    let enclosing_symbol = symbol_table.get_occ_symbol(occ)?.1;
    let str = hover_signature(enclosing_symbol)?;
    let contents = HoverContents::Array(vec![MarkedString::from_language_code(
        "rg".to_string(),
        str,
    )]);
    Some(Hover {
        contents,
        range: Some(pos.to_lsp()),
    })
}
