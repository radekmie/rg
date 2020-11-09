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
    pub fn generate(&self) -> Vec<RcStrHashMap<Value>> {
        self.rhs
            .types
            .iter()
            .fold(vec![HashMap::new()], |mut sources, (bind, type_)| {
                if let Some(value) = self
                    .rhs
                    .values
                    .get(bind)
                    .or_else(|| self.lhs.values.get(bind))
                {
                    for source in sources.iter_mut() {
                        source.insert(bind.clone(), Rc::clone(value));
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
                                    source.insert(bind.clone(), Rc::clone(value));
                                    source
                                })
                            })
                            .collect(),
                    }
                }
            })
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
        mode: ReachabilityMode,
    },
    Skip,
}

#[derive(Clone, Deserialize)]
pub struct EdgeName {
    label: RcStr,
    types: RcStrHashMap<Type>,
    values: RcStrHashMap<Value>,
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

#[derive(Clone, Deserialize)]
pub enum ReachabilityMode {
    #[serde(rename = "not")]
    Not,
    #[serde(rename = "rev")]
    Rev,
}

#[derive(Clone)]
pub struct State {
    position: EdgeName,
    values: RcStrHashMap<Value>,
}

impl State {
    pub fn eval(&self, game: &Game, expression: &Expression) -> Rc<Value> {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => match self.eval(game, lhs).deref() {
                    Value::Map {
                        default_value,
                        values,
                    } => Rc::clone(values.get(value).unwrap_or(default_value)),
                    _ => panic!("Only Map can be accessed."),
                },
                _ => panic!("Only Element can be key."),
            },
            Expression::BindReference { identifier } => {
                Rc::clone(self.position.values.get(identifier).unwrap())
            }
            Expression::Cast { rhs, .. } => {
                // TODO: Type check.
                self.eval(game, rhs)
            }
            Expression::ConstantReference { identifier } => {
                Rc::clone(game.constants.get(identifier).unwrap())
            }
            Expression::Literal { value } => Rc::clone(value),
            Expression::VariableReference { identifier } => {
                Rc::clone(self.values.get(identifier).unwrap())
            }
        }
    }

    pub fn eval_set(&mut self, game: &Game, expression: &Expression, set: Rc<Value>) {
        match expression {
            Expression::Access { lhs, rhs } => match self.eval(game, rhs).deref() {
                Value::Element { value } => match self.eval(game, lhs).deref() {
                    Value::Map {
                        default_value,
                        values,
                    } => self.eval_set(
                        game,
                        lhs,
                        Rc::from(Value::Map {
                            default_value: Rc::clone(default_value),
                            values: {
                                let mut values = values.clone();
                                if &set == default_value {
                                    values.remove(value);
                                } else {
                                    values.insert(value.clone(), set);
                                }
                                values
                            },
                        }),
                    ),
                    _ => panic!("Only Map can be accessed."),
                },
                _ => panic!("Only Element can be key."),
            },
            Expression::BindReference { .. } => panic!("BindReference is immutable."),
            Expression::Cast { .. } => panic!("Cast is immutable."),
            Expression::ConstantReference { .. } => panic!("ConstantReference is immutable."),
            Expression::Literal { .. } => panic!("Literal is immutable."),
            Expression::VariableReference { identifier } => {
                *self.values.get_mut(identifier).unwrap() = set;
            }
        }
    }

    pub fn initial(game: &Game) -> Self {
        State {
            position: game
                .edges
                .iter()
                .find_map(|edge| {
                    // TODO: Check binds.
                    if edge.lhs.label.deref() == "begin" {
                        Some(edge.lhs.clone())
                    } else {
                        None
                    }
                })
                .expect("No begin node found."),
            values: game
                .variables
                .iter()
                .map(|(name, variable)| (name.clone(), Rc::clone(&variable.default_value)))
                .collect(),
        }
    }

    pub fn is_final(&self) -> bool {
        self.position.label.deref() == "end"
    }

    pub fn is_reachable(&self, game: &Game, position: &EdgeName) -> bool {
        // TODO: Check binds.
        if self.position.label == position.label {
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
                .filter(|edge| edge.lhs.label == self.position.label)
                .flat_map(|edge| {
                    edge.generate()
                        .into_iter()
                        .map(move |values| (edge, values))
                })
                .collect(),
            spent: vec![],
            state: self,
        }
    }

    pub fn next_states_n<'a>(&'a self, game: &'a Game, n: usize) -> StateNextN<'a> {
        StateNextN {
            game,
            queue: vec![(self.clone(), n)],
        }
    }
}

pub struct StateNext<'a> {
    game: &'a Game,
    queue: Vec<(&'a Edge, RcStrHashMap<Value>)>,
    spent: Vec<State>,
    state: &'a State,
}

impl Iterator for StateNext<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        self.spent.pop().or_else(|| {
            self.queue.pop().and_then(|(edge, values)| {
                let mut state = State {
                    position: EdgeName {
                        label: edge.rhs.label.clone(),
                        types: edge.rhs.types.clone(),
                        values,
                    },
                    values: self.state.values.clone(),
                };

                let spent = match &edge.label {
                    EdgeLabel::Assignment { lhs, rhs } => {
                        state.eval_set(self.game, lhs, state.eval(self.game, rhs));
                        vec![state]
                    }
                    EdgeLabel::Comparison { lhs, rhs, negated } => {
                        let lhs_value = state.eval(self.game, lhs);
                        let rhs_value = state.eval(self.game, rhs);
                        let equal = lhs_value == rhs_value;
                        if equal == *negated {
                            vec![]
                        } else {
                            vec![state]
                        }
                    }
                    EdgeLabel::Reachability { lhs, rhs, mode } => {
                        let position = state.position;

                        state.position = lhs.clone();
                        let is_reachable = state.is_reachable(self.game, rhs);
                        state.position = position;

                        match mode {
                            ReachabilityMode::Not => {
                                if !is_reachable {
                                    vec![state]
                                } else {
                                    vec![]
                                }
                            }
                            ReachabilityMode::Rev => {
                                if is_reachable {
                                    vec![state]
                                } else {
                                    vec![]
                                }
                            }
                        }
                    }
                    EdgeLabel::Skip => vec![state],
                };

                self.spent = spent;
                self.next()
            })
        })
    }
}

pub struct StateNextN<'a> {
    game: &'a Game,
    queue: Vec<(State, usize)>,
}

impl Iterator for StateNextN<'_> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop().and_then(|(state, n)| {
            if n == 0 {
                Some(state)
            } else {
                let prev = state.values.get("player").unwrap();
                for state in state.next_states(self.game) {
                    let next = state.values.get("player").unwrap();
                    let depth = if prev == next || next.is_element_of("keeper") {
                        n
                    } else {
                        n - 1
                    };

                    self.queue.push((state, depth));
                }
                self.next()
            }
        })
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
        default_value: Rc<Value>,
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
                    default_value: a_default_value,
                    values: a_values,
                },
                Value::Map {
                    default_value: b_default_value,
                    values: b_values,
                },
            ) if a_default_value == b_default_value => a_values == b_values,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct Variable {
    #[serde(rename = "defaultValue")]
    default_value: Rc<Value>,
    #[serde(rename = "type")]
    type_: Type,
}
