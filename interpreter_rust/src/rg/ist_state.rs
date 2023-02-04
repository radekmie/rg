use crate::rg::ist::{
    EdgeLabel, Expression, Game, RuntimeId, Value, LABEL_END, LABEL_GOALS, LABEL_PLAYER,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct State {
    pub position: RuntimeId,
    pub values: Rc<BTreeMap<RuntimeId, Rc<Value<RuntimeId>>>>,
}

impl State {
    pub fn clone_at(&self, position: RuntimeId) -> Self {
        Self {
            position,
            values: self.values.clone(),
        }
    }

    pub fn eval<'a>(
        &'a self,
        game: &'a Game<RuntimeId>,
        expression: &'a Expression<RuntimeId>,
    ) -> &'a Rc<Value<RuntimeId>> {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => match self.eval(game, lhs).deref() {
                    Value::Map { default, values } => values.get(value).unwrap_or(default),
                    _ => panic!("Only Map can be accessed."),
                },
                _ => panic!("Only Element can be key."),
            },
            Expression::ConstantReference { identifier } => game.constants.get(identifier).unwrap(),
            Expression::Literal { value } => value,
            Expression::VariableReference { identifier } => self.values.get(identifier).unwrap(),
        }
    }

    pub fn eval_set(
        &mut self,
        game: &Game<RuntimeId>,
        expression: &Expression<RuntimeId>,
        set: Rc<Value<RuntimeId>>,
    ) {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => {
                    let mut map = self.eval(game, lhs).clone();
                    if let Value::Map { default, values } = Rc::make_mut(&mut map) {
                        if &set == default {
                            Rc::make_mut(values).remove(value);
                        } else {
                            Rc::make_mut(values).insert(*value, set);
                        }
                    } else {
                        panic!("Only Map can be accessed.");
                    }

                    self.eval_set(game, lhs, map);
                }
                _ => panic!("Only Element can be key."),
            },
            Expression::ConstantReference { .. } => panic!("ConstantReference is immutable."),
            Expression::Literal { .. } => panic!("Literal is immutable."),
            Expression::VariableReference { identifier } => {
                Rc::make_mut(&mut self.values).insert(*identifier, set);
            }
        }
    }

    pub fn get_goals(&self) -> &Rc<Value<RuntimeId>> {
        self.values.get(&LABEL_GOALS).unwrap()
    }

    pub fn get_player(&self) -> &Rc<Value<RuntimeId>> {
        self.values.get(&LABEL_PLAYER).unwrap()
    }

    pub fn is_final(&self) -> bool {
        self.position == LABEL_END
    }

    pub fn is_reachable(&self, game: &Game<RuntimeId>, position: RuntimeId) -> bool {
        self.next_states(game, false)
            .any(|state| state.position == position)
    }

    pub fn next_states<'a>(
        &'a self,
        game: &'a Game<RuntimeId>,
        break_on_player: bool,
    ) -> StateNext<'a> {
        StateNext {
            break_on_player,
            game,
            return_queue: Default::default(),
            search_queue: vec![self.clone()],
            visited_states: Default::default(),
        }
    }

    pub fn next_states_depth<'a>(
        &'a self,
        game: &'a Game<RuntimeId>,
        depth: usize,
        ignore_keeper: bool,
    ) -> StateNextDepth<'a> {
        StateNextDepth {
            game,
            ignore_keeper,
            queue: vec![(self.clone(), depth)],
        }
    }
}

pub struct StateNext<'a> {
    break_on_player: bool,
    game: &'a Game<RuntimeId>,
    return_queue: Vec<State>,
    search_queue: Vec<State>,
    visited_states: BTreeSet<State>,
}

impl Iterator for StateNext<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            break_on_player,
            game,
            return_queue,
            search_queue,
            visited_states,
        } = self;

        while !return_queue.is_empty() || !search_queue.is_empty() {
            if let Some(state) = return_queue.pop() {
                return Some(state);
            }

            if let Some(state) = search_queue.pop() {
                // Check whether this state was already visited and if so, skip
                // it. It could happen conditionally, only when a game requires
                // that, but that'd require an additional analysis.
                if visited_states.contains(&state) {
                    continue;
                }

                visited_states.insert(state.clone());

                if let Some(edges) = game.edges.get(&state.position) {
                    let mut reachables: Option<BTreeMap<(RuntimeId, RuntimeId), bool>> = None;
                    for edge in edges {
                        let mut state = state.clone_at(edge.next);
                        match &edge.label {
                            EdgeLabel::Assignment { lhs, rhs } => {
                                state.eval_set(game, lhs, state.eval(game, rhs).clone());
                                if *break_on_player && lhs.is_player_reference() {
                                    return_queue.push(state);
                                } else {
                                    search_queue.push(state);
                                }
                            }
                            EdgeLabel::Comparison { lhs, rhs, negated } => {
                                let lhs_value = state.eval(game, lhs);
                                let rhs_value = state.eval(game, rhs);
                                let is_equal = lhs_value == rhs_value;
                                if is_equal != *negated {
                                    search_queue.push(state);
                                }
                            }
                            EdgeLabel::Reachability { lhs, rhs, negated } => {
                                let is_reachable = *reachables
                                    .get_or_insert_with(BTreeMap::new)
                                    .entry((*lhs, *rhs))
                                    .or_insert_with(|| {
                                        state.clone_at(*lhs).is_reachable(game, *rhs)
                                    });

                                if is_reachable != *negated {
                                    search_queue.push(state);
                                }
                            }
                            EdgeLabel::Skip => {
                                search_queue.push(state);
                            }
                        }
                    }
                }

                if !*break_on_player {
                    return Some(state);
                }
            }
        }

        None
    }
}

pub struct StateNextDepth<'a> {
    game: &'a Game<RuntimeId>,
    ignore_keeper: bool,
    queue: Vec<(State, usize)>,
}

impl Iterator for StateNextDepth<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            game,
            ignore_keeper,
            queue,
        } = self;

        while let Some((state, depth)) = queue.pop() {
            if depth == 0 {
                return Some(state);
            }

            let prev = state.get_player();
            let skip = *ignore_keeper && prev.is_keeper();
            for state in state.next_states(game, true) {
                let next = state.get_player();
                let is_finish = next.is_keeper() && state.is_final() && !*ignore_keeper;
                let is_switch = next != prev && !skip;
                queue.push((state, depth - usize::from(is_finish || is_switch)));
            }
        }

        None
    }
}
