use gdl::parser;
use nom::Finish;
use std::io;

fn main() {
    let mut buffer = String::new();
    while io::stdin().read_line(&mut buffer).is_ok() {
        if buffer.chars().any(|c| c.is_alphanumeric()) {
            match parser::game(&buffer).finish() {
                Ok((_, game)) => {
                    println!("      prefix: {}", game.as_prefix());
                    println!("       infix: {}", game.as_infix());
                    let game = game.ground();
                    println!("    grounded: {}", game.as_infix());
                    let game = game.simplify();
                    println!("  simplified: {}", game.as_infix());
                }
                Err(error) => println!("\n\n[ERROR]\n{error}\n\n"),
            }
        }

        buffer.clear();
    }
}
