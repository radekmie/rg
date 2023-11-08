use crate::rg::ast::Game;
use crate::rg::parser::{parse, parse_with_errors};
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

    pub fn parse(&mut self) -> Vec<crate::rg::parser::Error> {
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

    fn make_symbol_table(&mut self) {
        let game = self.get_game();
        let symbol_table = SymbolTable::from_game(game);
        self.symbol_table = Some(symbol_table);
    }

    pub fn get_symbol_table(&mut self) -> &SymbolTable {
        if self.symbol_table.is_none() {
            self.make_symbol_table();
        }
        self.symbol_table.as_ref().unwrap()
    }
}
