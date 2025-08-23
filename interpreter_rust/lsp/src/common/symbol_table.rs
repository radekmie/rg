use super::symbol::{Flag, Symbol};
use std::fmt::{Display, Formatter, Result};
use utils::position::{Position, Positioned, Span};
use utils::{Identifier, ParserError};

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

    pub fn get_occ_symbol(&self, occ: &Occurrence) -> Option<(usize, &Symbol)> {
        occ.symbol
            .and_then(|idx| self.symbols.get(idx).map(|sym| (idx, sym)))
    }

    pub fn get_symbol_at(&self, pos: &Position) -> Option<(usize, &Symbol)> {
        self.get_occ_at(pos)
            .and_then(|occ| self.get_occ_symbol(occ))
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
            let (_, symbol) = self.get_occ_symbol(occ).unwrap();
            writeln!(f, "{}  {symbol}", occ.pos)?;
        }

        Ok(())
    }
}

pub struct SymbolTableBuilder {
    pub errors: Vec<ParserError>,
    pub occurrences: Vec<Occurrence>,
    pub symbols: Vec<Symbol>,
}

impl SymbolTableBuilder {
    /** The last symbol with matching id defined before this position is used. */
    pub fn find_symbol(
        &self,
        id: &Identifier,
        flag: &Option<Flag>,
        owner: &Option<usize>,
    ) -> Option<usize> {
        self.symbols.iter().rposition(|symbol| {
            symbol.id == id.identifier
                && (symbol.span().start <= id.span().start
                    || symbol.flag == Flag::Function
                    || symbol.flag == Flag::Type)
                && flag.as_ref().is_none_or(|f| symbol.flag == *f)
                && owner.as_ref().is_none_or(|o| symbol.is_owned_by(*o))
        })
    }

    pub fn occ_from_id(&self, identifier: &Identifier) -> Occurrence {
        let span = identifier.span();
        let symbol_idx = self.find_symbol(identifier, &None, &None);
        Occurrence::new(span, symbol_idx)
    }

    pub fn occ_with_flag(&self, identifier: &Identifier, flag: Flag) -> Occurrence {
        let span = identifier.span();
        let symbol_idx = self.find_symbol(identifier, &Some(flag), &None);
        Occurrence::new(span, symbol_idx)
    }

    pub fn add_occ(&mut self, identifier: &Identifier) {
        if !identifier.is_none() && !identifier.is_numeric() {
            let occ = self.occ_from_id(identifier);
            if occ.symbol.is_none() {
                self.errors
                    .push(ParserError::new_unknown_identifier(identifier));
            } else {
                self.occurrences.push(occ);
            }
        }
    }

    pub fn add_occ_with_flag(&mut self, identifier: &Identifier, flag: Flag) {
        if !(identifier.is_none() || identifier.is_numeric() && flag != Flag::Function) {
            let occ = self.occ_with_flag(identifier, flag);
            if occ.symbol.is_none() {
                self.errors
                    .push(ParserError::new_unknown_identifier(identifier));
            } else {
                self.occurrences.push(occ);
            }
        }
    }

    pub fn add_occ_with_flag_and_owner(
        &mut self,
        identifier: &Identifier,
        flag: Flag,
        owner: &Option<usize>,
    ) {
        if !identifier.is_none() && !identifier.is_numeric() {
            let span = identifier.span();
            let symbol_idx = self.find_symbol(identifier, &Some(flag), owner);
            if symbol_idx.is_none() {
                self.errors
                    .push(ParserError::new_unknown_identifier(identifier));
            } else {
                self.occurrences.push(Occurrence::new(span, symbol_idx));
            }
        }
    }

    pub fn is_defined(&self, symbol: &str) -> bool {
        self.symbols.iter().any(|sym| sym.id == symbol)
    }
}
