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
use serde_json::{from_str, to_string};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

pub fn prepare_ist(
    game: Game<String>,
) -> Result<(ist::Game<ist::RuntimeId>, Interner<ist::RuntimeId>), String> {
    let mut interner = Interner::default();
    let game =
        ist::Game::from(game.expand_generator_nodes()?).map_id(&mut |id| interner.intern(id));
    Ok((game, interner))
}

pub fn safe_parse_ast(ast: &str) -> Result<Game<String>, String> {
    from_str::<Game<String>>(ast).map_err(|error| error.to_string())
}

pub fn safe_parse_source(source: &str) -> Result<Game<String>, String> {
    match all_consuming(game)(source).finish() {
        Ok((_, game)) => game
            .map_id(&mut |id| id.to_string())
            .add_builtins()
            .map_err(|error| error.to_string()),
        Err(error) => Err(convert_error(source, error)),
    }
}

pub fn safe_serialize_ast(game: Game<String>) -> Result<String, String> {
    to_string(&game).map_err(|error| error.to_string())
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
        callback
            .apply(
                &this,
                &Array::from_iter::<[&JsValue; 4]>([
                    &plays.into(),
                    &moves.into(),
                    &turns.into(),
                    &goals
                        .iter()
                        .map(|(value, count)| {
                            format!(
                                "    {:5.2}% of {}",
                                *count as f32 * 1e2 / plays as f32,
                                value.map_id(&mut |id| interner.recall(id).unwrap())
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

#[wasm_bindgen(js_name = transformAddExplicitCasts)]
pub fn transform_add_explicit_casts(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_ast(safe_parse_ast(ast)?.add_explicit_casts()?)
}

#[wasm_bindgen(js_name = transformExpandGeneratorNodes)]
pub fn transform_expand_generator_nodes(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_ast(safe_parse_ast(ast)?.expand_generator_nodes()?)
}

#[wasm_bindgen(js_name = transformNormalizeTypes)]
pub fn transform_normalize_types(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_ast(safe_parse_ast(ast)?.normalize_types()?)
}

#[wasm_bindgen(js_name = transformSkipSelfAssignments)]
pub fn transform_skip_self_assignments(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_ast(safe_parse_ast(ast)?.skip_self_assignments()?)
}

#[wasm_bindgen(js_name = validateCheckReachabilities)]
pub fn validate_check_reachabilities(ast: &str) -> Result<(), String> {
    console_error_panic_hook::set_once();
    safe_parse_ast(ast)?.check_reachabilities()?;
    Ok(())
}

#[wasm_bindgen(js_name = validateCheckTypes)]
pub fn validate_check_types(ast: &str) -> Result<(), String> {
    console_error_panic_hook::set_once();
    safe_parse_ast(ast)?.check_types()?;
    Ok(())
}
