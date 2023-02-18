use nom::Finish;
use map_id::MapId;
use nom::combinator::all_consuming;
use nom::error::convert_error;
use rand::thread_rng;
use rg::ist::Game;
use rg::ist_tools::Interner;
use rg::parser::game_declaration;
use std::env::args;
use std::fs::read_to_string;

use std::time::Instant;

fn main() {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("game.rg file expected.");
    let source = read_to_string(file).expect("Couldn't open file.");
    let source = source.as_str();

    let game_declaration = match all_consuming(game_declaration)(source).finish() {
        Ok((_, game_declaration)) => game_declaration,
        Err(error) => panic!("{}", convert_error(source, error))
    };

    let mut interner = Interner::default();
    let game = Game::from(game_declaration).map_id(&mut |id| interner.intern(id));

    match args.get(2).expect("Operation expected.").as_str() {
        "perf" => {
            let depth = args
                .get(3)
                .map_or(10, |depth| depth.parse::<usize>().unwrap());

            for depth in 0..=depth {
                let start = Instant::now();
                game.perf(depth, &|count| {
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
            game.run(&mut rng, plays, &|(plays, moves, turns, goals)| {
                println!(
                    "{esc}c\
                     after {} plays:\n  \
                     avg. moves: {:.3}\n  \
                     avg. turns: {:.3}\n  \
                     avg. times: {:.3}ms\n  \
                     avg. goals:\n\
                     {}",
                    plays,
                    moves,
                    turns,
                    start.elapsed().as_nanos() as f32 / 1e6 / plays as f32,
                    goals
                        .iter()
                        .map(|(value, count)| format!(
                            "    {:5.2}% of {}",
                            *count as f32 * 1e2 / plays as f32,
                            value.map_id(&mut |id| interner.recall(id).unwrap())
                        ))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    esc = 27 as char
                );
            });
        }
        _ => panic!("Unknown operation."),
    }
}
