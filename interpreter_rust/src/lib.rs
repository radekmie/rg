pub mod deserializer;

use std::{collections::BTreeMap, ops::Deref, rc::Rc};

// We assume that there is not _a lot_ of unique symbols.
type Id = u16;

// Interned strings that the interpreter relies on.
const LABEL_BEGIN: Id = 0;
const LABEL_END: Id = 1;
const LABEL_KEEPER: Id = 2;
const LABEL_PLAYER: Id = 3;

type ValueMap = BTreeMap<Id, Rc<Value>>;

pub struct Edge {
    lhs: EdgeName,
    rhs: EdgeName,
    label: EdgeLabel,
}

impl Edge {
    pub fn generate(&self, lhs: &EdgeName) -> Vec<Rc<ValueMap>> {
        lhs.types
            .iter()
            .chain(
                self.rhs
                    .types
                    .iter()
                    .filter(|(bind, _)| !lhs.types.contains_key(bind)),
            )
            .fold(vec![self.rhs.values.clone()], |sources, (bind, type_)| {
                if let Some(value) = lhs.values.get(bind) {
                    sources
                        .iter()
                        .map(move |source| {
                            let mut source = source.clone();
                            Rc::make_mut(&mut source).insert(*bind, value.clone());
                            source
                        })
                        .collect()
                } else {
                    match type_.deref() {
                        Type::Arrow { .. } => panic!("Arrow iteration is disallowed."),
                        Type::Set { values } => values
                            .iter()
                            .flat_map(|value| {
                                sources.iter().map(move |source| {
                                    let mut source = source.clone();
                                    Rc::make_mut(&mut source).insert(*bind, value.clone());
                                    source
                                })
                            })
                            .collect(),
                    }
                }
            })
    }
}

pub enum EdgeLabel {
    Assignment {
        lhs: Expression,
        rhs: Expression,
    },
    Comparison {
        lhs: Expression,
        rhs: Expression,
        negated: bool,
    },
    Reachability {
        lhs: EdgeName,
        rhs: EdgeName,
        negated: bool,
    },
    Skip,
}

#[derive(Clone)]
pub struct EdgeName {
    label: Id,
    types: Rc<BTreeMap<Id, Type>>,
    values: Rc<ValueMap>,
}

impl EdgeName {
    fn is(&self, label: Id) -> bool {
        self.label == label
    }
}

impl PartialEq for EdgeName {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

pub enum Expression {
    Access {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    BindReference {
        identifier: Id,
    },
    ConstantReference {
        identifier: Id,
    },
    Literal {
        value: Rc<Value>,
    },
    VariableReference {
        identifier: Id,
    },
}

pub struct Game {
    constants: ValueMap,
    edges: Vec<Edge>,
    #[allow(dead_code)]
    types: BTreeMap<Id, Type>,
    variables: BTreeMap<Id, Variable>,
}

#[derive(Clone)]
pub struct State {
    position: EdgeName,
    values: Rc<ValueMap>,
}

impl State {
    pub fn eval<'a>(&'a self, game: &'a Game, expression: &'a Expression) -> &'a Rc<Value> {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => match self.eval(game, lhs).deref() {
                    Value::Map { default, values } => values.get(value).unwrap_or(default),
                    _ => panic!("Only Map can be accessed."),
                },
                _ => panic!("Only Element can be key."),
            },
            Expression::BindReference { identifier } => {
                // TODO: Check for unknown bound variables.
                self.position.values.get(identifier).unwrap()
            }
            Expression::ConstantReference { identifier } => {
                // TODO: Check for unknown constants.
                game.constants.get(identifier).unwrap()
            }
            Expression::Literal { value } => value,
            Expression::VariableReference { identifier } => {
                // TODO: Check for unknown variables.
                self.values.get(identifier).unwrap()
            }
        }
    }

    pub fn eval_set(&mut self, game: &Game, expression: &Expression, set: Rc<Value>) {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => {
                    let mut map = self.eval(game, lhs).clone();
                    if let Value::Map { default, values } = Rc::make_mut(&mut map) {
                        if &set == default {
                            values.remove(value);
                        } else {
                            values.insert(*value, set);
                        }
                    } else {
                        panic!("Only Map can be accessed.");
                    }

                    self.eval_set(game, lhs, map);
                }
                _ => panic!("Only Element can be key."),
            },
            Expression::BindReference { .. } => panic!("BindReference is immutable."),
            Expression::ConstantReference { .. } => panic!("ConstantReference is immutable."),
            Expression::Literal { .. } => panic!("Literal is immutable."),
            Expression::VariableReference { identifier } => {
                // TODO: Check for unknown variables.
                *Rc::make_mut(&mut self.values).get_mut(identifier).unwrap() = set;
            }
        }
    }

    pub fn initial(game: &Game) -> Self {
        State {
            position: game
                .edges
                .iter()
                .find_map(|edge| {
                    if edge.lhs.is(LABEL_BEGIN) {
                        Some(edge.lhs.clone())
                    } else {
                        None
                    }
                })
                .expect("No begin node found."),
            values: Rc::new(
                game.variables
                    .iter()
                    .map(|(name, variable)| (*name, variable.default.clone()))
                    .collect(),
            ),
        }
    }

    pub fn is_final(&self) -> bool {
        self.position.is(LABEL_END)
    }

    pub fn is_keeper(&self) -> bool {
        self.values
            .get(&LABEL_PLAYER)
            .unwrap()
            .is_element_of(LABEL_KEEPER)
    }

    pub fn is_reachable(&self, game: &Game, position: &EdgeName) -> bool {
        if self.position == *position {
            true
        } else {
            self.next_states(game)
                .any(|state| state.is_reachable(game, position))
        }
    }

    pub fn next_states<'a>(&'a self, game: &'a Game) -> StateNext<'a> {
        StateNext {
            game,
            queue: game
                .edges
                .iter()
                .filter(|edge| edge.lhs == self.position)
                .flat_map(|edge| {
                    edge.generate(&self.position)
                        .into_iter()
                        .map(move |values| (edge, values))
                })
                .collect(),
            reachables: BTreeMap::new(),
            values: self.values.clone(),
        }
    }

    pub fn next_states_n<'a>(
        &'a self,
        game: &'a Game,
        n: usize,
        ignore_keeper: bool,
    ) -> StateNextN<'a> {
        StateNextN {
            game,
            ignore_keeper,
            queue: vec![(self.clone(), n)],
        }
    }
}

