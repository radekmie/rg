use serde::Deserialize;
use std::{collections::HashMap, fs, ops::Deref, rc::Rc};

type RcStr = Rc<str>;
type RcStrHashMap<T> = HashMap<RcStr, Rc<T>>;

#[derive(Clone, Deserialize)]
pub struct Edge {
    lhs: EdgeName,
    rhs: EdgeName,
    label: EdgeLabel,
}

impl Edge {
    pub fn generate(&self) -> Vec<Rc<RcStrHashMap<Value>>> {
        self.rhs.types.iter().fold(
            vec![self.rhs.values.clone()],
            |mut sources, (bind, type_)| {
                if self.rhs.values.contains_key(bind) {
                    sources
                } else if let Some(value) = self.lhs.values.get(bind) {
                    for source in sources.iter_mut() {
                        Rc::make_mut(source).insert(bind.clone(), value.clone());
                    }

                    sources
                } else {
                    match type_.deref() {
                        Type::Arrow { .. } => panic!("Arrow iteration is disallowed."),
                        Type::Set { values } => values
                            .iter()
                            .flat_map(|value| {
                                sources.iter().map(move |source| {
                                    let mut source = source.clone();
                                    Rc::make_mut(&mut source).insert(bind.clone(), value.clone());
                                    source
                                })
                            })
                            .collect(),
                    }
                }
            },
        )
    }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind")]
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

#[derive(Clone, Deserialize)]
pub struct EdgeName {
    label: RcStr,
    types: Rc<RcStrHashMap<Type>>,
    values: Rc<RcStrHashMap<Value>>,
}

impl EdgeName {
    fn is(&self, label: &str) -> bool {
        self.label.deref() == label
    }
}

impl PartialEq for EdgeName {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum Expression {
    Access {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Cast {
        lhs: Type,
        rhs: Box<Expression>,
    },
    BindReference {
        identifier: RcStr,
    },
    ConstantReference {
        identifier: RcStr,
    },
    Literal {
        value: Rc<Value>,
    },
    VariableReference {
        identifier: RcStr,
    },
}

#[derive(Clone, Deserialize)]
pub struct Game {
    constants: RcStrHashMap<Value>,
    edges: Vec<Edge>,
    types: RcStrHashMap<Type>,
    variables: RcStrHashMap<Variable>,
}

impl Game {
    pub fn from_ist(source: &str) -> Self {
        serde_json::from_str::<Game>(&source).unwrap()
    }

    pub fn from_ist_file(source_file: &str) -> Self {
        let source = fs::read_to_string(source_file).unwrap();
        Game::from_ist(&source)
    }
}

#[derive(Clone)]
pub struct State {
    position: EdgeName,
    values: Rc<RcStrHashMap<Value>>,
}

impl State {
    pub fn eval(&self, game: &Game, expression: &Expression) -> Rc<Value> {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => match self.eval(game, lhs).deref() {
                    Value::Map { default, values } => values.get(value).unwrap_or(default).clone(),
                    _ => panic!("Only Map can be accessed."),
                },
                _ => panic!("Only Element can be key."),
            },
            Expression::BindReference { identifier } => {
                self.position.values.get(identifier).unwrap().clone()
            }
            Expression::Cast { rhs, .. } => {
                // TODO: Type check.
                self.eval(game, rhs)
            }
            Expression::ConstantReference { identifier } => {
                game.constants.get(identifier).unwrap().clone()
            }
            Expression::Literal { value } => value.clone(),
            Expression::VariableReference { identifier } => {
                self.values.get(identifier).unwrap().clone()
            }
        }
    }

    pub fn eval_set(&mut self, game: &Game, expression: &Expression, set: Rc<Value>) {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => {
                    let mut map = self.eval(game, lhs);
                    if let Value::Map { default, values } = Rc::make_mut(&mut map) {
                        if &set == default {
                            values.remove(value);
                        } else {
                            values.insert(value.clone(), set);
                        }
                    } else {
                        panic!("Only Map can be accessed.");
                    }

                    self.eval_set(game, lhs, map);
                }
                _ => panic!("Only Element can be key."),
            },
            Expression::BindReference { .. } => panic!("BindReference is immutable."),
            Expression::Cast { .. } => panic!("Cast is immutable."),
            Expression::ConstantReference { .. } => panic!("ConstantReference is immutable."),
            Expression::Literal { .. } => panic!("Literal is immutable."),
            Expression::VariableReference { identifier } => {
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
                    if edge.lhs.is("begin") {
                        Some(edge.lhs.clone())
                    } else {
                        None
                    }
                })
                .expect("No begin node found."),
            values: Rc::new(
                game.variables
                    .iter()
                    .map(|(name, variable)| (name.clone(), variable.default.clone()))
                    .collect(),
            ),
        }
    }

    pub fn is_final(&self) -> bool {
        self.position.is("end")
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
                    edge.generate()
                        .into_iter()
                        .map(move |values| (edge, values))
                })
                .collect(),
            reachables: HashMap::new(),
            values: self.values.clone(),
        }
    }

    pub fn next_states_n<'a>(&'a self, game: &'a Game, n: usize, skip: bool) -> StateNextN<'a> {
        StateNextN {
            game,
            queue: vec![(self.clone(), n)],
            skip,
        }
    }
}

pub struct StateNext<'a> {
    game: &'a Game,
    queue: Vec<(&'a Edge, Rc<RcStrHashMap<Value>>)>,
    reachables: HashMap<(RcStr, RcStr), bool>,
    values: Rc<RcStrHashMap<Value>>,
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
                    label: edge.rhs.label.clone(),
                    types: edge.rhs.types.clone(),
                    values: binds,
                },
                values: values.clone(),
            };

            match &edge.label {
                EdgeLabel::Assignment { lhs, rhs } => {
                    state.eval_set(game, lhs, state.eval(game, rhs));
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
                    let is_reachable = *reachables
                        .entry((lhs.label.clone(), rhs.label.clone()))
                        .or_insert_with(|| {
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
    queue: Vec<(State, usize)>,
    skip: bool,
}

impl Iterator for StateNextN<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((state, n)) = self.queue.pop() {
            let prev = state.values.get("player").unwrap();
            if n == 0 || self.skip && n == 1 && prev.is_element_of("keeper") {
                return Some(state);
            }

            for state in state.next_states(self.game) {
                let next = state.values.get("player").unwrap();
                let depth = if prev == next || next.is_element_of("keeper") {
                    n
                } else {
                    n - 1
                };

                self.queue.push((state, depth));
            }
        }

        None
    }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum Type {
    Arrow { lhs: Box<Type>, rhs: Box<Type> },
    Set { values: Vec<Rc<Value>> },
}

#[derive(Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum Value {
    Map {
        #[serde(rename = "defaultValue")]
        default: Rc<Value>,
        values: RcStrHashMap<Value>,
    },
    Element {
        value: RcStr,
    },
}

impl Value {
    pub fn is_element_of(&self, tag: &str) -> bool {
        match self {
            Value::Element { value } => value.deref() == tag,
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

#[derive(Clone, Deserialize)]
pub struct Variable {
    #[serde(rename = "defaultValue")]
    default: Rc<Value>,
    #[serde(rename = "type")]
    type_: Type,
}
