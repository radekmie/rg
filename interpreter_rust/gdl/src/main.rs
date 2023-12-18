use gdl::parser;
use nom::branch::alt;
use nom::combinator::all_consuming;
use nom::Finish;
use std::io;

fn main() {
    let mut buffer = String::new();
    while io::stdin().read_line(&mut buffer).is_ok() {
        if buffer.chars().all(|c| c.is_whitespace()) {
            break;
        }

        match all_consuming(alt((parser::infix::game, parser::prefix::game)))(&buffer).finish() {
            Ok((_, game)) => {
                print!(
                    "     infix: {}\n    prefix: {}\n  grounded: {}\n",
                    game.as_infix(),
                    game.as_prefix(),
                    game.ground().as_infix(),
                );
            }
            Err(error) => println!("\n\n[ERROR]\n{error}\n\n"),
        }

        buffer.clear();
    }
}
