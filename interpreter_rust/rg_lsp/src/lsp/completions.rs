use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionResponse,
};

use crate::rg::{
    ast::*,
    position::*,
    symbol::{Flag, Symbol},
    symbol_table::SymbolTable,
};

use super::logger;

enum CompletionKind {
    Type,
    Variable, // const/var/member/param
    Value,    // type member
    Edge,
    Param,
    Keyword,
    Toplevel, // keyword + edge
    Any,
    None,
}

impl CompletionKind {
    fn predicate(&self) -> impl Fn(&&Symbol) -> bool {
        match self {
            CompletionKind::Any => |_: &&Symbol| true,
            CompletionKind::Type => |sym: &&Symbol| sym.flag == Flag::Type,
            CompletionKind::Value => |sym: &&Symbol| sym.flag == Flag::Member,
            CompletionKind::Edge => |sym: &&Symbol| sym.flag == Flag::Edge,
            CompletionKind::Toplevel => |sym: &&Symbol| sym.flag == Flag::Edge,
            CompletionKind::Param => |sym: &&Symbol| sym.flag == Flag::Param,
            CompletionKind::Variable => |sym: &&Symbol| {
                sym.flag == Flag::Variable
                    || sym.flag == Flag::Constant
                    || sym.flag == Flag::Member
                    || sym.flag == Flag::Param
            },
            _ => |_: &&Symbol| false,
        }
    }
}

pub fn capabilities() -> CompletionOptions {
    CompletionOptions {
        resolve_provider: Some(false),
        trigger_characters: None,
        all_commit_characters: None,
        work_done_progress_options: Default::default(),
        completion_item: None,
    }
}

pub fn completions(
    pos: Position,
    game: &Game,
    symbol_table: &SymbolTable,
) -> Option<CompletionResponse> {
    let items = completion_items(pos, game, symbol_table);
    if items.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(items))
    }
}

fn completion_items(pos: Position, game: &Game, symbol_table: &SymbolTable) -> Vec<CompletionItem> {
    match completion_kind(pos, game) {
        CompletionKind::None => vec![],
        CompletionKind::Keyword => keyword_completions(),
        CompletionKind::Toplevel => {
            let symbols = get_symbols(symbol_table, &completion_kind(pos, game).predicate());
            let mut items: Vec<CompletionItem> = symbols
                .iter()
                .map(|sym| completion_item(sym.id.clone(), Some(sym.flag.clone().into())))
                .collect();
            items.extend(keyword_completions());
            items
        }
        _ => {
            let symbols = get_symbols(symbol_table, &completion_kind(pos, game).predicate());
            symbols
                .iter()
                .map(|sym| completion_item(sym.id.clone(), Some(sym.flag.clone().into())))
                .collect()
        }
    }
}

fn get_symbols<'a>(
    symbol_table: &'a SymbolTable,
    predicate: &dyn Fn(&&Symbol) -> bool,
) -> Vec<&'a Symbol> {
    symbol_table.symbols.iter().filter(predicate).collect()
}

fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        completion_item("const".into(), Some(CompletionItemKind::KEYWORD)),
        completion_item("var".into(), Some(CompletionItemKind::KEYWORD)),
        completion_item("type".into(), Some(CompletionItemKind::KEYWORD)),
        completion_item("@any".into(), Some(CompletionItemKind::KEYWORD)),
        completion_item("@disjoint".into(), Some(CompletionItemKind::KEYWORD)),
        completion_item("@multiAny".into(), Some(CompletionItemKind::KEYWORD)),
        completion_item("@unique".into(), Some(CompletionItemKind::KEYWORD)),
    ]
}

fn completion_item(label: String, kind: Option<CompletionItemKind>) -> CompletionItem {
    CompletionItem {
        label,
        kind,
        ..CompletionItem::default()
    }
}

impl From<Flag> for CompletionItemKind {
    fn from(flag: Flag) -> Self {
        match flag {
            Flag::Type => CompletionItemKind::CLASS,
            Flag::Edge => CompletionItemKind::METHOD,
            Flag::Member => CompletionItemKind::FIELD,
            Flag::Constant => CompletionItemKind::CONSTANT,
            Flag::Variable => CompletionItemKind::VARIABLE,
            Flag::Param => CompletionItemKind::PROPERTY,
        }
    }
}

fn completion_kind(pos: Position, game: &Game) -> CompletionKind {
    let enclosing_stat = game.enclosing_stat_pos(pos);
    if let Some(stat) = enclosing_stat {
        logger::log(&format!("enclosing_stat: {}", stat).into());
        match stat {
            Stat::Constant(Constant {
                identifier,
                type_,
                value,
                ..
            }) => {
                if identifier.span().encloses_pos(&pos) {
                    CompletionKind::None
                } else if type_.span().encloses_pos(&pos) {
                    CompletionKind::Type
                } else if value.span().encloses_pos(&pos) {
                    CompletionKind::Value
                } else {
                    CompletionKind::Any
                }
            }
            Stat::Variable(Variable {
                default_value,
                identifier,
                type_,
                ..
            }) => {
                if identifier.span().encloses_pos(&pos) {
                    CompletionKind::None
                } else if type_.span().encloses_pos(&pos) {
                    CompletionKind::Type
                } else if default_value.span().encloses_pos(&pos) {
                    CompletionKind::Value
                } else {
                    CompletionKind::Any
                }
            }
            Stat::Edge(Edge {
                lhs, rhs, label, ..
            }) => {
                if lhs.span().encloses_pos(&pos) {
                    CompletionKind::Edge
                } else if rhs.span().encloses_pos(&pos) {
                    CompletionKind::Edge
                } else if label.span().encloses_pos(&pos) {
                    CompletionKind::Any
                } else {
                    CompletionKind::None
                }
            }
            Stat::Typedef(Typedef {
                identifier, type_, ..
            }) => {
                if identifier.span().encloses_pos(&pos) {
                    CompletionKind::None
                } else if type_.span().encloses_pos(&pos) {
                    CompletionKind::Type
                } else {
                    CompletionKind::Any
                }
            }
            Stat::Pragma(_) => CompletionKind::Edge,
        }
    } else {
        logger::log(&format!("enclosing_stat: None").into());
        CompletionKind::Toplevel
    }
}
