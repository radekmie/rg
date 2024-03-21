use crate::rg::symbol_table::SymbolTable;
use rg::ast::{Game, Identifier};
use rg::parsing::parser::parse_with_errors;
use utils::parsing::error::Error;

pub struct Document {
    pub game: Game<Identifier>,
    pub symbol_table: SymbolTable,
    pub text: String,
}

impl Document {
    pub fn new(text: String) -> (Self, Vec<Error>) {
        let (game, mut parse_errors) = parse_with_errors(&text);
        let (symbol_table, mut symbol_table_errors) = SymbolTable::from_game(&game);
        parse_errors.append(&mut symbol_table_errors);
        (
            Self {
                game,
                symbol_table,
                text,
            },
            parse_errors,
        )
    }
}
