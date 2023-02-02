use crate::rg::ist::{Game, RuntimeId, LABEL_BEGIN, LABEL_KEEPER, LABEL_PLAYER};
use crate::rg::ist_state::State;
use rand::seq::IteratorRandom;
use rand::Rng;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::rc::Rc;

pub struct Interner<Id: Ord> {
    // TODO: Maybe `Rc<String>` would be better?
    id_to_string: BTreeMap<Id, String>,
    string_to_id: BTreeMap<String, Id>,
}

impl<Id: Copy + Ord + TryFrom<usize>> Interner<Id> {
    pub fn intern(&mut self, string: &str) -> Id
    where
        <Id as TryFrom<usize>>::Error: Debug,
    {
        if let Some(id) = self.string_to_id.get(string) {
            return *id;
        }

        const ERROR: &str = "Maximum number of interned strings reached! Increase Id size.";
        let id = self
            .string_to_id
            .len()
            .checked_add(1)
            .expect(ERROR)
            .try_into()
            .expect(ERROR);
        self.intern_as(string, id)
    }

    pub fn intern_as(&mut self, string: &str, id: Id) -> Id {
        assert!(!self.id_to_string.contains_key(&id));
        assert!(!self.string_to_id.contains_key(string));
        self.id_to_string.insert(id, string.into());
        self.string_to_id.insert(string.into(), id);
        id
    }

    pub fn recall(&self, id: &Id) -> Option<&String> {
        self.id_to_string.get(id)
    }
}

impl Default for Interner<RuntimeId> {
    fn default() -> Self {
        let mut interner = Self {
            id_to_string: Default::default(),
            string_to_id: Default::default(),
        };
        interner.intern_as("begin", LABEL_BEGIN);
        interner.intern_as("keeper", LABEL_KEEPER);
        interner.intern_as("player", LABEL_PLAYER);
        interner
    }
}

pub fn initial_state(game: &Game<RuntimeId>) -> State {
    State {
        position: LABEL_BEGIN,
        values: Rc::new(
            game.variables
                .iter()
                .map(|(name, variable)| (*name, variable.default.clone()))
                .collect(),
        ),
    }
}

pub fn perf(game: &Game<RuntimeId>, depth: usize, callback: &impl Fn(usize)) {
    callback(
        initial_state(game)
            .next_states_depth(game, depth, true)
            .count(),
    );
}

pub fn run<R: Rng>(
    game: &Game<RuntimeId>,
    rng: &mut R,
    plays: usize,
    callback: &impl Fn((usize, f32, f32)),
) {
    fn avg(counter: &BTreeMap<usize, usize>) -> f32 {
        let (x0, n0) = counter
            .iter()
            .fold((0, 0), |(x0, n0), (x, n)| (x0 + n * x, n0 + n));
        x0 as f32 / n0 as f32
    }

    fn increase(counter: &mut BTreeMap<usize, usize>, x: usize) {
        counter.entry(x).and_modify(|n| *n += 1).or_insert(1);
    }

    // Display stats every ~1% of plays.
    let step = 1f32.max(10f32.powf((plays as f32 / 100f32).log10().floor())) as usize;

    // Initialize counters.
    let mut moves: BTreeMap<usize, usize> = Default::default();
    let mut turns: BTreeMap<usize, usize> = Default::default();

    for play in 1..=plays {
        let mut state = initial_state(game);
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

        increase(&mut turns, turn);

        if play % step == 0 {
            callback((play, avg(&moves), avg(&turns)));
        }
    }
}
