use js_sys::{Array, Function};
use map_id::MapId;
use nom::combinator::all_consuming;
use nom::error::convert_error;
use nom::Finish;
use rand::thread_rng;
use rg::ist::Game;
use rg::ist_tools::Interner;
use rg::parser::game_declaration;
use rg::{ast::GameDeclaration, ist::RuntimeId};
use rg_transform::{add_builtins, expand_generator_nodes, skip_self_assignments};
use serde_json::{from_str, to_string};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

pub fn prepare_ist(
    mut game_declaration: GameDeclaration<String>,
) -> (Game<RuntimeId>, Interner<RuntimeId>) {
    expand_generator_nodes(&mut game_declaration);
    let mut interner = Interner::default();
    let game = Game::from(game_declaration).map_id(&mut |id| interner.intern(id));
    (game, interner)
}

pub fn safe_parse_ast(ast: &str) -> Result<GameDeclaration<String>, String> {
    from_str::<GameDeclaration<String>>(ast).map_err(|error| error.to_string())
}

pub fn safe_parse_source(source: &str) -> Result<GameDeclaration<String>, String> {
    match all_consuming(game_declaration)(source).finish() {
        Ok((_, game_declaration)) => {
            let mut game_declaration = game_declaration.map_id(&mut |id| id.to_string());
            add_builtins(&mut game_declaration).map_err(|error| error.to_string())?;
            Ok(game_declaration)
        }
        Err(error) => Err(convert_error(source, error)),
    }
}

pub fn safe_serialize_ast(game_declaration: GameDeclaration<String>) -> Result<String, String> {
    to_string(&game_declaration).map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = parseRg)]
pub fn parse_rg(source: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    safe_serialize_ast(safe_parse_source(source)?)
}

#[wasm_bindgen(js_name = perfRg)]
pub fn perf_rg(ast: &str, depth: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let game = prepare_ist(safe_parse_ast(ast)?).0;
    let this = JsValue::null();
    game.perf(depth, &|count| {
        callback.call1(&this, &count.into()).unwrap();
    });

    Ok(())
}

#[wasm_bindgen(js_name = runRg)]
pub fn run_rg(ast: &str, plays: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let (game, interner) = prepare_ist(safe_parse_ast(ast)?);
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
    let game_declaration = safe_parse_ast(ast)?;
    Ok(format!("{game_declaration}"))
}

#[wasm_bindgen(js_name = transformExpandGeneratorNodes)]
pub fn transform_expand_generator_nodes(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let mut game_declaration = safe_parse_ast(ast)?;
    expand_generator_nodes(&mut game_declaration);
    safe_serialize_ast(game_declaration)
}

#[wasm_bindgen(js_name = transformSkipSelfAssignments)]
pub fn transform_skip_self_assignments(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let mut game_declaration = safe_parse_ast(ast)?;
    skip_self_assignments(&mut game_declaration);
    safe_serialize_ast(game_declaration)
}
