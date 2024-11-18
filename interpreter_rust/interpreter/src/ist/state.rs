use super::{EdgeLabel, Expression, Game, RuntimeId, Value, LABEL_END};
use std::collections::BTreeSet;
use std::rc::Rc;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct State {
    pub goals: Rc<Value<RuntimeId>>,
    pub player: Rc<Value<RuntimeId>>,
    pub position: RuntimeId,
    pub tags: Rc<Vec<RuntimeId>>,
    pub values: Rc<Vec<Rc<Value<RuntimeId>>>>,
    pub visible: Rc<Value<RuntimeId>>,
}

impl State {
    pub fn clone_at(&self, position: RuntimeId) -> Self {
        Self {
            goals: self.goals.clone(),
            player: self.goals.clone(),
            position,
            tags: self.tags.clone(),
            values: self.values.clone(),
            visible: self.visible.clone(),
        }
    }

    pub fn eval<'a>(
        &'a self,
        game: &'a Game<RuntimeId>,
        expression: &'a Expression<RuntimeId>,
    ) -> &'a Rc<Value<RuntimeId>> {
        match expression {
            Expression::Access { lhs, rhs } => {
                let Value::Element { value } = &**self.eval(game, rhs) else {
                    panic!("Only Element can be key.")
                };
                let Value::Map { default, values } = &**self.eval(game, lhs) else {
                    panic!("Only Map can be accessed.")
                };
                values.get(value).unwrap_or(default)
            }
            Expression::ConstantReference { index } => &game.constants[*index],
            Expression::GoalsReference => &self.goals,
            Expression::Literal { value } => value,
            Expression::PlayerReference => &self.player,
            Expression::VariableReference { index } => &self.values[*index],
            Expression::VisibleReference => &self.visible,
        }
    }

    pub fn eval_set(
        &mut self,
        game: &Game<RuntimeId>,
        expression: &Expression<RuntimeId>,
        set: Rc<Value<RuntimeId>>,
    ) {
        match expression {
            Expression::Access { lhs, rhs } => {
                let Value::Element { value } = &**self.eval(game, rhs) else {
                    panic!("Only Element can be key.")
                };

                let mut map = self.eval(game, lhs).clone();
                let Value::Map { default, values } = Rc::make_mut(&mut map) else {
                    panic!("Only Map can be accessed.");
                };

                if &set == default {
                    Rc::make_mut(values).remove(value);
                } else {
                    Rc::make_mut(values).insert(*value, set);
                }

                self.eval_set(game, lhs, map);
            }
            Expression::ConstantReference { .. } => panic!("ConstantReference is immutable."),
            Expression::GoalsReference => self.goals = set,
            Expression::Literal { .. } => panic!("Literal is immutable."),
            Expression::PlayerReference => self.player = set,
            Expression::VariableReference { index } => {
                Rc::make_mut(&mut self.values)[*index] = set;
            }
            Expression::VisibleReference => self.visible = set,
        }
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
            return_queue: Vec::default(),
            search_queue: vec![self.clone()],
            visited_states: VisitedStates::new(game),
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
    visited_states: VisitedStates<'a>,
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
                if visited_states.visited(&state) {
                    continue;
                }

                if let Some(edges) = game.edges.get(&state.position) {
                    let is_fully_disjoint = game.disjoints.contains(&state.position);
                    let mut reachables: Option<Vec<(RuntimeId, RuntimeId, bool)>> = None;
                    for edge in edges {
                        let mut state = state.clone_at(edge.next);
                        match &edge.label {
                            EdgeLabel::Assignment { lhs, rhs } => {
                                state.eval_set(game, lhs, state.eval(game, rhs).clone());
                                if *break_on_player && *lhs == Expression::PlayerReference {
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
                                } else {
                                    continue;
                                }
                            }
                            EdgeLabel::Reachability { lhs, rhs, negated } => {
                                let reachables = reachables.get_or_insert_with(Vec::new);
                                let is_reachable = reachables
                                    .iter()
                                    .find_map(|x| {
                                        if x.0 == *lhs && x.1 == *rhs {
                                            Some(x.2)
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or_else(|| {
                                        let is_reachable =
                                            state.clone_at(*lhs).is_reachable(game, *rhs);
                                        reachables.push((*lhs, *rhs, is_reachable));
                                        is_reachable
                                    });

                                if is_reachable != *negated {
                                    search_queue.push(state);
                                } else {
                                    continue;
                                }
                            }
                            EdgeLabel::Skip => {
                                search_queue.push(state);
                            }
                            EdgeLabel::Tag { symbol } => {
                                state.tags = Rc::new([&state.tags[..], &[*symbol]].concat());
                                search_queue.push(state);
                            }
                        }

                        if is_fully_disjoint {
                            break;
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

            let prev = &state.player;
            let skip = *ignore_keeper && prev.is_keeper();
            let mut tags_set = BTreeSet::default();
            for mut state in state.next_states(game, true) {
                if !tags_set.insert(state.tags.clone()) {
                    continue;
                }

                let next = &state.player;
                let is_finish = next.is_keeper() && state.is_final() && !*ignore_keeper;
                let is_switch = next != prev && !skip;
                state.tags = Rc::default();
                queue.push((state, depth - usize::from(is_finish || is_switch)));
            }
        }

        None
    }
}

struct VisitedStates<'a> {
    game: &'a Game<RuntimeId>,
    #[expect(clippy::type_complexity)]
    states: Option<BTreeSet<(RuntimeId, Rc<Vec<RuntimeId>>, Rc<Vec<Rc<Value<RuntimeId>>>>)>>,
}

impl<'a> VisitedStates<'a> {
    fn new(game: &'a Game<RuntimeId>) -> Self {
        Self { game, states: None }
    }

    fn visited(&mut self, state: &State) -> bool {
        if self.game.uniques.contains(&state.position) {
            return false;
        }

        let values = self.game.repeats.get(&state.position).map_or_else(
            || state.values.clone(),
            |indexes| {
                Rc::new(
                    indexes
                        .iter()
                        .map(|&index| state.values[index].clone())
                        .collect(),
                )
            },
        );

        !self.states.get_or_insert_with(BTreeSet::new).insert((
            state.position,
            state.tags.clone(),
            values,
        ))
    }
}
