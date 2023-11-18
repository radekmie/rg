use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionOptions,
    CompletionOptionsCompletionItem, CompletionResponse,
};

use rg::{ast::*, position::*};

use crate::rg::{
    stat::Stat,
    symbol::{Flag, Symbol},
    symbol_table::SymbolTable,
};

use super::ast_features::AstFeatures;

#[derive(Debug, PartialEq)]
enum CompletionKind {
    Type,
    Variable, // const/var/member/param
    Value,    // type member
    Edge,
    Param,
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
            CompletionKind::Variable => |sym: &&Symbol| sym.flag != Flag::Edge,
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
        completion_item: Some(CompletionOptionsCompletionItem {
            label_details_support: Some(true),
        }),
    }
}

pub fn completions(
    pos: Position,
    game: &Game<Identifier>,
    symbol_table: &SymbolTable,
) -> Option<CompletionResponse> {
    let items = completion_items(pos, game, symbol_table);
    if items.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(items))
    }
}

fn completion_items(
    pos: Position,
    game: &Game<Identifier>,
    symbol_table: &SymbolTable,
) -> Vec<CompletionItem> {
    let completion_kind = CompletionKind::from_game(pos, game);
    let mut items = match completion_kind {
        CompletionKind::None => vec![],
        CompletionKind::Toplevel => {
            let symbols = get_symbols(symbol_table, &CompletionKind::Toplevel.predicate());
            let mut items: Vec<CompletionItem> = symbols
                .into_iter()
                .map(|sym| completion_item(game, sym))
                .collect();
            items.extend(keyword_completions());
            items
        }
        kind => {
            let symbols = get_symbols(symbol_table, &kind.predicate());
            symbols
                .into_iter()
                .map(|sym| completion_item(game, sym))
                .collect()
        }
    };
    items.dedup();
    items
}

fn get_symbols<'a>(
    symbol_table: &'a SymbolTable,
    predicate: &dyn Fn(&&Symbol) -> bool,
) -> Vec<&'a Symbol> {
    symbol_table.symbols.iter().filter(predicate).collect()
}

fn keyword_completions() -> Vec<CompletionItem> {
    vec![
        ("const", Some(CompletionItemKind::KEYWORD)),
        ("var", Some(CompletionItemKind::KEYWORD)),
        ("type", Some(CompletionItemKind::KEYWORD)),
        ("@any", Some(CompletionItemKind::KEYWORD)),
        ("@disjoint", Some(CompletionItemKind::KEYWORD)),
        ("@multiAny", Some(CompletionItemKind::KEYWORD)),
        ("@unique", Some(CompletionItemKind::KEYWORD)),
    ]
    .into_iter()
    .map(|(label, kind)| CompletionItem {
        label: label.to_string(),
        kind,
        ..Default::default()
    })
    .collect()
}

