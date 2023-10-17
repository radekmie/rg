use crate::rg::ast::Game;
use crate::rg::parser::parse;
use crate::rg::symbols::*;
use ropey::Rope;

pub struct Document {
    pub text: Rope,
    pub game: Game,
    pub document_symbols: DocumentSymbols,
}

impl Document {
    pub fn new(content: String) -> Self {
        let text = Rope::from_str(&content);
        let game = parse(&content);
        let document_symbols = DocumentSymbols::new(&game);
        Self {
            text,
            game,
            document_symbols,
        }
    }
}

pub struct DocumentSymbols {
    pub symbols: Vec<Symbol>,
    pub occurences: Vec<Occurrence>,
}

impl DocumentSymbols {
    pub fn new(game: &Game) -> Self {
        let symbols = Symbol::from_game(game);
        let occurences = Occurrence::from_game(game);
        Self {
            symbols,
            occurences,
        }
    }
}
