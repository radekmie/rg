mod ist;

use ist::tools::{new_ist_interner, ISTInterner};
use js_sys::{Array, Function};
use map_id::MapId;
use rand::thread_rng;
use rg::{ast::Game, parsing::parser::parse_with_errors};
use serde::Deserialize;
use serde_json::{from_str, to_string};
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

pub fn safe_parse_source(source: &str) -> Result<Game<Arc<str>>, String> {
    let (game, errors) = parse_with_errors(source);
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

pub fn safe_serialize_ast(game: &Game<Arc<str>>) -> Result<String, String> {
    to_string(game).map_err(|error| error.to_string())
}

#[derive(Deserialize)]
struct Flags {
    #[serde(rename = "addExplicitCasts")]
    add_explicit_casts: bool,
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
    #[serde(rename = "pruneUnreachableNodes")]
    prune_unreachable_nodes: bool,
    #[serde(rename = "skipSelfAssignments")]
    skip_self_assignments: bool,
}

#[wasm_bindgen(js_name = analyzeRg)]
pub fn analyze_rg(
    source: &str,
    flags: &str,
    join_fork_suffixes: &Function,
    inline_reachability: &Function,
) -> Result<Array, String> {
    let flags = from_str::<Flags>(flags).map_err(|error| error.to_string())?;

    let mut game = safe_parse_source(source)?;
    loop {
        game.check_maps()?;
        game.check_multiple_edges()?;
        game.check_reachabilities()?;
        game.check_types()?;

        let copy = game.clone();

        macro_rules! pass {
            ($fn:ident $block:block) => {
                if flags.$fn {
                    $block
                    if game != copy {
                        continue;
                    }
                }
            };
            (node $fn:ident) => {
                pass!($fn {
                    let ast = safe_serialize_ast(&game)?;
                    let ast = $fn.call1(&JsValue::null(), &ast.into()).unwrap().as_string().unwrap();
                    game = safe_parse_ast(&ast)?;
                });
            };
            (rust $fn:ident) => {
                pass!($fn {
                    game.$fn()?;
                });
            };
        }

        pass!(rust normalize_types);
        pass!(rust skip_self_assignments);
        pass!(rust compact_skip_edges);
        pass!(rust add_explicit_casts);
        pass!(rust expand_generator_nodes);
        pass!(node join_fork_suffixes);
        pass!(node inline_reachability);
        pass!(rust prune_unreachable_nodes);
        pass!(rust mangle_symbols);
        pass!(rust calculate_uniques);

        break;
    }

    let source_formatted = format!("{game}");
    assert_eq!(safe_parse_source(&source_formatted)?, game);
    Ok(Array::of2(
        &safe_serialize_ast(&game)?.into(),
        &source_formatted.into(),
    ))
}

#[wasm_bindgen(js_name = parseGdl)]
pub fn parse_gdl(source: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let gdl = gdl::parser::game(source)
        .map_err(|error| error.to_string())?
        .1;
    let rg = gdl_to_rg::gdl_to_rg(&gdl);
    Ok(rg.to_string())
}

#[wasm_bindgen(js_name = parseRg)]
pub fn parse_rg(source: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_ast(&safe_parse_source(source)?)
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
