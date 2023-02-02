pub mod rg;
pub mod utils;

use nom::combinator::all_consuming;
use nom::error::convert_error;
use nom::Finish;
use regex::{Captures, Regex};
use rg::parser::game_declaration;
use std::ops::Deref;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
pub fn parse_rg(source: &str) -> Result<String, JsValue> {
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
