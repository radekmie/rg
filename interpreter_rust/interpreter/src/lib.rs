use js_sys::{Array, Function};
use map_id::MapId;
use nom::combinator::all_consuming;
use nom::error::convert_error;
use nom::Finish;
use rand::thread_rng;
use rg::ast::Game;
use rg::ist;
use rg::ist_tools::Interner;
use rg::parser::game;
use serde::Deserialize;
use serde_json::{from_str, to_string};
use std::rc::Rc;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

pub fn prepare_ist(
    mut game: Game<Rc<str>>,
) -> Result<(ist::Game<ist::RuntimeId>, Interner<ist::RuntimeId>), String> {
    game.expand_generator_nodes()?;
    let mut interner = Interner::default();
    let game = ist::Game::from(game).map_id(&mut |id| interner.intern(id));
    Ok((game, interner))
}

pub fn safe_parse_ast(ast: &str) -> Result<Game<Rc<str>>, String> {
    from_str::<Game<Rc<str>>>(ast).map_err(|error| error.to_string())
}

pub fn safe_parse_source(source: &str) -> Result<Game<Rc<str>>, String> {
    match all_consuming(game)(source).finish() {
        Ok((_, game)) => {
            let mut game = game.map_id(&mut |id| Rc::from(*id));
            game.add_builtins()?;
            Ok(game)
        }
        Err(error) => Err(convert_error(source, error)),
    }
}

pub fn safe_serialize_ast(game: Game<Rc<str>>) -> Result<String, String> {
    to_string(&game).map_err(|error| error.to_string())
}

#[derive(Deserialize)]
struct Flags {
    #[serde(rename = "addExplicitCasts")]
    add_explicit_casts: bool,
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
                    let ast = safe_serialize_ast(game)?;
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
        pass!(rust mangle_symbols);

        break;
    }

    let source_formatted = format!("{game}");
    assert_eq!(safe_parse_source(&source_formatted)?, game);
    Ok(Array::of2(
        &safe_serialize_ast(game)?.into(),
        &source_formatted.into(),
    ))
}

#[wasm_bindgen(js_name = parseRg)]
pub fn parse_rg(source: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_ast(safe_parse_source(source)?)
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
    Ok(format!("{game}"))
}
