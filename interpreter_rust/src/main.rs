use interpreter_rust::rg::ist::Game;
use interpreter_rust::rg::ist_tools::{perf, run, Interner};
use interpreter_rust::utils::map_id::MapId;
use rand::thread_rng;
use serde_json::from_str;
use std::env::args;
use std::fs::read_to_string;
use std::time::Instant;

fn main() {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("Game IST file expected.");
    let source = read_to_string(file).expect("Couldn't open file.");

    let mut interner = Interner::default();
    let game = from_str::<Game<&str>>(source.as_str())
        .expect("Incorrect IST file.")
        .map_id(&mut |id| interner.intern(id));

    match args.get(2).expect("Operation expected.").as_str() {
        "perf" => {
            let depth = args
                .get(3)
                .map_or(10, |depth| depth.parse::<usize>().unwrap());

            for depth in 0..=depth {
                let start = Instant::now();
                perf(&game, depth, &|count| {
                    println!(
                        "perf(depth: {}) = {} in {:.3}ms",
                        depth,
                        count,
                        start.elapsed().as_nanos() as f32 / 1e6
                    )
                });
            }
        }
        "run" => {
            let mut rng = thread_rng();
            let plays = args
                .get(3)
                .map_or(10, |plays| plays.parse::<usize>().unwrap());

            let start = Instant::now();
            run(&game, &mut rng, plays, &|(plays, moves, turns)| {
                println!(
                    "{esc}c\
                     after {} plays:\n\
                       avg. moves: {:.3}\n\
                       avg. turns: {:.3}\n\
                       avg. times: {:.3}ms",
                    plays,
                    moves,
                    turns,
                    start.elapsed().as_nanos() as f32 / 1e6 / plays as f32,
                    esc = 27 as char
                );
            });
        }
        _ => panic!("Unknown operation."),
    }
}
