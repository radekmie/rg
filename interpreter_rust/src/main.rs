use interpreter_rust::*;
use rand::{rngs::ThreadRng, seq::IteratorRandom};
use std::{env, time::Instant};

fn avg(numbers: &[f32]) -> f32 {
    numbers.iter().sum::<f32>() / numbers.len() as f32
}

fn run(game: &Game, rng: &mut ThreadRng, plays: usize) {
    let mut moves = vec![];
    let mut times = vec![];
    let mut turns = vec![];

    for play in 1..=plays {
        let start = Instant::now();
        let mut state = State::initial(game);
        let mut turn = 0;
        loop {
            let states = state.next_states_n(game, 1, false).collect::<Vec<_>>();
            if states.is_empty() {
                break;
            }

            moves.push(states.len() as f32);
            if !state.is_keeper() {
                turn += 1;
            }

            state = states.into_iter().choose(rng).unwrap();
        }

        times.push(start.elapsed().as_nanos() as f32 / 1e6);
        turns.push(turn as f32);

        println!("after {} plays:", play);
        println!("  avg. moves: {:.3}", avg(&moves));
        println!("  avg. times: {:.3}ms", avg(&times));
        println!("  avg. turns: {:.3}", avg(&turns));
    }
}

fn run_perf(game: &Game, depth: usize) {
    let state = State::initial(game);
    let start = Instant::now();
    let count = state.next_states_n(game, depth, true).count();
    println!(
        "runPerf(depth: {}): {:.3}ms",
        depth,
        start.elapsed().as_nanos() as f32 / 1e6
    );
    println!("runPerf(depth: {}) = {}", depth, count);
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let file = args.get(1).expect("Game IST file expected.");
    let game = Game::from_ist_file(&file);

    match args.get(2).expect("Operation expected.").as_str() {
        "perf" => {
            let depth = args
                .get(3)
                .map_or(10, |depth| depth.parse::<usize>().unwrap());

            for depth in 0..=depth {
                run_perf(&game, depth);
            }
        }
        "run" => {
            let mut rng = rand::thread_rng();
            let plays = args
                .get(3)
                .map_or(10, |plays| plays.parse::<usize>().unwrap());

            run(&game, &mut rng, plays);
        }
        _ => panic!("Unknown operation."),
    }
}
