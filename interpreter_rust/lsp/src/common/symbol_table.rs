use std::fmt::{Display, Formatter, Result};

use utils::position::{Position, Positioned, Span};

use super::symbol::Symbol;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Occurrence {
    pub pos: Span,
    pub symbol: Option<usize>,
}

impl Occurrence {
    pub fn new(pos: Span, symbol: Option<usize>) -> Self {
        Self { pos, symbol }
    }
}

impl Positioned for Occurrence {
    fn span(&self) -> Span {
        self.pos
    }
}

pub struct SymbolTable {
    pub occurrences: Vec<Occurrence>,
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn get_occ_at(&self, pos: &Position) -> Option<&Occurrence> {
        self.occurrences
            .iter()
            .find(|occ| occ.pos.encloses_position(pos))
    }

    pub fn get_occ_symbol(&self, occ: &Occurrence) -> Option<&Symbol> {
        occ.symbol.and_then(|idx| self.symbols.get(idx))
    }

    pub fn get_symbol_at(&self, pos: &Position) -> Option<&Symbol> {
        self.get_occ_at(pos)
            .and_then(|occ| self.get_occ_symbol(occ))
    }

    pub fn sym_idx(&self, symbol: &Symbol) -> Option<usize> {
        self.symbols.iter().position(|sym| sym == symbol)
    }

    pub fn all_symbol_occurences(&self, symbol_idx: usize) -> Vec<Occurrence> {
        self.occurrences
            .iter()
            .filter(|occ| occ.symbol.is_some_and(|sym| sym == symbol_idx))
            .cloned()
            .collect()
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Symbols:\n")?;
        for (idx, symbol) in self.symbols.iter().enumerate() {
            writeln!(f, "{idx}. {}  {symbol}", symbol.pos)?;
        }

        writeln!(f, "Occurrences:\n")?;
        for occ in &self.occurrences {
            let symbol = self.get_occ_symbol(occ).unwrap();
            writeln!(f, "{}  {symbol}", occ.pos)?;
        }

        Ok(())
    }
}