pub struct StateNext<'a> {
    game: &'a Game,
    queue: Vec<(&'a Edge, Rc<ValueMap>)>,
    reachables: BTreeMap<(Id, Id), bool>,
    values: Rc<ValueMap>,
}

impl Iterator for StateNext<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        let StateNext {
            game,
            queue,
            reachables,
            values,
        } = self;

        while let Some((edge, binds)) = queue.pop() {
            let mut state = State {
                position: EdgeName {
                    label: edge.rhs.label,
                    types: edge.rhs.types.clone(),
                    values: binds,
                },
                values: values.clone(),
            };

            match &edge.label {
                EdgeLabel::Assignment { lhs, rhs } => {
                    state.eval_set(game, lhs, state.eval(game, rhs).clone());
                    return Some(state);
                }
                EdgeLabel::Comparison { lhs, rhs, negated } => {
                    let lhs_value = state.eval(game, lhs);
                    let rhs_value = state.eval(game, rhs);
                    let is_equal = lhs_value == rhs_value;
                    if is_equal != *negated {
                        return Some(state);
                    }
                }
                EdgeLabel::Reachability { lhs, rhs, negated } => {
                    let is_reachable =
                        *reachables.entry((lhs.label, rhs.label)).or_insert_with(|| {
                            let position = state.position.clone();
                            state.position = lhs.clone();
                            let is_reachable = state.is_reachable(game, rhs);
                            state.position = position;
                            is_reachable
                        });
                    if is_reachable != *negated {
                        return Some(state);
                    }
                }
                EdgeLabel::Skip => {
                    return Some(state);
                }
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.queue.len(), Some(self.queue.len()))
    }
}

pub struct StateNextN<'a> {
    game: &'a Game,
    ignore_keeper: bool,
    queue: Vec<(State, usize)>,
}

impl Iterator for StateNextN<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((state, n)) = self.queue.pop() {
            if n == 0 {
                return Some(state);
            }

            let prev = state.values.get(&LABEL_PLAYER).unwrap();
            let skip = self.ignore_keeper && prev.is_element_of(LABEL_KEEPER);
            for state in state.next_states(self.game) {
                let next = state.values.get(&LABEL_PLAYER).unwrap();
                let depth = if skip || prev == next { n } else { n - 1 };

                self.queue.push((state, depth));
            }
        }

        None
    }
}

pub enum Type {
    Arrow { lhs: Box<Type>, rhs: Box<Type> },
    Set { values: Vec<Rc<Value>> },
}

#[derive(Clone)]
pub enum Value {
    Map {
        default: Rc<Value>,
        values: ValueMap,
    },
    Element {
        value: Id,
    },
}

impl Value {
    pub fn is_element_of(&self, tag: Id) -> bool {
        match self {
            Value::Element { value } => *value == tag,
            Value::Map { .. } => false,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Element { value: a }, Value::Element { value: b }) => a == b,
            (
                Value::Map {
                    default: a_default,
                    values: a_values,
                },
                Value::Map {
                    default: b_default,
                    values: b_values,
                },
            ) if a_default == b_default => a_values == b_values,
            _ => unimplemented!(),
        }
    }
}

pub struct Variable {
    default: Rc<Value>,
    #[allow(dead_code)]
    type_: Box<Type>,
}
