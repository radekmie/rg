pub mod deserializer;
pub mod rg;
pub mod utils;

// Below code should be moved into rg::ist module.

use regex::{Captures, Regex};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::ops::Deref;
use std::rc::Rc;

// We assume that there is not _a lot_ of unique symbols.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Id(u16);

// Interned strings that the interpreter relies on.
const LABEL_BEGIN: Id = Id(0);
const LABEL_KEEPER: Id = Id(1);
const LABEL_PLAYER: Id = Id(2);

type ValueMap = Rc<BTreeMap<Id, Rc<Value>>>;

#[derive(Debug)]
pub struct Edge {
    label: EdgeLabel,
    next: Id,
}

#[derive(Debug)]
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
        lhs: Id,
        rhs: Id,
        negated: bool,
    },
    Skip,
}

#[derive(Debug)]
pub enum Expression {
    Access {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
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

impl Expression {
    pub fn is_player_reference(&self) -> bool {
        matches!(self, Self::VariableReference { identifier } if *identifier == LABEL_PLAYER)
    }
}

#[derive(Debug)]
pub struct Game {
    constants: ValueMap,
    edges: BTreeMap<Id, Vec<Edge>>,
    id_map: IdMap,
    #[allow(dead_code)]
    types: BTreeMap<Id, Type>,
    variables: BTreeMap<Id, Variable>,
}

impl Game {
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

#[derive(Debug)]
pub struct IdMap {
    // TODO: Maybe `Rc<String>` would be better?
    id_to_string: BTreeMap<Id, String>,
    string_to_id: BTreeMap<String, Id>,
}

impl IdMap {
    pub fn intern(&mut self, string: &str) -> Id {
        if let Some(id) = self.string_to_id.get(string) {
            return *id;
        }

        const ERROR: &str = "Maximum number of interned strings reached! Increase Id size.";
        let id = Id(self
            .string_to_id
            .len()
            .checked_add(1)
            .expect(ERROR)
            .try_into()
            .expect(ERROR));
        self.intern_as(string, id)
    }

    fn intern_as(&mut self, string: &str, id: Id) -> Id {
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

impl Default for IdMap {
    fn default() -> Self {
        let mut id_map = Self {
            id_to_string: Default::default(),
            string_to_id: Default::default(),
        };

        id_map.intern_as("begin", LABEL_BEGIN);
        id_map.intern_as("keeper", LABEL_KEEPER);
        id_map.intern_as("player", LABEL_PLAYER);
        id_map
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct State {
    position: Id,
    values: ValueMap,
}

impl State {
    pub fn clone_at(&self, position: Id) -> Self {
        Self {
            position,
            values: self.values.clone(),
        }
    }

    pub fn eval<'a>(&'a self, game: &'a Game, expression: &'a Expression) -> &'a Rc<Value> {
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

    pub fn eval_set(&mut self, game: &Game, expression: &Expression, set: Rc<Value>) {
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

    pub fn get_player(&self) -> &Rc<Value> {
        self.values.get(&LABEL_PLAYER).unwrap()
    }

    pub fn is_reachable(&self, game: &Game, position: Id) -> bool {
        self.next_states(game, false)
            .any(|state| state.position == position)
    }

    pub fn next_states<'a>(&'a self, game: &'a Game, break_on_player: bool) -> StateNext<'a> {
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
        game: &'a Game,
        depth: usize,
        ignore_keeper: bool,
    ) -> StateNextDepth<'a> {
        StateNextDepth {
            game,
            ignore_keeper,
            queue: vec![(self.clone(), depth)],
        }
    }

    pub fn serialize(&self, game: &Game) -> String {
        let string = format!("{:?}", self);

        // Replace `Id`s with full names.
        let id_regex = Regex::new(r"Id\(\s*(\d+),?\s*\)").unwrap();
        let string = id_regex
            .replace_all(string.as_str(), |captures: &Captures| {
                captures
                    .get(1)
                    .map(|id| Id(id.as_str().parse().unwrap()))
                    .and_then(|id| game.id_map.recall(&id))
                    .unwrap()
            })
            .to_string();

        // Shorten `Element`s into their values.
        let element_regex = Regex::new(r"Element\s*\{\s*value:\s*(.*?),?\s*\}").unwrap();
        let string = element_regex
            .replace_all(string.as_str(), |captures: &Captures| {
                captures
                    .get(1)
                    .map(|value| value.as_str().to_string())
                    .unwrap()
            })
            .to_string();

        string
    }
}

pub struct StateNext<'a> {
    break_on_player: bool,
    game: &'a Game,
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
                    let mut reachables: Option<BTreeMap<(Id, Id), bool>> = None;
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
    game: &'a Game,
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

#[derive(Debug)]
pub enum Type {
    Arrow { lhs: Rc<Type>, rhs: Rc<Type> },
    Set { values: Vec<Rc<Value>> },
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
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
    pub fn is_keeper(&self) -> bool {
        matches!(self, Self::Element { value } if *value == LABEL_KEEPER)
    }
}

#[derive(Debug)]
pub struct Variable {
    default: Rc<Value>,
    #[allow(dead_code)]
    type_: Rc<Type>,
}

use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen]
pub fn parse_rg(source: &str) -> Result<String, JsValue> {
    use nom::combinator::all_consuming;
    use nom::error::convert_error;
    use nom::Finish;
    use rg::parser::game_declaration;

    // Parsing comments would require far more complex grammar (and parser),
    // because a comment can occur basically everywhere.
    let comment_regex = Regex::new(r"(//.*?)(\n|$)").unwrap();
    let source = comment_regex.replace_all(source, |captures: &Captures| {
        captures
            .get(1)
            .map(|comment| {
                format!(
                    "{:indent$}{}",
                    "",
                    captures.get(2).map_or("", |newline| newline.as_str()),
                    indent = comment.as_str().len()
                )
            })
            .unwrap()
    });

    let result = match all_consuming(game_declaration)(&source).finish() {
        Ok((_, game_declaration)) => match serde_json::to_string(game_declaration.deref()) {
            Ok(json) => Ok(json),
            Err(error) => Err(error.to_string().into()),
        },
        Err(error) => Err(convert_error(source.deref(), error).into()),
    };

    result
}
