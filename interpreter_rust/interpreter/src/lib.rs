mod ist;

use hrg::{ast::GameDeclaration, parsing::parser::parse_with_errors as unsafe_parse_hrg};
use ist::tools::{new_ist_interner, ISTInterner};
use js_sys::{Array, Function};
use map_id::MapId;
use rand::thread_rng;
use rg::{ast::Game, parsing::parser::parse_with_errors as unsafe_parse_rg};
use serde::Deserialize;
use serde_json::{from_str, json, to_string};
use std::sync::Arc;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

pub fn prepare_ist(
    mut game: Game<Arc<str>>,
) -> Result<(ist::Game<ist::RuntimeId>, ISTInterner), String> {
    game.expand_generator_nodes()?;
    let mut interner = new_ist_interner();
    let game = ist::Game::from(game).map_id(&mut |id| interner.intern(id));
    Ok((game, interner))
}

pub fn safe_parse_ast(ast: &str) -> Result<Game<Arc<str>>, String> {
    from_str::<Game<Arc<str>>>(ast).map_err(|error| error.to_string())
}

pub fn safe_parse_hrg_source(source: &str) -> Result<GameDeclaration<Arc<str>>, String> {
    let (game, errors) = unsafe_parse_hrg(source);
    if errors.is_empty() {
        let game = game.map_id(&mut |id| Arc::from(id.identifier.as_str()));
        Ok(game)
    } else {
        Err(errors
            .into_iter()
            .map(|error| format!("{error}"))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

pub fn safe_parse_rg_source(source: &str) -> Result<Game<Arc<str>>, String> {
    let (game, errors) = unsafe_parse_rg(source);
    if errors.is_empty() {
        let mut game = game.map_id(&mut |id| Arc::from(id.identifier.as_str()));
        game.add_builtins()?;
        Ok(game)
    } else {
        Err(errors
            .into_iter()
            .map(|error| format!("{error}"))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

pub fn safe_serialize_hrg_ast(game: &GameDeclaration<Arc<str>>) -> Result<String, String> {
    to_string(game).map_err(|error| error.to_string())
}

pub fn safe_serialize_rg_ast(game: &Game<Arc<str>>) -> Result<String, String> {
    to_string(game).map_err(|error| error.to_string())
}

#[derive(Deserialize)]
struct Flags {
    #[serde(rename = "addExplicitCasts")]
    add_explicit_casts: bool,
    #[serde(rename = "calculateSimpleApply")]
    calculate_simple_apply: bool,
    #[serde(rename = "calculateTagIndexes")]
    calculate_tag_indexes: bool,
    #[serde(rename = "calculateUniques")]
    calculate_uniques: bool,
    #[serde(rename = "compactSkipEdges")]
    compact_skip_edges: bool,
    #[serde(rename = "expandGeneratorNodes")]
    expand_generator_nodes: bool,
    #[serde(rename = "inlineReachability")]
    inline_reachability: bool,
    #[serde(rename = "joinForkSuffixes")]
    join_fork_suffixes: bool,
    #[serde(rename = "mangleSymbols")]
    mangle_symbols: bool,
    #[serde(rename = "normalizeTypes")]
    normalize_types: bool,
    #[serde(rename = "pruneSingletonTypes")]
    prune_singleton_types: bool,
    #[serde(rename = "pruneUnreachableNodes")]
    prune_unreachable_nodes: bool,
    #[serde(rename = "skipSelfAssignments")]
    skip_self_assignments: bool,
    #[serde(rename = "skipSelfComparisons")]
    skip_self_comparisons: bool,
}

#[wasm_bindgen(js_name = analyzeRg)]
pub fn analyze_rg(
    source: &str,
    flags: &str,
    inline_reachability: &Function,
) -> Result<Array, String> {
    let mut steps = vec![];

    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            let step = json!({ $($json)+ });
            steps.push(to_string(&step).map_err(|error| error.to_string())?);
        }};
    }

    macro_rules! quit {
        () => {{
            let array = Array::new();
            for step in steps {
                array.push(&step.into());
            }

            return Ok(array);
        }};
    }

    macro_rules! fail {
        ($error:expr) => {{
            step!({ "kind": "error", "value": $error });
            quit!();
        }};
    }

    macro_rules! check {
        ($expr:expr) => {
            match $expr {
                Ok(expr) => expr,
                Err(error) => fail!(error.to_string()),
            }
        };
    }

    macro_rules! game_step {
        ($game:expr, $title:expr) => {{
            step!({ "kind": "source", "language": "rg", "value": $game.to_string(), "title": $title });
            step!({ "kind": "ast", "language": "rg", "value": $game, "title": $title });
        }};
    }

    let flags = check!(from_str::<Flags>(flags));
    let mut game = check!(safe_parse_rg_source(source));
    game_step!(game, "");

    loop {
        check!(game.check_maps());
        // check!(game.check_multiple_edges());
        check!(game.check_reachabilities());
        check!(game.check_types());

        let copy = game.clone();

        macro_rules! pass {
            ($fn:ident $block:block) => {
                if flags.$fn {
                    $block
                    if game != copy {
                        game_step!(game, stringify!($fn));
                        continue;
                    }
                }
            };
            (node $fn:ident) => {
                pass!($fn {
                    let ast = check!(safe_serialize_rg_ast(&game));
                    let ast = $fn.call1(&JsValue::null(), &ast.into()).unwrap().as_string().unwrap();
                    game = check!(safe_parse_ast(&ast));
                });
            };
            (rust $fn:ident) => {
                pass!($fn {
                    check!(game.$fn());
                });
            };
        }

        pass!(rust normalize_types);
        pass!(rust skip_self_assignments);
        pass!(rust skip_self_comparisons);
        pass!(rust compact_skip_edges);
        pass!(rust add_explicit_casts);
        pass!(rust expand_generator_nodes);
        pass!(rust join_fork_suffixes);
        pass!(node inline_reachability);
        pass!(rust prune_singleton_types);
        pass!(rust prune_unreachable_nodes);
        pass!(rust mangle_symbols);
        pass!(rust calculate_simple_apply);
        pass!(rust calculate_tag_indexes);
        pass!(rust calculate_uniques);

        break;
    }

    assert_eq!(check!(safe_parse_rg_source(&game.to_string())), game);
    quit!()
}

#[wasm_bindgen(js_name = parseGdl)]
pub fn parse_gdl(source: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let gdl = gdl::parser::game(source)
        .map_err(|error| error.to_string())?
        .1;
    let rg = gdl_to_rg::gdl_to_rg(&gdl);
    safe_serialize_rg_ast(&rg)
}

#[wasm_bindgen(js_name = parseHrg)]
pub fn parse_hrg(source: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_hrg_ast(&safe_parse_hrg_source(source)?)
}

#[wasm_bindgen(js_name = perfRg)]
pub fn perf_rg(ast: &str, depth: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let game = prepare_ist(safe_parse_ast(ast)?)?.0;
    let this = JsValue::null();
    game.perf(depth, &|count| {
        callback.call1(&this, &count.into()).unwrap();
    });

    Ok(())
}

#[wasm_bindgen(js_name = runRg)]
pub fn run_rg(ast: &str, plays: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let (game, interner) = prepare_ist(safe_parse_ast(ast)?)?;
    let this = JsValue::null();
    let mut rng = thread_rng();
    game.run(&mut rng, plays, &|(plays, moves, turns, goals)| {
        let mut sorted_goals = goals
            .iter()
            .map(|(value, count)| (value.map_id(&mut |id| interner.recall(id).unwrap()), count))
            .collect::<Vec<_>>();
        sorted_goals.sort();
        callback
            .apply(
                &this,
                &Array::from_iter::<[&JsValue; 4]>([
                    &plays.into(),
                    &moves.into(),
                    &turns.into(),
                    &sorted_goals
                        .into_iter()
                        .map(|(value, count)| {
                            format!(
                                "    {:5.2}% of {}",
                                *count as f32 * 1e2 / plays as f32,
                                value
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .into(),
                ]),
            )
            .unwrap();
    });

    Ok(())
}

#[wasm_bindgen(js_name = serializeRg)]
pub fn serialize_rg(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let game = safe_parse_ast(ast)?;
    Ok(game.to_string())
}
