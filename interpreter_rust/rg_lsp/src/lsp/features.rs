use tower_lsp::lsp_types::{self as l, GotoDefinitionResponse, Location};
use tower_lsp::lsp_types::{DocumentSymbolResponse, SymbolInformation, Url};

use crate::rg::position::Positioned;
use crate::rg::symbols::*;

use super::utils::*;

pub fn document_symbol(uri: &Url, symbol_table: &SymbolTable) -> Option<DocumentSymbolResponse> {
    let mut symbols = Vec::new();
    for symbol in symbol_table.symbols.iter() {
        let symbol_information = SymbolInformation {
            name: symbol.id.clone(),
            kind: flag_to_kind(symbol.flag),
            location: l::Location {
                uri: uri.clone(),
                range: symbol.pos.into(),
            },
            deprecated: None,
            container_name: None,
            tags: None,
        };
        symbols.push(symbol_information);
    }
    let document_symbol_response = DocumentSymbolResponse::Flat(symbols);
    Some(document_symbol_response)
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
    let location = to_location(uri, enclosing_symbol.pos);
    Some(GotoDefinitionResponse::Scalar(location))
}
