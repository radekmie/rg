use crate::rg::symbol_table::*;
use rg::ast::{Game, Identifier};
use rg::error::Error;
use rg::parser::parse_with_errors;

pub struct Document {
    pub text: String,
    pub game: Game<Identifier>,
    pub symbol_table: SymbolTable,
}

impl Document {
    pub fn new(content: String) -> (Self, Vec<Error>) {
        let (game, mut parse_errors) = parse_with_errors(&content);
        let (symbol_table, mut symbol_table_errors) = SymbolTable::from_game(&game);
        parse_errors.append(&mut symbol_table_errors);
        (
            Document {
                text: content,
                game,
                symbol_table,
            },
            parse_errors,
        )
    }
}
