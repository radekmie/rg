pub mod rg;
pub mod utils;

use crate::rg::ist_tools::Interner;
use crate::utils::map_id::MapId;
use js_sys::{Array, Function};
use nom::combinator::all_consuming;
use nom::error::convert_error;
use nom::Finish;
use rand::thread_rng;
use regex::{Captures, Regex};
use rg::ist::Game;
use rg::ist_tools::{perf, run};
use rg::parser::game_declaration;
use serde_json::from_str;
use std::iter::FromIterator;
use std::ops::Deref;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
pub fn parse_rg(source: &str) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    // Parsing comments would require far more complex grammar (and parser),
    // because a comment can occur basically everywhere.
    let comment_regex = Regex::new(r"(//.*?)(\n|$)").unwrap();
    let source = comment_regex.replace_all(source, |captures: &Captures| {
        captures
            .get(1)
            .map(|comment| {
                format!(
                    "{:indent$}{}",
                    "",
                    captures.get(2).map_or("", |newline| newline.as_str()),
                    indent = comment.as_str().len()
                )
            })
            .unwrap()
    });

    let result = match all_consuming(game_declaration)(&source).finish() {
        Ok((_, game_declaration)) => serde_json::to_string(game_declaration.deref())
            .map_err(|error| error.to_string().into()),
        Err(error) => Err(convert_error(source.deref(), error).into()),
    };

    result
}

#[wasm_bindgen]
pub fn perf_rg(source: &str, depth: usize, callback: &Function) {
    console_error_panic_hook::set_once();

    let mut interner = Interner::default();
    let game = from_str::<Game<&str>>(source)
        .expect("Incorrect IST file.")
        .map_id(&mut |id| interner.intern(id));

    let this = JsValue::null();
    perf(&game, depth, &|count| {
        callback.call1(&this, &count.into()).unwrap();
    });
}

#[wasm_bindgen]
pub fn run_rg(source: &str, plays: usize, callback: &Function) {
    console_error_panic_hook::set_once();

    let mut interner = Interner::default();
    let game = from_str::<Game<&str>>(source)
        .expect("Incorrect IST file.")
        .map_id(&mut |id| interner.intern(id));

    let this = JsValue::null();
    let mut rng = thread_rng();
    run(&game, &mut rng, plays, &|(plays, moves, turns, goals)| {
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
                                "    {}: {:6.2}%",
                                value.map_id(&mut |id| interner.recall(id).unwrap()),
                                *count as f32 * 1e2 / plays as f32
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .into(),
                ]),
            )
            .unwrap();
    });
}
