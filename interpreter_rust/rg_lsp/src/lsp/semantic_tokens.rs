use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities, WorkDoneProgressOptions,
};

use rg::{
    ast::{Game, Identifier},
    position::*,
};

use crate::rg::{stat::Stat, symbol::Flag, symbol_table::*};

use tower_lsp::lsp_types::Position as LPos;

use super::{document::Document, utils::pos_to_lsp};

pub fn capabilities() -> SemanticTokensServerCapabilities {
    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
        work_done_progress_options: WorkDoneProgressOptions::default(),
        legend: SemanticTokensLegend {
            token_types: SEMANTIC_TOKENS_TYPES
                .iter()
                .copied()
                .map(SemanticTokenType::new)
                .collect(),
            token_modifiers: SEMANTIC_TOKENS_MODIFIERS.to_vec(),
        },
        range: None,
        full: Some(tower_lsp::lsp_types::SemanticTokensFullOptions::Bool(true)),
    })
}
pub const SEMANTIC_TOKENS_TYPES: &[&str] = &[
    "keyword",
    "type",
    "parameter",
    "variable",
    "function",
    "comment",
    "operator",
    "macro",
    "member",
    "constant",
];

fn semantic_token_type(token_type: &str) -> u32 {
    SEMANTIC_TOKENS_TYPES
        .iter()
        .position(|t| t == &token_type)
        .expect("Unknown token type") as u32
}

pub const SEMANTIC_TOKENS_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
];

fn semantic_token_modifier(token_modifier: SemanticTokenModifier) -> u32 {
    SEMANTIC_TOKENS_MODIFIERS
        .iter()
        .position(|t| t == &token_modifier)
        .expect("Unknown token modifier") as u32
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Token {
    pos: LPos,
    len: u32,
    token_type: u32,
    token_modifier: u32,
}

#[derive(Debug, Clone, Default)]
struct Delta {
    last_line: u32,
    last_col_end: u32,
    line: u32,
    column: u32,
}
impl Delta {
    fn step(&mut self, pos: &LPos) {
        let (d_line, d_column) = if pos.line == self.last_line {
            (0, pos.character - self.last_col_end)
        } else {
            (pos.line - self.last_line, pos.character)
        };
        self.line = d_line;
        self.column = d_column;
        self.last_line = pos.line;
        self.last_col_end = pos.character;
    }
}

pub fn semantic_tokens_full(document: &Document) -> Vec<SemanticToken> {
    let comments = comment_tokens(document.text.as_str());
    let keywords = ast_tokens(&document.game);
    let symbols = symbol_table_tokens(&document.symbol_table);
    let mut tokens = [&comments[..], &keywords[..], &symbols[..]].concat();
    tokens.sort_by_key(|t| t.pos);
    let mut delta = Delta::default();
    tokens
        .into_iter()
        .map(|token| {
            delta.step(&token.pos);
            SemanticToken {
                delta_line: delta.line,
                delta_start: delta.column,
                length: token.len,
                token_type: token.token_type,
                token_modifiers_bitset: token.token_modifier,
            }
        })
        .collect()
}

fn ast_tokens(game: &Game<Identifier>) -> Vec<Token> {
    let mut tokens = Vec::new();
    Stat::from_game(game).iter().for_each(|stat| {
        let keyword = stat.keyword();
        if !keyword.is_empty() {
            let token_type = match stat {
                Stat::Pragma(_) => semantic_token_type("macro"),
                _ => semantic_token_type("keyword"),
            };
            let token = Token {
                pos: pos_to_lsp(&stat.span().start),
                len: keyword.len() as u32,
                token_type,
                token_modifier: 0,
            };
            tokens.push(token);
        }
    });
    tokens
}

fn comment_tokens(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        if let Some(col) = line.find("//") {
            let token = Token {
                pos: pos_to_lsp(&Position {
                    line: line_idx + 1,
                    column: col + 1,
                }),
                len: (line.len() - col) as u32,
                token_type: semantic_token_type("comment"),
                token_modifier: 0,
            };
            tokens.push(token);
        }
    }

    tokens
}

fn symbol_table_tokens(symbol_table: &SymbolTable) -> Vec<Token> {
    let mut tokens = Vec::new();
    for occ in symbol_table.occurrences.iter() {
        let symbol = symbol_table.get_occ_symbol(occ);
        if let Some(symbol) = symbol {
            let definition_mod = if symbol.pos.equal_span(&occ.pos) {
                semantic_token_modifier(SemanticTokenModifier::DEFINITION)
            } else {
                0
            };
            let token_type = match symbol.flag {
                Flag::Constant => semantic_token_type("constant"),
                Flag::Edge => semantic_token_type("function"),
                Flag::Type => semantic_token_type("type"),
                Flag::Variable => semantic_token_type("variable"),
                Flag::Member => semantic_token_type("member"),
                Flag::Param => semantic_token_type("parameter"),
            };
            let const_mod = if symbol.flag == Flag::Constant {
                semantic_token_modifier(SemanticTokenModifier::READONLY)
            } else {
                0
            };
            // web_sys::console::log_1(&format!("{}: {}", symbol.id, occ.pos).into());
            let token = Token {
                pos: pos_to_lsp(&occ.start()),
                len: symbol.id.len() as u32,
                token_type,
                token_modifier: definition_mod | const_mod,
            };
            tokens.push(token);
        }
    }
    tokens
}
