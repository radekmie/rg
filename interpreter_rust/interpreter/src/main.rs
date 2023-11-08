use interpreter::{prepare_ist, safe_parse_source};
use rg_lsp::rg::parser::{parse, parse_with_errors};
use map_id::MapId;
use rand::thread_rng;
use rg_lsp::rg::symbol_table::SymbolTable;
use std::env::args;
use std::fs::read_to_string;
use std::time::Instant;

fn main() -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("game.rg file expected.");
    let source = read_to_string(file).map_err(|error| error.to_string())?;
    let (game, errors) = parse_with_errors(source.as_str());
    
    println!(" LSP TREE ");
    println!("{}",game);
    println!(" ERRORS ");
    for err in errors {
        println!("{}", err);
    }

    Ok(())
}
