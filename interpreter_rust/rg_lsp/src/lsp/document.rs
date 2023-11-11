use crate::rg::ast::Game;
use crate::rg::error::Error;
use crate::rg::parser::parse_with_errors;
use crate::rg::symbol_table::*;

pub struct Document {
    pub text: String,
    game: Option<Game>,
    symbol_table: Option<SymbolTable>,
}

impl Document {
    pub fn new(content: String) -> Self {
        Document {
            text: content,
            game: None,
            symbol_table: None,
        }
    }

    pub fn parse(&mut self) -> Vec<crate::rg::error::Error> {
        let (game, errors) = parse_with_errors(&self.text);
        self.game = Some(game);
        errors
    }

    pub fn get_game(&mut self) -> &Game {
        if self.game.is_none() {
            self.parse();
        }
        self.game.as_ref().unwrap()
    }

    pub fn make_symbol_table(&mut self)  -> Vec<Error> {
        let game = self.get_game();
        let (symbol_table, errors) = SymbolTable::from_game(game);
        self.symbol_table = Some(symbol_table);
        errors
    }

    pub fn get_symbol_table(&mut self) -> &SymbolTable {
        if self.symbol_table.is_none() {
            self.make_symbol_table();
        }
        self.symbol_table.as_ref().unwrap()
    }
}
