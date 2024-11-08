use interpreter::{analyze_rg_inner, prepare_ist, Flags};
use rand::thread_rng;
use std::env::args;
use std::fs::read_to_string;
use std::time::Instant;

fn main() -> Result<(), String> {
    let args = args().collect::<Vec<_>>();
    let file = args.get(1).expect("game.rg file expected.");
    let source = read_to_string(file).map_err(|error| error.to_string())?;
    let game = analyze_rg_inner(source.as_str(), &Flags::none(), None::<fn(_)>)?;
    let (game, interner) = prepare_ist(game)?;

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
                    );
                });
            }
        }
        "run" => {
            let mut rng = thread_rng();
            let plays = args
                .get(3)
                .map_or(10, |plays| plays.parse::<usize>().unwrap());

            game.run(&mut rng, &interner, plays, &|lines| {
                println!("{esc}c{}", lines.join("\n"), esc = 27 as char);
            });
        }
        _ => panic!("Unknown operation."),
    }

    Ok(())
}
