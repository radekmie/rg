use gdl::parser;
use map_id::MapId;
use nom::Finish;
use std::io;
use std::sync::Arc;

fn main() {
    let mut buffer = String::new();
    while io::stdin().read_line(&mut buffer).is_ok() {
        if buffer.chars().any(char::is_alphanumeric) {
            match parser::game(&buffer).finish() {
                Ok((_, game)) => {
                    let game = game.map_id(&mut |id| Arc::from(*id));
                    println!("        debug: {game:?}");
                    println!("       prefix: {}", game.as_prefix());
                    println!("        infix: {}", game.as_infix());
                    let game = game.ground();
                    println!("     grounded: {}", game.as_infix());
                    let game = game.simplify();
                    println!("   simplified: {}", game.as_infix());
                    let game = game.symbolify();
                    println!("  symbolified: {}", game.as_infix());
                }
                Err(error) => println!("\n\n[ERROR]\n{error}\n\n"),
            }
        }

        buffer.clear();
    }
}
