use gdl::parser;
use gdl_to_rg::gdl_to_rg;
use map_id::MapId;
use std::env::args;
use std::fs::read_to_string;
use std::sync::Arc;
use utils::Interner;

fn main() -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("game.kif file expected.");
    let source = read_to_string(file)
        .map_err(|error| error.to_string())?
        .lines()
        .filter(|line| !line.starts_with(';'))
        .collect::<String>();

    let mut interner: Interner<&str, u8> = Interner::default();
    let gdl = parser::game(&source)
        .unwrap()
        .1
        .map_id(&mut |id| interner.intern(id))
        .ground()
        .map_id(&mut |id| *interner.recall(id).unwrap())
        .expand_ors()
        .eval_distinct()
        .simplify()
        .map_id(&mut |id| Arc::from(*id))
        .symbolify();
    let rg = gdl_to_rg(&gdl);
    print!("{rg}");
    Ok(())
}
