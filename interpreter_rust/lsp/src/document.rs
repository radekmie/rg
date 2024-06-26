use crate::common::symbol_table::SymbolTable;
use crate::hrg::symbol_table::from_game as hrg_symbol_table;
use crate::rg::symbol_table::from_game as rg_symbol_table;
use hrg::parsing::parser::parse_with_errors as hrg_parse;
use rg::parsing::parser::parse_with_errors as rg_parse;
use tower_lsp::lsp_types::Url;
use utils::{Error, Identifier};

pub struct Document {
    pub tree: Ast,
    pub symbol_table: SymbolTable,
    pub text: String,
}

pub enum Ast {
    Rg(rg::ast::Game<Identifier>),
    Hrg,
}

impl Document {
    pub fn new(uri: &Url, text: String) -> (Self, Vec<Error>) {
        if uri.as_str().ends_with(".rg") {
            Self::rg(text)
        } else {
            Self::hrg(text)
        }
    }
    fn rg(text: String) -> (Self, Vec<Error>) {
        let (game, mut parse_errors) = rg_parse(&text);
        let (symbol_table, mut symbol_table_errors) = rg_symbol_table(&game);
        parse_errors.append(&mut symbol_table_errors);
        (
            Self {
                tree: Ast::Rg(game),
                symbol_table,
                text,
            },
            parse_errors,
        )
    }

    fn hrg(text: String) -> (Self, Vec<Error>) {
        let (game, mut parse_errors) = hrg_parse(&text);
        let (symbol_table, mut symbol_table_errors) = hrg_symbol_table(&game);
        parse_errors.append(&mut symbol_table_errors);
        (
            Self {
                tree: Ast::Hrg,
                symbol_table,
                text,
            },
            parse_errors,
        )
    }
}
