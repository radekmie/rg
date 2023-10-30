use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities, WorkDoneProgressOptions,
};

use crate::rg::{
    ast::{Game, PragmaKind, Stat},
    position::*,
    symbol::Flag,
    symbol_table::*,
};

use super::document::Document;

pub fn semantic_tokens_capabilities() -> SemanticTokensServerCapabilities {
    SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
        work_done_progress_options: WorkDoneProgressOptions::default(),
        legend: SemanticTokensLegend {
            token_types: SEMANTIC_TOKENS_TYPES.to_vec(),
            token_modifiers: SEMANTIC_TOKENS_MODIFIERS.to_vec(),
        },
        range: None,
        full: Some(tower_lsp::lsp_types::SemanticTokensFullOptions::Bool(true)),
    })
}

pub const SEMANTIC_TOKENS_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,
    SemanticTokenType::TYPE,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::METHOD,
    SemanticTokenType::COMMENT,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::MACRO,
];

fn semantic_token_type(token_type: SemanticTokenType) -> u32 {
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
    pos: Position,
    len: u32,
    token_type: u32,
    token_modifier: u32,
}

pub fn semantic_tokens_full(document: &mut Document) -> Vec<SemanticToken> {
    let comments = comment_tokens(&document.text.as_str());
    let keywords = ast_tokens(document.get_game());
    let symbols = symbol_table_tokens(document.get_symbol_table());
    let mut tokens = [&comments[..], &keywords[..], &symbols[..]].concat();
    tokens.sort_by(|a, b| a.pos.cmp(&b.pos));
    let mut semantic_tokens = Vec::new();
    let mut last_line = 0;
    let mut last_col_end = 0;
    for token in tokens.iter() {
        let (d_line, d_column) = if token.pos.line == last_line {
            (0, token.pos.column - last_col_end)
        } else {
            (token.pos.line - last_line, token.pos.column)
        };
        last_line = token.pos.line;
        last_col_end = token.pos.column + (token.len as usize);
        let semantic_token = SemanticToken {
            delta_line: d_line as u32,
            delta_start: d_column as u32,
            length: token.len,
            token_type: token.token_type,
            token_modifiers_bitset: token.token_modifier,
        };
        semantic_tokens.push(semantic_token);
    }

    semantic_tokens
}

fn ast_tokens(ast: &Game) -> Vec<Token> {
    let mut tokens = Vec::new();
    for stat in ast.stats.iter() {
        match stat {
            Stat::Typedef(typedef) => {
                let token = Token {
                    pos: typedef.start(),
                    len: 4, // "type".len()
                    token_type: semantic_token_type(SemanticTokenType::KEYWORD),
                    token_modifier: 0,
                };
                tokens.push(token);
            }
            Stat::Variable(variable) => {
                let token = Token {
                    pos: variable.start(),
                    len: 3, // "var".len()
                    token_type: semantic_token_type(SemanticTokenType::KEYWORD),
                    token_modifier: 0,
                };
                tokens.push(token);
            }
            Stat::Constant(constant) => {
                let token = Token {
                    pos: constant.start(),
                    len: 8, // "constant".len()
                    token_type: semantic_token_type(SemanticTokenType::KEYWORD),
                    token_modifier: 0,
                };
                tokens.push(token);
            }
            Stat::Pragma(pragma) => {
                let len = match pragma.kind {
                    PragmaKind::Any => 3,      // "any".len()
                    PragmaKind::Disjoint => 8, // "disjoint".len()
                    PragmaKind::MultiAny => 9, // "multi_any".len()
                    PragmaKind::Unique => 6,   // "unique".len()
                };
                let token = Token {
                    pos: pragma.start(),
                    len,
                    token_type: semantic_token_type(SemanticTokenType::MACRO),
                    token_modifier: 0,
                };
                tokens.push(token);
            }
            Stat::Edge(_) => (),
        }
    }
    tokens
}

fn comment_tokens(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (line_idx, line) in text.lines().enumerate() {
        if let Some(col) = line.find("//") {
            let token = Token {
                pos: Position {
                    line: line_idx,
                    column: col,
                },
                len: line.len() as u32 - col as u32,
                token_type: semantic_token_type(SemanticTokenType::COMMENT),
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
        let symbol = symbol_table.get_occ_symbol(occ).unwrap();
        let definition_mod = if symbol.pos == occ.pos {
            semantic_token_modifier(SemanticTokenModifier::DEFINITION)
        } else {
            0
        };
        let token_type = match symbol.flag {
            Flag::Constant => semantic_token_type(SemanticTokenType::VARIABLE),
            Flag::Edge => semantic_token_type(SemanticTokenType::METHOD),
            Flag::Type => semantic_token_type(SemanticTokenType::TYPE),
            Flag::Variable => semantic_token_type(SemanticTokenType::VARIABLE),
            Flag::Member => semantic_token_type(SemanticTokenType::VARIABLE),
            Flag::Param => semantic_token_type(SemanticTokenType::PARAMETER),
        };
        let const_mod = if symbol.flag == Flag::Constant {
            semantic_token_modifier(SemanticTokenModifier::READONLY)
        } else {
            0
        };
        let token = Token {
            pos: occ.start(),
            len: symbol.id.len() as u32,
            token_type,
            token_modifier: definition_mod | const_mod,
        };
    }
    tokens
}
