use super::state::State;
use crate::ist::{Game, RuntimeId, Value, LABEL_BEGIN, LABEL_END, LABEL_KEEPER};
use map_id::MapId;
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;
use std::sync::Arc;
use utils::interner::Interner;
use web_time::Instant;

pub type ISTInterner = Interner<Arc<str>, RuntimeId>;

pub fn new_ist_interner() -> ISTInterner {
    let mut interner = Interner::default();
    interner.intern_as(&Arc::from("begin"), LABEL_BEGIN);
    interner.intern_as(&Arc::from("end"), LABEL_END);
    interner.intern_as(&Arc::from("keeper"), LABEL_KEEPER);
    interner
}

impl Game<RuntimeId> {
    pub fn initial_state(&self) -> State {
        State {
            goals: self.initial_goals.clone(),
            player: self.initial_player.clone(),
            position: LABEL_BEGIN,
            tags: Rc::default(),
            values: self.initial_values.clone(),
            visible: self.initial_visible.clone(),
        }
    }

    pub fn perf(&self, depth: usize, callback: &impl Fn(usize)) {
        callback(
            self.initial_state()
                .next_states_depth(self, depth, true)
                .count(),
        );
    }

    pub fn run<Id: Clone + Display + Ord, R: Rng>(
        &self,
        rng: &mut R,
        interner: &Interner<Id, RuntimeId>,
        plays: usize,
        callback: &impl Fn(Vec<String>),
    ) {
        fn stats(counter: &BTreeMap<usize, usize>) -> (usize, usize, f32, f32) {
            let max = *counter.keys().max().unwrap();
            let min = *counter.keys().min().unwrap();
            let (x0, n0) = counter
                .iter()
                .fold((0, 0), |(x0, n0), (x, n)| (x0 + n * x, n0 + n));
            let avg = x0 as f32 / n0 as f32;
            let var = counter
                .iter()
                .map(|(x, n)| *n as f32 * (avg - *x as f32).powf(2.0))
                .sum::<f32>()
                / n0 as f32;
            (min, max, avg, var.sqrt())
        }

        fn increase<Key: Ord>(counter: &mut BTreeMap<Key, usize>, x: Key) {
            counter.entry(x).and_modify(|n| *n += 1).or_insert(1);
        }

        // Display stats every ~1% of plays.
        let step = 1f32.max(10f32.powf((plays as f32 / 100f32).log10().floor())) as usize;

        // Initialize counters.
        let mut goals: BTreeMap<Rc<Value<RuntimeId>>, usize> = BTreeMap::default();
        let mut moves: BTreeMap<usize, usize> = BTreeMap::default();
        let mut times: BTreeMap<usize, usize> = BTreeMap::default();
        let mut turns: BTreeMap<usize, usize> = BTreeMap::default();

        for play in 1..=plays {
            let mut state = self.initial_state();
            let mut turn = 0;
            let now = Instant::now();
            loop {
                let states = state.next_states_depth(self, 1, false).collect::<Vec<_>>();
                if states.is_empty() {
                    increase(&mut goals, state.goals.clone());
                    break;
                }

                if !state.player.is_keeper() {
                    increase(&mut moves, states.len());
                    turn += 1;
                }

                state = states.into_iter().choose(rng).unwrap();
            }

            increase(&mut times, now.elapsed().as_micros() as usize);
            increase(&mut turns, turn);

            if play % step == 0 {
                // Statistics.
                let moves = stats(&moves);
                let times = stats(&times);
                let turns = stats(&turns);

                // Formatting.
                macro_rules! width {
                    ($index:tt) => {
                        // TODO: `log10` would be much faster, but was incorrect
                        // for some numbers due to precision.
                        3.max(format!("{:.3}", moves.$index).len())
                            .max(format!("{:.3}", times.$index).len())
                            .max(format!("{:.3}", turns.$index).len())
                    };
                }

                let w0 = width!(0);
                let w1 = width!(1);
                let w2 = width!(2);
                let w3 = width!(3);

                macro_rules! line {
                    ($ident:ident, $extra:expr) => {
                        format!(
                            "  {}  {:w0$}  {:w2$.3}  {:w3$.3}  {:w1$} {}",
                            stringify!($ident),
                            $ident.0,
                            $ident.2,
                            $ident.3,
                            $ident.1,
                            $extra,
                            w0 = w0,
                            w1 = w1,
                            w2 = w2,
                            w3 = w3
                        )
                    };
                }

                let mut lines = vec![
                    format!("after {play} plays"),
                    format!(
                        "         {:>w0$}  {:>w2$}  {:>w3$}  {:>w1$}",
                        "min",
                        "avg",
                        "std.dev",
                        "max",
                        w0 = w0,
                        w1 = w1,
                        w2 = w2,
                        w3 = w3
                    ),
                    line!(moves, ""),
                    line!(turns, ""),
                    line!(times, "[µs]"),
                    format!("  scores"),
                ];

                // Scores.
                let mut sorted_goals = goals
                    .iter()
                    .map(|(score, count)| {
                        (score.map_id(&mut |id| interner.recall(id).unwrap()), count)
                    })
                    .collect::<Vec<_>>();
                sorted_goals.sort_unstable();
                for (score, count) in sorted_goals {
                    lines.push(format!(
                        "    {:5.2}% of {score}",
                        *count as f32 * 1e2 / play as f32
                    ));
                }

                callback(lines);
            }
        }
    }
}
