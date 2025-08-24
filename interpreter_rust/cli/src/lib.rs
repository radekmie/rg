mod analyze;
mod flags;

pub use crate::analyze::analyze;
pub use crate::flags::Flags;

use js_sys::{Array, Function};
use map_id::MapId;
use rand::thread_rng;
use rg_interpreter::Game;
use serde_json::{from_str, json, to_string};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen(js_name = analyze)]
pub fn wasm_analyze(
    source: String,
    extension: &str,
    flags: &str,
    callback: &Function,
) -> Result<(), String> {
    console_error_panic_hook::set_once();
    analyze(
        source,
        extension,
        &from_str::<Flags>(flags).unwrap(),
        false,
        &mut Some(|step| {
            callback
                .call1(&JsValue::null(), &JsValue::from(step))
                .unwrap();
        }),
    )?;
    Ok(())
}

#[wasm_bindgen(js_name = apply)]
pub fn wasm_apply(ast: &str, path: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let (game, interner, variables_indexes) =
        Game::try_from(from_str(ast).map_err(|error| error.to_string())?)?;
    let state = game.initial_state_after(&interner, path)?;
    let moves = state
        .next_states(&game, true)
        .map(|state| {
            state
                .tags
                .iter()
                .map(|tag| interner.recall(tag).unwrap().as_ref())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>();

    let is_final = state.is_final();
    let state = format!(
        "goals: {}\nplayer: {}\nposition: {}\nvalues:\n  {}\nvisible: {}",
        state.goals.map_id(&mut |id| interner.recall(id).unwrap()),
        state.player.map_id(&mut |id| interner.recall(id).unwrap()),
        interner.recall(&state.position).unwrap(),
        state
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| format!(
                "{}: {}",
                variables_indexes
                    .iter()
                    .find(|variable| *variable.1 == index)
                    .unwrap()
                    .0,
                value.map_id(&mut |id| interner.recall(id).unwrap())
            ))
            .collect::<Vec<_>>()
            .join("\n  "),
        state.visible.map_id(&mut |id| interner.recall(id).unwrap())
    );

    let result = json!({ "isFinal": is_final, "moves": moves, "state": state });
    to_string(&result).map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = perf)]
pub fn wasm_perf(
    ast: &str,
    initial_state_path: &str,
    depth: usize,
    callback: &Function,
) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let (game, interner, _) = Game::try_from(from_str(ast).map_err(|error| error.to_string())?)?;
    let initial_state = game.initial_state_after(&interner, initial_state_path)?;
    let (count, time) = game.perf(&initial_state, depth);
    callback
        .call2(&JsValue::null(), &count.into(), &time.into())
        .unwrap();
    Ok(())
}

#[wasm_bindgen(js_name = run)]
pub fn wasm_run(
    ast: &str,
    initial_state_path: &str,
    plays: usize,
    callback: &Function,
) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let (game, interner, _) = Game::try_from(from_str(ast).map_err(|error| error.to_string())?)?;
    let initial_state = game.initial_state_after(&interner, initial_state_path)?;
    let this = JsValue::null();
    let mut rng = thread_rng();
    game.run(
        &mut rng,
        &interner,
        &initial_state,
        plays,
        &Some(|lines: Vec<_>| {
            let lines = Array::from_iter(lines.into_iter().map(JsValue::from));
            callback.call1(&this, &lines).unwrap();
        }),
    )
}
