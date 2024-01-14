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
        .collect::<Vec<_>>()
        .join("\n");

    let mut interner: Interner<&str, u8> = Interner::default();
    let gdl = parser::game(&source)
        .unwrap()
        .1
        .map_id(&mut |id| interner.intern(id))
        .ground()
        .expand_ors(&interner.intern(&"or"))
        .eval_distinct(&interner.intern(&"distinct"))
        .simplify()
        .map_id(&mut |id| Arc::from(*interner.recall(id).unwrap()))
        .symbolify();
    println!("// {}", gdl.as_infix());
    let rg = gdl_to_rg(&gdl);
    print!("{rg}");
    Ok(())
}