fn completion_item(game: &Game<Identifier>, symbol: &Symbol) -> CompletionItem {
    let type_ = if symbol.flag == Flag::Type {
        None
    } else {
        game.symbol_type(symbol)
    };
    CompletionItem {
        label: symbol.id.clone(),
        kind: Some(symbol.flag.clone().into()),
        label_details: type_.map(|t| CompletionItemLabelDetails {
            detail: Some(format!(" : {}", t)),
            ..Default::default()
        }),
        ..Default::default()
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

impl CompletionKind {
    fn from_game(pos: Position, game: &Game<Identifier>) -> Self {
        let enclosing_stat = game.enclosing_stat_pos(pos);
        if let Some(stat) = enclosing_stat {
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
                        Self::from_edge_name(pos, lhs)
                    } else if rhs.span().encloses_pos(&pos) {
                        Self::from_edge_name(pos, rhs)
                    } else if label.span().encloses_pos(&pos) {
                        match label {
                            EdgeLabel::Assignment { .. } => CompletionKind::Variable,
                            EdgeLabel::Comparison { .. } => CompletionKind::Variable,
                            EdgeLabel::Reachability { .. } => CompletionKind::Edge,
                            EdgeLabel::Skip { .. } => CompletionKind::Variable,
                            EdgeLabel::Tag { .. } => CompletionKind::Param,
                        }
                    } else {
                        CompletionKind::Edge
                    }
                }
                Stat::Typedef(Typedef {
                    identifier, type_, ..
                }) => {
                    if identifier.span().encloses_pos(&pos) {
                        CompletionKind::None
                    } else if type_.span().encloses_pos(&pos) {
                        match type_.as_ref() {
                            Type::Set { .. } => CompletionKind::None,
                            _ => CompletionKind::Type,
                        }
                    } else {
                        CompletionKind::Any
                    }
                }
                Stat::Pragma(pragma) => Self::from_edge_name(pos, pragma.edge_name()),
            }
        } else {
            CompletionKind::Toplevel
        }
    }

    fn from_edge_name(pos: Position, edge_name: &EdgeName<Identifier>) -> Self {
        edge_name
            .parts
            .iter()
            .find_map(|part| {
                if part.span().encloses_pos(&pos) {
                    match part {
                        EdgeNamePart::Literal { .. } => Some(CompletionKind::Edge),
                        EdgeNamePart::Binding {
                            type_, identifier, ..
                        } => {
                            if identifier.span().encloses_pos(&pos)
                                || !type_.span().start.is_after(&identifier.span().end)
                            {
                                Some(CompletionKind::Param)
                            } else {
                                Some(CompletionKind::Type)
                            }
                        }
                    }
                } else {
                    None
                }
            })
            .or_else(|| Some(CompletionKind::Edge))
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use rg::{parser::parse_with_errors, position::Position};

    use super::CompletionKind;

    fn find_cursor(input: &str) -> (Position, String) {
        input
            .lines()
            .enumerate()
            .find_map(|(line_idx, line)| {
                if let Some(col) = line.find('^') {
                    Some((Position::new(line_idx + 1, col + 1), input.replace('^', "")))
                } else {
                    None
                }
            })
            .expect("No cursor found")
    }

    fn completion_kind(input: &str, expected: CompletionKind) {
        let (pos, input) = find_cursor(input);
        let (game, _) = parse_with_errors(input.as_str());
        let obtained = CompletionKind::from_game(pos, &game);
        assert!(
            obtained == expected,
            "\nexpected: {:?}, obtained: {:?}\n",
            expected,
            obtained
        );
    }

    #[test]
    fn const_def() {
        completion_kind("const ^", CompletionKind::None);
    }

    #[test]
    fn const_def1() {
        completion_kind("const foo: ^", CompletionKind::Type);
    }

    #[test]
    fn const_def2() {
        completion_kind("const foo: ^ = 1;", CompletionKind::Type);
    }

    #[test]
    fn const_def3() {
        completion_kind("const foo: Bar = ^  ", CompletionKind::Value);
    }

    #[test]
    fn const_def4() {
        completion_kind("const foo: Bar = {:null, ^}", CompletionKind::Value);
    }

    #[test]
    fn const_def5() {
        completion_kind("const foo: Bar = {:null, e1:^}", CompletionKind::Value);
    }

    #[test]
    fn edge_label() {
        completion_kind("begin, e1: ^", CompletionKind::Variable);
    }

    #[test]
    fn edge_label1() {
        completion_kind("begin, e1(param: Foo): ^", CompletionKind::Variable);
    }

    #[test]
    fn edge_label2() {
        completion_kind("begin, e1(param: Foo): foo[^]", CompletionKind::Variable);
    }

    #[test]
    fn edge_label3() {
        completion_kind(
            "begin, e1(param: Foo): Cast(^) == a",
            CompletionKind::Variable,
        );
    }

    #[test]
    fn edge_label4() {
        completion_kind(
            "begin, e1(param: Foo): Cast(a) == ^",
            CompletionKind::Variable,
        );
    }

    #[test]
    fn edge_label5() {
        completion_kind(
            "begin, e1(param: Foo): Cast(a) == a^",
            CompletionKind::Variable,
        );
    }

    #[test]
    fn edge_label6() {
        completion_kind("begin, e1(param: Foo): ^ == a", CompletionKind::Variable);
    }

    #[test]
    fn edge_label7() {
        completion_kind("begin, e1(param: Foo): ^ != a", CompletionKind::Variable);
    }

    #[test]
    fn edge_label8() {
        completion_kind("begin, e1(param: Foo): ^ != ;", CompletionKind::Variable);
    }

    #[test]
    fn edge_label9() {
        completion_kind("begin, e1(param: Foo): ! ^ ;", CompletionKind::Edge);
    }

    #[test]
    fn edge_label10() {
        completion_kind("begin, e1(param: Foo): ! foo -> ^ ;", CompletionKind::Edge);
    }

    #[test]
    fn edge_label11() {
        completion_kind("begin, e1(param: Foo): ! ^ -> foo ;", CompletionKind::Edge);
    }

    #[test]
    fn edge_label12() {
        completion_kind("begin, e1(param: Foo): a = ^ ;", CompletionKind::Variable);
    }

    #[test]
    fn edge_label13() {
        completion_kind(
            "begin, e1(param: Foo): a = foo^ ;",
            CompletionKind::Variable,
        );
    }

    #[test]
    fn edge_name() {
        completion_kind("begin, ^", CompletionKind::Edge);
    }

    #[test]
    fn edge_name1() {
        completion_kind("begin ^, e1", CompletionKind::Edge)
    }

    #[test]
    fn edge_name2() {
        completion_kind("begin(^), e1", CompletionKind::Param);
    }

    #[test]
    fn edge_name3() {
        completion_kind("begin(param: ^), ", CompletionKind::Type);
    }

    #[test]
    fn toplevel() {
        completion_kind("^", CompletionKind::Toplevel);
    }

    #[test]
    fn toplevel1() {
        completion_kind("const foo: Bar = {:null}\n^", CompletionKind::Toplevel);
    }

    #[test]
    fn type_def() {
        completion_kind("type ^ = Bar", CompletionKind::None);
    }

    #[test]
    fn type_def1() {
        completion_kind("type Foo = ^", CompletionKind::Type);
    }

    #[test]
    fn type_def2() {
        completion_kind("type Foo = ^ -> Bar;", CompletionKind::Type);
    }

    #[test]
    fn type_def3() {
        completion_kind("type Foo = -> ^", CompletionKind::Type);
    }

    #[test]
    fn type_def4() {
        completion_kind("type Foo = {null, ^ }", CompletionKind::None);
    }
}
