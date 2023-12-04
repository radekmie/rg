use crate::rg::ast_features::AstFeatures;
use crate::rg::symbol::{Flag, Symbol};
use crate::rg::symbol_table::SymbolTable;
use rg::ast::*;
use rg::position::Position;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionOptions,
    CompletionOptionsCompletionItem, CompletionResponse,
};

#[derive(Debug, PartialEq)]
pub enum CompletionKind {
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
    let completion_kind = game
        .stat_enclosing_position(&pos)
        .map(|stat| stat.completion_kind(&pos))
        .unwrap_or(CompletionKind::Toplevel);
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
    static KEYWORDS: [&str; 7] = [
        "constr",
        "var",
        "type",
        "@any",
        "@unique",
        "@multiAny",
        "@disjoint",
    ];
    KEYWORDS
        .into_iter()
        .map(|label| CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
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

#[cfg(test)]
mod test {
    use rg::{parsing::parser::parse_with_errors, position::Position};

    use crate::rg::ast_features::AstFeatures;

    use super::CompletionKind;

    fn find_cursor(input: &str) -> (Position, String) {
        input
            .lines()
            .enumerate()
            .find_map(|(line_idx, line)| {
                line.find('^')
                    .map(|col| (Position::new(line_idx + 1, col + 1), input.replace('^', "")))
            })
            .expect("No cursor found")
    }

    fn completion_kind(input: &str, expected: CompletionKind) {
        let (pos, game) = find_cursor(input);
        let (game, _) = parse_with_errors(game.as_str());
        let obtained = game
            .stat_enclosing_position(&pos)
            .map(|stat| stat.completion_kind(&pos))
            .unwrap_or(CompletionKind::Toplevel);
        assert!(
            obtained == expected,
            "Failed on \n{}\nexpected: {:?}, obtained: {:?}\n",
            input,
            expected,
            obtained
        );
    }

    #[test]
    fn const_def() {
        completion_kind("const ^", CompletionKind::None);
        completion_kind("const foo: ^", CompletionKind::Type);
        completion_kind("const foo: ^ = 1;", CompletionKind::Type);
        completion_kind("const foo: Bar = ^  ", CompletionKind::Value);
        completion_kind("const foo: Bar = {:null, ^}", CompletionKind::Value);
        completion_kind("const foo: Bar = {:null, e1:^}", CompletionKind::Value);
    }

    #[test]
    fn edge_label() {
        completion_kind("begin, e1: ^", CompletionKind::Variable);
        completion_kind("begin, e1(param: Foo): ^", CompletionKind::Variable);
        completion_kind("begin, e1(param: Foo): foo[^]", CompletionKind::Variable);
        completion_kind(
            "begin, e1(param: Foo): Cast(^) == a",
            CompletionKind::Variable,
        );
        completion_kind(
            "begin, e1(param: Foo): Cast(a) == ^",
            CompletionKind::Variable,
        );
        completion_kind(
            "begin, e1(param: Foo): Cast(a) == a^",
            CompletionKind::Variable,
        );
        completion_kind("begin, e1(param: Foo): ^ == a", CompletionKind::Variable);
        completion_kind("begin, e1(param: Foo): ^ != a", CompletionKind::Variable);
        completion_kind("begin, e1(param: Foo): ^ != ;", CompletionKind::Variable);
        completion_kind("begin, e1(param: Foo): ! ^ ;", CompletionKind::Edge);
        completion_kind("begin, e1(param: Foo): ! foo -> ^ ;", CompletionKind::Edge);
        completion_kind("begin, e1(param: Foo): ! ^ -> foo ;", CompletionKind::Edge);
        completion_kind("begin, e1(param: Foo): a = ^ ;", CompletionKind::Variable);
        completion_kind(
            "begin, e1(param: Foo): a = foo^ ;",
            CompletionKind::Variable,
        );
    }

    #[test]
    fn edge_name() {
        completion_kind("begin, ^", CompletionKind::Toplevel);
        completion_kind("begin ^, e1", CompletionKind::Edge);
        completion_kind("begin(^), e1", CompletionKind::Param);
        completion_kind("begin(param: ^), ", CompletionKind::Type);
    }

    #[test]
    fn toplevel() {
        completion_kind("^", CompletionKind::Toplevel);
        completion_kind("const foo: Bar = {:null};\n^", CompletionKind::Toplevel);
    }

    #[test]
    fn type_def() {
        completion_kind("type ^ = Bar", CompletionKind::None);
        completion_kind("type Foo = ^", CompletionKind::Type);
        completion_kind("type Foo = ^ -> Bar;", CompletionKind::Type);
        completion_kind("type Foo = -> ^", CompletionKind::Type);
        completion_kind("type Foo = {null, ^ }", CompletionKind::None);
        completion_kind("type Foo = Bar ^ ;", CompletionKind::Type);
    }
}
