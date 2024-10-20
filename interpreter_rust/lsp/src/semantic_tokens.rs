use crate::common::symbol_table::SymbolTable;
use crate::common::utils::ToLspPosition;
use crate::document::Ast;
use crate::rg::ast_features::AstFeatures;
use crate::{common::symbol::Flag, document::Document};
use rg::ast::Game;
use tower_lsp::lsp_types::{
    Position as LspPosition, SemanticToken, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
    WorkDoneProgressOptions,
};
use utils::position::Positioned;
use utils::Identifier;

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

fn semantic_token_modifier(token_modifier: &SemanticTokenModifier) -> u32 {
    SEMANTIC_TOKENS_MODIFIERS
        .iter()
        .position(|t| t == token_modifier)
        .expect("Unknown token modifier") as u32
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Token {
    pos: LspPosition,
    len: u32,
    type_: u32,
    modifier_bitset: u32,
}

#[derive(Debug, Clone, Default)]
struct Delta {
    last_line: u32,
    last_col_end: u32,
    line: u32,
    column: u32,
}

impl Delta {
    fn step(&mut self, pos: LspPosition) {
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
    let keywords = match &document.tree {
        Ast::Rg(game) => ast_tokens(game),
        Ast::Hrg => vec![],
    };
    let symbols = symbol_table_tokens(&document.symbol_table);
    let mut tokens = [&keywords[..], &symbols[..]].concat();
    tokens.sort_by_key(|t| t.pos);
    let mut delta = Delta::default();
    tokens
        .into_iter()
        .map(|token| {
            delta.step(token.pos);
            SemanticToken {
                delta_line: delta.line,
                delta_start: delta.column,
                length: token.len,
                token_type: token.type_,
                token_modifiers_bitset: token.modifier_bitset,
            }
        })
        .collect()
}

fn ast_tokens(game: &Game<Identifier>) -> Vec<Token> {
    let mut tokens = Vec::new();
    game.stats().into_iter().for_each(|stat| {
        let keyword = stat.keyword();
        if !keyword.is_empty() {
            let token_type = semantic_token_type(stat.token_type());
            let token = Token {
                pos: stat.span().start.to_lsp(),
                len: keyword.len() as u32,
                type_: token_type,
                modifier_bitset: 0,
            };
            tokens.push(token);
        }
    });
    tokens
}

fn symbol_table_tokens(symbol_table: &SymbolTable) -> Vec<Token> {
    let mut tokens = Vec::new();
    for occ in &symbol_table.occurrences {
        if let Some((_, symbol)) = symbol_table.get_occ_symbol(occ) {
            let definition_mod = if symbol.pos.equal_span(&occ.pos) {
                semantic_token_modifier(&SemanticTokenModifier::DEFINITION)
            } else {
                0
            };
            let token_type = match symbol.flag {
                Flag::Constant => semantic_token_type("constant"),
                Flag::Function => semantic_token_type("function"),
                Flag::Type => semantic_token_type("type"),
                Flag::Variable => semantic_token_type("variable"),
                Flag::Member => semantic_token_type("member"),
                Flag::Param => semantic_token_type("parameter"),
            };
            let const_mod = if symbol.flag == Flag::Constant {
                semantic_token_modifier(&SemanticTokenModifier::READONLY)
            } else {
                0
            };
            let token = Token {
                pos: occ.start().to_lsp(),
                len: symbol.id.len() as u32,
                type_: token_type,
                modifier_bitset: definition_mod | const_mod,
            };
            tokens.push(token);
        }
    }
    tokens
}
