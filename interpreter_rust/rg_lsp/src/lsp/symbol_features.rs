use crate::rg::symbol::Symbol;
use crate::rg::symbol_table::*;
use tower_lsp::lsp_types as l;

use super::utils::{lsp_to_pos, lsp_to_span};

impl SymbolTable {
    pub fn symbol_enclosing_range(&self, range: l::Range) -> Option<&Symbol> {
        self.get_symbol_at_span(&lsp_to_span(range))
    }

    pub fn symbol_enclosing_pos(&self, pos: l::Position) -> Option<&Symbol> {
        self.get_symbol_at(&lsp_to_pos(pos))
    }

    pub fn occ_enclosing_range(&self, range: l::Range) -> Option<&Occurrence> {
        self.get_occ_at_span(&lsp_to_span(range))
    }

    pub fn occ_enclosing_pos(&self, pos: l::Position) -> Option<&Occurrence> {
        self.get_occ_at(&lsp_to_pos(pos))
    }

    pub fn all_symbol_occurences(&self, symbol_idx: usize) -> Vec<Occurrence> {
        let mut occurrences = Vec::new();
        for occ in self.occurrences.iter() {
            if occ.symbol.is_some_and(|sym| sym == symbol_idx) {
                occurrences.push(occ.clone());
            }
        }
        occurrences
    }
}
