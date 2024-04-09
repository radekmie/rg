use crate::{
    common::{
        symbol::{Flag, Symbol},
        symbol_table::SymbolTable,
    },
    document::Ast,
    rg::ast_features::AstFeatures,
};
use rg::ast::Game;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionOptions,
    CompletionOptionsCompletionItem, CompletionResponse, WorkDoneProgressOptions,
};
use utils::{position::Position, Identifier};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    fn predicate(self) -> impl Fn(&&Symbol) -> bool {
        match self {
            Self::Any => |_: &&Symbol| true,
            Self::Edge => |sym: &&Symbol| sym.flag == Flag::Edge,
            Self::None => |_: &&Symbol| false,
            Self::Param => |sym: &&Symbol| sym.flag == Flag::Param,
            Self::Toplevel => |sym: &&Symbol| sym.flag == Flag::Edge,
            Self::Type => |sym: &&Symbol| sym.flag == Flag::Type,
            Self::Value => |sym: &&Symbol| sym.flag == Flag::Member,
            Self::Variable => |sym: &&Symbol| sym.flag != Flag::Edge,
        }
    }
}

pub fn capabilities() -> CompletionOptions {
    CompletionOptions {
        resolve_provider: Some(false),
        trigger_characters: None,
        all_commit_characters: None,
        work_done_progress_options: WorkDoneProgressOptions::default(),
        completion_item: Some(CompletionOptionsCompletionItem {
            label_details_support: Some(true),
        }),
    }
}

pub fn completions(
    pos: Position,
    game: &Ast,
    symbol_table: &SymbolTable,
) -> Option<CompletionResponse> {
    match game {
        Ast::Rg(game) => {
            let items = completion_items(pos, game, symbol_table);
            if items.is_empty() {
                None
            } else {
                Some(CompletionResponse::Array(items))
            }
        }
        Ast::Hrg(_) => None,
    }
}

fn completion_items(
    pos: Position,
    game: &Game<Identifier>,
    symbol_table: &SymbolTable,
) -> Vec<CompletionItem> {
    let completion_kind = game
        .stat_enclosing_position(&pos)
        .map_or(CompletionKind::Toplevel, |stat| stat.completion_kind(&pos));
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
    static KEYWORDS: [&str; 9] = [
        "const",
        "var",
        "type",
        "@distinct",
        "@repeat",
        "@simpleApply",
        "@tagIndex",
        "@tagMaxIndex",
        "@unique",
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
            detail: Some(format!(" : {t}")),
            ..Default::default()
        }),
        ..Default::default()
    }
}

impl From<Flag> for CompletionItemKind {
    fn from(flag: Flag) -> Self {
        match flag {
            Flag::Type => Self::CLASS,
            Flag::Edge => Self::METHOD,
            Flag::Member => Self::FIELD,
            Flag::Constant => Self::CONSTANT,
            Flag::Variable => Self::VARIABLE,
            Flag::Param => Self::PROPERTY,
        }
    }
}

#[cfg(test)]
mod test {
    use rg::parsing::parser::parse_with_errors;
    use utils::position::Position;

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
            .map_or(CompletionKind::Toplevel, |stat| stat.completion_kind(&pos));
        assert!(
            obtained == expected,
            "Failed on \n{input}\nexpected: {expected:?}, obtained: {obtained:?}\n"
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
    fn label() {
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
    fn node() {
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
