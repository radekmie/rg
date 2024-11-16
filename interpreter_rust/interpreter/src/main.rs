use interpreter::{analyze_rg_inner, prepare_ist, Flags};
use rand::thread_rng;
use std::env::args;
use std::fs::read_to_string;

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
                game.perf(depth, &|count, time| {
                    println!("perf(depth: {depth}) = {count} in {time:.3}ms",);
                });
            }
        }
        "run" => {
            let mut rng = thread_rng();
            let plays = args
                .get(3)
                .map_or(10, |plays| plays.parse::<usize>().unwrap());

            game.run(
                &mut rng,
                &interner,
                plays,
                &Some(|lines: Vec<_>| {
                    println!("{esc}c{}", lines.join("\n"), esc = 27 as char);
                }),
            );
        }
        _ => panic!("Unknown operation."),
    }

    Ok(())
}
