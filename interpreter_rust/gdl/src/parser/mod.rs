pub mod infix;
pub mod prefix;
mod utils;

use crate::ast::Game;
use nom::branch::alt;
use nom::combinator::all_consuming;
use nom::Parser;
use utils::Result;

pub fn game(input: &str) -> Result<'_, Game<&str>> {
    alt((all_consuming(infix::game), all_consuming(prefix::game))).parse(input)
}
