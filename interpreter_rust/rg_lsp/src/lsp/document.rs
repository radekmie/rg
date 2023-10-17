use ropey::Rope;
use crate::ast::Game;
use crate::parser::parse;

use super::symbols::DocumentSymbols;

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


