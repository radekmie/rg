use crate::common::{
    symbol::Symbol,
    symbol_table::{Occurrence, SymbolTable},
};
use hrg::ast::GameDeclaration;
use utils::{Error, Identifier};

struct SymbolTableWithErrors {
    errors: Vec<Error>,
    occurrences: Vec<Occurrence>,
    symbols: Vec<Symbol>,
}

impl SymbolTableWithErrors {
    fn from_game(_game: &GameDeclaration<Identifier>) -> Self {
        Self {
            errors: vec![],
            occurrences: vec![],
            symbols: vec![],
        }
    }
}

pub fn from_game(game: &GameDeclaration<Identifier>) -> (SymbolTable, Vec<Error>) {
    let table = SymbolTableWithErrors::from_game(game);
    (
        SymbolTable {
            symbols: table.symbols,
            occurrences: table.occurrences,
        },
        table.errors,
    )
}
