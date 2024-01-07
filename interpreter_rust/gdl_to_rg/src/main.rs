use gdl::parser;
use gdl_to_rg::gdl_to_rg;
use map_id::MapId;
use std::env::args;
use std::fs::read_to_string;
use std::sync::Arc;

fn main() -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("game.kif file expected.");
    let source = read_to_string(file)
        .map_err(|error| error.to_string())?
        .lines()
        .filter(|line| !line.starts_with(';'))
        .collect::<String>();
    let gdl = parser::game(&source)
        .unwrap()
        .1
        .map_id(&mut |id| Arc::from(*id))
        .ground()
        .expand_ors()
        .eval_distinct()
        .simplify()
        .symbolify();
    let rg = gdl_to_rg(&gdl);
    print!("{rg}");
    Ok(())
}
