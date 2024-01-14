use super::state::State;
use crate::ist::{
    Game, RuntimeId, Value, LABEL_BEGIN, LABEL_END, LABEL_GOALS, LABEL_KEEPER, LABEL_PLAYER,
};
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::BTreeMap;
use std::rc::Rc;
use utils::Interner;

pub type ISTInterner = Interner<Rc<str>, RuntimeId>;

pub fn new_ist_interner() -> ISTInterner {
    let mut interner = Interner::default();
    interner.intern_as(&Rc::from("begin"), LABEL_BEGIN);
    interner.intern_as(&Rc::from("end"), LABEL_END);
    interner.intern_as(&Rc::from("goals"), LABEL_GOALS);
    interner.intern_as(&Rc::from("keeper"), LABEL_KEEPER);
    interner.intern_as(&Rc::from("player"), LABEL_PLAYER);
    interner
}

impl Game<RuntimeId> {
    pub fn initial_state(&self) -> State {
        State {
            position: LABEL_BEGIN,
            tags: Rc::default(),
            values: Rc::new(
                self.variables
                    .iter()
                    .map(|(name, variable)| (*name, variable.default.clone()))
                    .collect(),
            ),
        }
    }

    pub fn perf(&self, depth: usize, callback: &impl Fn(usize)) {
        callback(
            self.initial_state()
                .next_states_depth(self, depth, true)
                .count(),
        );
    }

    pub fn run<R: Rng>(
        &self,
        rng: &mut R,
        plays: usize,
        callback: &impl Fn((usize, f32, f32, &BTreeMap<Value<RuntimeId>, usize>)),
    ) {
        fn avg(counter: &BTreeMap<usize, usize>) -> f32 {
            let (x0, n0) = counter
                .iter()
                .fold((0, 0), |(x0, n0), (x, n)| (x0 + n * x, n0 + n));
            x0 as f32 / n0 as f32
        }

        fn increase<Key: Ord>(counter: &mut BTreeMap<Key, usize>, x: Key) {
            counter.entry(x).and_modify(|n| *n += 1).or_insert(1);
        }

        // Display stats every ~1% of plays.
        let step = 1f32.max(10f32.powf((plays as f32 / 100f32).log10().floor())) as usize;

        // Initialize counters.
        let mut goals: BTreeMap<Value<RuntimeId>, usize> = BTreeMap::default();
        let mut moves: BTreeMap<usize, usize> = BTreeMap::default();
        let mut turns: BTreeMap<usize, usize> = BTreeMap::default();

        for play in 1..=plays {
            let mut state = self.initial_state();
            let mut turn = 0;
            loop {
                let states = state.next_states_depth(self, 1, false).collect::<Vec<_>>();
                if states.is_empty() {
                    increase(&mut goals, state.get_goals().clone());
                    break;
                }

                if !state.get_player().is_keeper() {
                    increase(&mut moves, states.len());
                    turn += 1;
                }

                state = states.into_iter().choose(rng).unwrap();
            }

            increase(&mut turns, turn);

            if play % step == 0 {
                callback((play, avg(&moves), avg(&turns), &goals));
            }
        }
    }
}
