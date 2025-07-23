use super::state::State;
use crate::ist::{Game, RuntimeId, Value, LABEL_BEGIN, LABEL_END, LABEL_KEEPER, LABEL_RANDOM};
use map_id::MapId;
use rand::seq::IteratorRandom;
use rand::Rng;
use rg::ast::{Error, Game as GameAst};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;
use std::sync::Arc;
use utils::interner::Interner;
use web_time::Instant;

type Id = Arc<str>;

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

    pub fn initial_state_after(
        &self,
        interner: &Interner<Id, RuntimeId>,
        path: &str,
    ) -> Result<State, String> {
        if !path.starts_with('/') || !path.ends_with('/') {
            return Err("Incorrect path format.".to_string());
        }

        let mut state = self.initial_state();
        if path.len() != 1 {
            'next: for move_ in path[1..path.len() - 1].split('/') {
                let tags = if move_.is_empty() {
                    vec![]
                } else {
                    move_
                        .split(' ')
                        .map(|tag| {
                            interner
                                .interned(&Id::from(tag))
                                .copied()
                                .ok_or_else(|| format!("Unknown tag '{tag}'."))
                        })
                        .collect::<Result<Vec<_>, _>>()?
                };

                for next_state in state.next_states(self, true) {
                    if tags == *next_state.tags.as_ref() {
                        state = next_state;
                        state.tags = Rc::default();
                        continue 'next;
                    }
                }

                return Err(format!("Path '{path}' failed at '{move_}'."));
            }
        }

        Ok(state)
    }

    pub fn perf(&self, initial_state: &State, depth: usize) -> (usize, f32) {
        let now = Instant::now();
        let count = initial_state.next_states_depth(self, depth).count();
        (count, now.elapsed().as_micros() as f32 / 1e3)
    }

    pub fn run<Id: Clone + Display + Ord, R: Rng>(
        &self,
        rng: &mut R,
        interner: &Interner<Id, RuntimeId>,
        initial_state: &State,
        plays: usize,
        callback: &Option<impl Fn(Vec<String>)>,
    ) -> Result<(), String> {
        fn stats(counter: &BTreeMap<usize, usize>) -> (usize, usize, f32, f32) {
            if counter.is_empty() {
                return (0, 0, 0.0, 0.0);
            }

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
            let mut state = initial_state.clone();
            let mut turn = 0;
            let now = Instant::now();
            loop {
                let mut states = state.next_states(self, true).collect::<Vec<_>>();
                if states.is_empty() {
                    if !state.is_final() {
                        return Err(format!(
                            "Game unexpectedly ended in {}.",
                            interner.recall(&state.position).unwrap()
                        ));
                    }

                    increase(&mut goals, state.goals.clone());
                    break;
                }

                if state.player.is_keeper() {
                    if states.len() != 1 {
                        return Err(format!(
                            "keeper had {} moves in {}.",
                            states.len(),
                            interner.recall(&state.position).unwrap()
                        ));
                    }
                } else if !state.player.is_random() {
                    increase(&mut moves, states.len());
                    turn += 1;
                }

                // Check if `(position, tags)` implies `values`, i.e., if all
                // states are derivable from their position and tags.
                states.sort_unstable_by(|x, y| {
                    x.position
                        .cmp(&y.position)
                        .then_with(|| x.tags.cmp(&y.tags))
                });

                for states in states.windows(2) {
                    if let [x, y] = states {
                        if x.position == y.position && x.tags == y.tags && x.values != y.values {
                            return Err(format!(
                                "Encountered two moves with different variables from {} to {}.",
                                interner.recall(&state.position).unwrap(),
                                interner.recall(&x.position).unwrap()
                            ));
                        }
                    }
                }

                state = states.into_iter().choose(rng).unwrap();
                state.tags = Rc::default();
            }

            increase(&mut times, now.elapsed().as_micros() as usize);
            increase(&mut turns, turn);

            if let Some(callback) = callback {
                if play != plays {
                    continue;
                }

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
                            .max(format!("{:.3}", times.$index as f32 / 1e3).len())
                            .max(format!("{:.3}", turns.$index).len())
                    };
                }

                let w0 = width!(0);
                let w1 = width!(1);
                let w2 = width!(2);
                let w3 = width!(3).max(7);

                macro_rules! line {
                    ($ident:ident, $scale:expr, $suffix:expr) => {
                        format!(
                            "  {}  {:w0$}  {:w2$.3}  {:w3$.3}  {:w1$} {}",
                            stringify!($ident),
                            $ident.0 as f32 / $scale as f32,
                            $ident.2 as f32 / $scale as f32,
                            $ident.3 as f32 / $scale as f32,
                            $ident.1 as f32 / $scale as f32,
                            $suffix,
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
                    line!(moves, 1, ""),
                    line!(turns, 1, ""),
                    line!(times, 1e3, "[ms]"),
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

        Ok(())
    }

    pub fn plays<Id: Clone + Display + Ord, R: Rng>(
        &self,
        rng: &mut R,
        interner: &Interner<Id, RuntimeId>,
        initial_state: &State,
        time_in_seconds: usize,
    ) -> Result<(), String> {
        let mut plays: usize = 0;
        let start = Instant::now();
        let time_in_micros = time_in_seconds as u128 * 1_000_000;

        while (start.elapsed().as_micros() as u128) < time_in_micros {
            let mut state = initial_state.clone();
            plays += 1;
            loop {
                let mut states = state.next_states(self, true).collect::<Vec<_>>();
                if states.is_empty() {
                    if !state.is_final() {
                        return Err(format!(
                            "Game unexpectedly ended in {}.",
                            interner.recall(&state.position).unwrap()
                        ));
                    }

                    break;
                }

                if state.player.is_keeper() {
                    if states.len() != 1 {
                        return Err(format!(
                            "keeper had {} moves in {}.",
                            states.len(),
                            interner.recall(&state.position).unwrap()
                        ));
                    }
                }

                // Check if `(position, tags)` implies `values`, i.e., if all
                // states are derivable from their position and tags.
                states.sort_unstable_by(|x, y| {
                    x.position
                        .cmp(&y.position)
                        .then_with(|| x.tags.cmp(&y.tags))
                });

                for states in states.windows(2) {
                    if let [x, y] = states {
                        if x.position == y.position && x.tags == y.tags && x.values != y.values {
                            return Err(format!(
                                "Encountered two moves with different variables from {} to {}.",
                                interner.recall(&state.position).unwrap(),
                                interner.recall(&x.position).unwrap()
                            ));
                        }
                    }
                }

                state = states.into_iter().choose(rng).unwrap();
                state.tags = Rc::default();
            }
        }

        println!("Played {plays}");

        Ok(())
    }

    /// This should be provided via the `TryFrom` trait, but it's impossible due
    /// to orphan rules.
    #[expect(clippy::type_complexity)]
    pub fn try_from(
        game: GameAst<Id>,
    ) -> Result<(Self, Interner<Id, RuntimeId>, BTreeMap<Id, usize>), Error<Id>> {
        let mut interner = Interner::default();
        interner.intern_as(&Arc::from("begin"), LABEL_BEGIN);
        interner.intern_as(&Arc::from("end"), LABEL_END);
        interner.intern_as(&Arc::from("keeper"), LABEL_KEEPER);
        interner.intern_as(&Arc::from("random"), LABEL_RANDOM);

        let context = Game::from_game(game);
        let game = context.game.map_id(&mut |id| interner.intern(id));
        Ok((game, interner, context.variables_indexes))
    }
}
