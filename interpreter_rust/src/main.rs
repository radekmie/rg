use interpreter_rust::rg::ist::{Game, RuntimeId, LABEL_BEGIN, LABEL_KEEPER, LABEL_PLAYER};
use interpreter_rust::utils::interner::Interner;
use interpreter_rust::utils::map_id::MapId;
use rand::{rngs::ThreadRng, seq::IteratorRandom};
use std::fs::read_to_string;
use std::{collections::BTreeMap, env, time::Instant};

fn avg(counter: &BTreeMap<usize, usize>) -> f32 {
    let (x0, n0) = counter
        .iter()
        .fold((0, 0), |(x0, n0), (x, n)| (x0 + n * x, n0 + n));
    x0 as f32 / n0 as f32
}

fn increase(counter: &mut BTreeMap<usize, usize>, x: usize) {
    counter.entry(x).and_modify(|n| *n += 1).or_insert(1);
}

fn run(game: &Game<RuntimeId>, rng: &mut ThreadRng, plays: usize) {
    // Display stats every ~1% of plays.
    let step = 1f32.max(10f32.powf((plays as f32 / 100f32).log10().floor())) as usize;

    // Initialize counters.
    let mut moves: BTreeMap<usize, usize> = Default::default();
    let mut times: BTreeMap<usize, usize> = Default::default();
    let mut turns: BTreeMap<usize, usize> = Default::default();

    for play in 1..=plays {
        let start = Instant::now();
        let mut state = game.initial_state();
        let mut turn = 0;
        loop {
            let states = state.next_states_depth(game, 1, false).collect::<Vec<_>>();
            if states.is_empty() {
                break;
            }

            if !state.get_player().is_keeper() {
                increase(&mut moves, states.len());
                turn += 1;
            }

            state = states.into_iter().choose(rng).unwrap();
        }

        increase(&mut times, start.elapsed().as_nanos() as usize);
        increase(&mut turns, turn);

        if play % step == 0 {
            print!("{esc}c", esc = 27 as char);
            println!("after {} plays:", play);
            println!("  avg. moves: {:.3}", avg(&moves));
            println!("  avg. turns: {:.3}", avg(&turns));
            println!("  avg. times: {:.3}ms", avg(&times) / 1e6f32);
        }
    }
}

fn perf(game: &Game<RuntimeId>, depth: usize) {
    let start = Instant::now();
    let state = game.initial_state();
    let count = state.next_states_depth(game, depth, true).count();
    println!(
        "perf(depth: {}) = {} in {:.3}ms",
        depth,
        count,
        start.elapsed().as_nanos() as f32 / 1e6
    );
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let file = args.get(1).expect("Game IST file expected.");
    let source = read_to_string(file).expect("Couldn't open file.");

    let mut interner = Interner::default();
    interner.intern_as("begin", LABEL_BEGIN);
    interner.intern_as("keeper", LABEL_KEEPER);
    interner.intern_as("player", LABEL_PLAYER);

    let game = serde_json::from_str::<Game<&str>>(source.as_str())
        .expect("Incorrect IST file.")
        .map_id(&mut |id| interner.intern(id));

    match args.get(2).expect("Operation expected.").as_str() {
        "perf" => {
            let depth = args
                .get(3)
                .map_or(10, |depth| depth.parse::<usize>().unwrap());

            for depth in 0..=depth {
                perf(&game, depth);
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
