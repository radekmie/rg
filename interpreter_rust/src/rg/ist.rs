use crate::utils::map_id::MapId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::rc::Rc;

// Interned strings that the interpreter relies on.
pub type RuntimeId = u16;
pub const LABEL_BEGIN: RuntimeId = 0;
pub const LABEL_KEEPER: RuntimeId = 1;
pub const LABEL_PLAYER: RuntimeId = 2;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub struct Edge<Id: Ord> {
    pub label: EdgeLabel<Id>,
    pub next: Id,
}

impl<OldId: Ord, NewId: Ord> MapId<Edge<NewId>, OldId, NewId> for Edge<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Edge<NewId> {
        Edge {
            label: self.label.map_id(map),
            next: map(&self.next),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum EdgeLabel<Id: Ord> {
    Assignment {
        lhs: Expression<Id>,
        rhs: Expression<Id>,
    },
    Comparison {
        lhs: Expression<Id>,
        rhs: Expression<Id>,
        negated: bool,
    },
    Reachability {
        lhs: Id,
        rhs: Id,
        negated: bool,
    },
    Skip,
}

impl<OldId: Ord, NewId: Ord> MapId<EdgeLabel<NewId>, OldId, NewId> for EdgeLabel<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> EdgeLabel<NewId> {
        match self {
            EdgeLabel::Assignment { lhs, rhs } => EdgeLabel::Assignment {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            EdgeLabel::Comparison { lhs, rhs, negated } => EdgeLabel::Comparison {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
                negated: *negated,
            },
            EdgeLabel::Reachability { lhs, rhs, negated } => EdgeLabel::Reachability {
                lhs: map(lhs),
                rhs: map(rhs),
                negated: *negated,
            },
            EdgeLabel::Skip => EdgeLabel::Skip,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum Expression<Id: Ord> {
    Access {
        lhs: Rc<Expression<Id>>,
        rhs: Rc<Expression<Id>>,
    },
    ConstantReference {
        identifier: Id,
    },
    Literal {
        value: Rc<Value<Id>>,
    },
    VariableReference {
        identifier: Id,
    },
}

impl Expression<RuntimeId> {
    pub fn is_player_reference(&self) -> bool {
        matches!(self, Self::VariableReference { identifier } if *identifier == LABEL_PLAYER)
    }
}

impl<OldId: Ord, NewId: Ord> MapId<Expression<NewId>, OldId, NewId> for Expression<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Expression<NewId> {
        match self {
            Expression::Access { lhs, rhs } => Expression::Access {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            Expression::ConstantReference { identifier } => Expression::ConstantReference {
                identifier: map(identifier),
            },
            Expression::Literal { value } => Expression::Literal {
                value: value.map_id(map),
            },
            Expression::VariableReference { identifier } => Expression::VariableReference {
                identifier: map(identifier),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub struct Game<Id: Ord> {
    pub constants: BTreeMap<Id, Rc<Value<Id>>>,
    pub edges: BTreeMap<Id, Vec<Edge<Id>>>,
    pub types: BTreeMap<Id, Type<Id>>,
    pub variables: BTreeMap<Id, Variable<Id>>,
}

impl Game<RuntimeId> {
    pub fn initial_state(&self) -> State {
        State {
            position: LABEL_BEGIN,
            values: Rc::new(
                self.variables
                    .iter()
                    .map(|(name, variable)| (*name, variable.default.clone()))
                    .collect(),
            ),
        }
    }
}

impl<OldId: Ord, NewId: Ord> MapId<Game<NewId>, OldId, NewId> for Game<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Game<NewId> {
        Game {
            constants: self.constants.map_id(map),
            edges: self.edges.map_id(map),
            types: self.types.map_id(map),
            variables: self.variables.map_id(map),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct State {
    position: RuntimeId,
    values: Rc<BTreeMap<RuntimeId, Rc<Value<RuntimeId>>>>,
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

    pub fn get_player(&self) -> &Rc<Value<RuntimeId>> {
        self.values.get(&LABEL_PLAYER).unwrap()
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
                let step = if skip || prev == next { 0 } else { 1 };

                queue.push((state, depth - step));
            }
        }

        None
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum Type<Id: Ord> {
    Arrow {
        lhs: Rc<Type<Id>>,
        rhs: Rc<Type<Id>>,
    },
    Set {
        values: Vec<Value<Id>>,
    },
}

impl<OldId: Ord, NewId: Ord> MapId<Type<NewId>, OldId, NewId> for Type<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Type<NewId> {
        match self {
            Type::Arrow { lhs, rhs } => Type::Arrow {
                lhs: lhs.map_id(map),
                rhs: rhs.map_id(map),
            },
            Type::Set { values } => Type::Set {
                values: values.map_id(map),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub enum Value<Id: Ord> {
    Element {
        value: Id,
    },
    Map {
        #[serde(rename = "defaultValue")]
        default: Rc<Value<Id>>,
        values: Rc<BTreeMap<Id, Rc<Value<Id>>>>,
    },
}

impl Value<RuntimeId> {
    pub fn is_keeper(&self) -> bool {
        matches!(self, Self::Element { value } if *value == LABEL_KEEPER)
    }
}

impl<OldId: Ord, NewId: Ord> MapId<Value<NewId>, OldId, NewId> for Value<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Value<NewId> {
        match self {
            Value::Element { value } => Value::Element { value: map(value) },
            Value::Map { default, values } => Value::Map {
                default: default.map_id(map),
                values: values.map_id(map),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind")]
pub struct Variable<Id: Ord> {
    #[serde(rename = "defaultValue")]
    pub default: Rc<Value<Id>>,
    #[serde(rename = "type")]
    pub type_: Rc<Type<Id>>,
}

impl<OldId: Ord, NewId: Ord> MapId<Variable<NewId>, OldId, NewId> for Variable<OldId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Variable<NewId> {
        Variable {
            default: self.default.map_id(map),
            type_: self.type_.map_id(map),
        }
    }
}
