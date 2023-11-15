use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities, WorkDoneProgressOptions,
};

use rg::{
    ast::{Game, Identifier, Pragma},
    position::*,
};

use crate::rg::{symbol::Flag, symbol_table::*};

use tower_lsp::lsp_types::Position as LPos;

use super::{document::Document, utils::pos_to_lsp};

pub fn capabilities() -> SemanticTokensServerCapabilities {
    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
        work_done_progress_options: WorkDoneProgressOptions::default(),
        legend: SemanticTokensLegend {
            token_types: SEMANTIC_TOKENS_TYPES
                .to_vec()
                .into_iter()
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
        .unwrap() as u32
}

pub const SEMANTIC_TOKENS_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
];

fn semantic_token_modifier(token_modifier: SemanticTokenModifier) -> u32 {
    SEMANTIC_TOKENS_MODIFIERS
        .iter()
        .position(|t| t == &token_modifier)
        .unwrap() as u32
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Token {
    pos: LPos,
    len: u32,
    token_type: u32,
    token_modifier: u32,
}

pub fn semantic_tokens_full(document: &Document) -> Vec<SemanticToken> {
    let comments = comment_tokens(&document.text.as_str());
    let keywords = ast_tokens(&document.game);
    let symbols = symbol_table_tokens(&document.symbol_table);
    let mut tokens = [&comments[..], &keywords[..], &symbols[..]].concat();
    tokens.sort_by(|a, b| a.pos.cmp(&b.pos));
    let mut semantic_tokens = Vec::new();
    let mut last_line = 0;
    let mut last_col_end = 0;
    let mut last_len = 0;
    for token in tokens.iter() {
        let (d_line, d_column) = if token.pos.line == last_line {
            (0, token.pos.character - last_col_end + last_len)
        } else {
            (token.pos.line - last_line, token.pos.character)
        };
        last_line = token.pos.line;
        last_col_end = token.pos.character + token.len;
        last_len = token.len;
        let semantic_token = SemanticToken {
            delta_line: d_line,
            delta_start: d_column,
            length: token.len,
            token_type: token.token_type,
            token_modifiers_bitset: token.token_modifier,
        };
        semantic_tokens.push(semantic_token);
    }

    semantic_tokens
}

fn ast_tokens(game: &Game<Identifier>) -> Vec<Token> {
    let mut tokens = Vec::new();
    for constant in game.constants.iter() {
        let token = Token {
            pos: pos_to_lsp(constant.start()),
            len: 5, // "const".len()
            token_type: semantic_token_type("keyword"),
            token_modifier: 0,
        };
        tokens.push(token);
    }

    for variable in game.variables.iter() {
        let token = Token {
            pos: pos_to_lsp(variable.start()),
            len: 3, // "var".len()
            token_type: semantic_token_type("keyword"),
            token_modifier: 0,
        };
        tokens.push(token);
    }
    for typedef in game.typedefs.iter() {
        let token = Token {
            pos: pos_to_lsp(typedef.start()),
            len: 4, // "type".len()
            token_type: semantic_token_type("keyword"),
            token_modifier: 0,
        };
        tokens.push(token);
    }
    for pragma in game.pragmas.iter() {
        let len = match pragma {
            Pragma::Any { .. } => 3,      // "any".len()
            Pragma::Disjoint { .. } => 8, // "disjoint".len()
            Pragma::MultiAny { .. } => 9, // "multi_any".len()
            Pragma::Unique { .. } => 6,   // "unique".len()
        } + 1; // +1 for `@`
        let token = Token {
            pos: pos_to_lsp(pragma.start()),
            len,
            token_type: semantic_token_type("macro"),
            token_modifier: 0,
        };
        tokens.push(token);
    }
    tokens
}

fn comment_tokens(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        if let Some(col) = line.find("//") {
            let token = Token {
                pos: pos_to_lsp(Position {
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
                pos: pos_to_lsp(occ.start()),
                len: symbol.id.len() as u32,
                token_type,
                token_modifier: definition_mod | const_mod,
            };
            tokens.push(token);
        }
    }
    tokens
}
