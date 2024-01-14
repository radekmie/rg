use gdl::parser;
use gdl_to_rg::gdl_to_rg;
use std::env::args;
use std::fs::read_to_string;

fn main() -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("game.kif file expected.");
    let source = read_to_string(file).map_err(|error| error.to_string())?;
    let gdl = parser::game(&source).unwrap().1;
    println!("// {}", gdl.as_infix());
    let rg = gdl_to_rg(&gdl);
    print!("{rg}");
    Ok(())
}
