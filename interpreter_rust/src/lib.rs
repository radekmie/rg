use serde::Deserialize;
use std::{collections::BTreeMap, fs, ops::Deref, rc::Rc};

// We assume that there is not _a lot_ of unique symbols.
type Id = u8;

// Interned strings that the interpreter relies on.
const LABEL_BEGIN: Id = 0;
const LABEL_END: Id = 1;
const LABEL_KEEPER: Id = 2;
const LABEL_PLAYER: Id = 3;

pub struct Edge {
    lhs: EdgeName,
    rhs: EdgeName,
    label: EdgeLabel,
}

impl Edge {
    pub fn generate(&self) -> Vec<Rc<BTreeMap<Id, Rc<Value>>>> {
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

#[derive(Deserialize)]
struct EdgeSerialized {
    lhs: EdgeNameSerialized,
    rhs: EdgeNameSerialized,
    label: EdgeLabelSerialized,
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

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum EdgeLabelSerialized {
    Assignment {
        lhs: ExpressionSerialized,
        rhs: ExpressionSerialized,
    },
    Comparison {
        lhs: ExpressionSerialized,
        rhs: ExpressionSerialized,
        negated: bool,
    },
    Reachability {
        lhs: EdgeNameSerialized,
        rhs: EdgeNameSerialized,
        negated: bool,
    },
    Skip,
}

#[derive(Clone)]
pub struct EdgeName {
    label: Id,
    types: Rc<BTreeMap<Id, Type>>,
    values: Rc<BTreeMap<Id, Rc<Value>>>,
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

#[derive(Deserialize)]
struct EdgeNameSerialized {
    label: String,
    types: BTreeMap<String, TypeSerialized>,
    values: BTreeMap<String, Box<ValueSerialized>>,
}

pub enum Expression {
    Access {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Cast {
        lhs: Box<Type>,
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

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum ExpressionSerialized {
    Access {
        lhs: Box<ExpressionSerialized>,
        rhs: Box<ExpressionSerialized>,
    },
    Cast {
        lhs: TypeSerialized,
        rhs: Box<ExpressionSerialized>,
    },
    BindReference {
        identifier: String,
    },
    ConstantReference {
        identifier: String,
    },
    Literal {
        value: Box<ValueSerialized>,
    },
    VariableReference {
        identifier: String,
    },
}

pub struct Game {
    constants: BTreeMap<Id, Rc<Value>>,
    edges: Vec<Edge>,
    types: BTreeMap<Id, Type>,
    variables: BTreeMap<Id, Variable>,
}

impl Game {
    pub fn from_ist(source: &str) -> Self {
        serde_json::from_str::<GameSerialized>(&source)
            .unwrap()
            .into()
    }

    pub fn from_ist_file(source_file: &str) -> Self {
        let source = fs::read_to_string(source_file).unwrap();
        Game::from_ist(&source)
    }
}

#[derive(Deserialize)]
pub struct GameSerialized {
    constants: BTreeMap<String, Box<ValueSerialized>>,
    edges: Vec<EdgeSerialized>,
    types: BTreeMap<String, TypeSerialized>,
    variables: BTreeMap<String, VariableSerialized>,
}

impl Into<Game> for GameSerialized {
    fn into(self) -> Game {
        let mut state = GameSerializedState::new();
        Game {
            constants: self
                .constants
                .into_iter()
                .map(|(key, value)| (state.intern_string(&key), state.intern_value(&value)))
                .collect(),
            edges: self
                .edges
                .into_iter()
                .map(|edge| state.intern_edge(edge))
                .collect(),
            types: self
                .types
                .into_iter()
                .map(|(key, type_)| (state.intern_string(&key), *state.intern_type(&type_)))
                .collect(),
            variables: self
                .variables
                .into_iter()
                .map(|(key, variable)| {
                    (state.intern_string(&key), state.intern_variable(&variable))
                })
                .collect(),
        }
    }
}

struct GameSerializedState {
    strings: BTreeMap<String, Id>,
}

impl GameSerializedState {
    fn intern_edge(&mut self, edge: EdgeSerialized) -> Edge {
        Edge {
            lhs: self.intern_edge_name(edge.lhs),
            rhs: self.intern_edge_name(edge.rhs),
            label: self.intern_edge_label(edge.label),
        }
    }

    fn intern_edge_name(&mut self, edge_name: EdgeNameSerialized) -> EdgeName {
        EdgeName {
            label: self.intern_string(&edge_name.label),
            types: Rc::new(
                edge_name
                    .types
                    .into_iter()
                    .map(|(key, type_)| (self.intern_string(&key), *self.intern_type(&type_)))
                    .collect(),
            ),
            values: Rc::new(
                edge_name
                    .values
                    .into_iter()
                    .map(|(key, value)| (self.intern_string(&key), self.intern_value(&value)))
                    .collect(),
            ),
        }
    }

    fn intern_edge_label(&mut self, edge_label: EdgeLabelSerialized) -> EdgeLabel {
        match edge_label {
            EdgeLabelSerialized::Assignment { lhs, rhs } => EdgeLabel::Assignment {
                lhs: *self.intern_expression(&lhs),
                rhs: *self.intern_expression(&rhs),
            },
            EdgeLabelSerialized::Comparison { lhs, rhs, negated } => EdgeLabel::Comparison {
                lhs: *self.intern_expression(&lhs),
                rhs: *self.intern_expression(&rhs),
                negated,
            },
            EdgeLabelSerialized::Reachability { lhs, rhs, negated } => EdgeLabel::Reachability {
                lhs: self.intern_edge_name(lhs),
                rhs: self.intern_edge_name(rhs),
                negated,
            },
            EdgeLabelSerialized::Skip => EdgeLabel::Skip,
        }
    }

    fn intern_expression(&mut self, expression: &ExpressionSerialized) -> Box<Expression> {
        let expression = match expression {
            ExpressionSerialized::Access { lhs, rhs } => Expression::Access {
                lhs: self.intern_expression(lhs),
                rhs: self.intern_expression(rhs),
            },
            ExpressionSerialized::BindReference { identifier } => Expression::BindReference {
                identifier: self.intern_string(&identifier),
            },
            ExpressionSerialized::Cast { lhs, rhs } => Expression::Cast {
                lhs: self.intern_type(&lhs),
                rhs: self.intern_expression(rhs),
            },
            ExpressionSerialized::ConstantReference { identifier } => {
                Expression::ConstantReference {
                    identifier: self.intern_string(&identifier),
                }
            }
            ExpressionSerialized::Literal { value } => Expression::Literal {
                value: self.intern_value(&value),
            },
            ExpressionSerialized::VariableReference { identifier } => {
                Expression::VariableReference {
                    identifier: self.intern_string(&identifier),
                }
            }
        };

        Box::new(expression)
    }

    fn intern_string(&mut self, string: &String) -> Id {
        let next_id = (self.strings.len() + 1) as Id;
        self.strings
            .entry(string.clone())
            .or_insert(next_id)
            .clone()
    }

    fn intern_type(&mut self, type_: &TypeSerialized) -> Box<Type> {
        let type_ = match type_.deref() {
            TypeSerialized::Arrow { lhs, rhs } => Type::Arrow {
                lhs: self.intern_type(lhs),
                rhs: self.intern_type(rhs),
            },
            TypeSerialized::Set { values } => Type::Set {
                values: values
                    .into_iter()
                    .map(|value| self.intern_value(&value))
                    .collect(),
            },
        };

        Box::new(type_)
    }

    fn intern_value(&mut self, value: &ValueSerialized) -> Rc<Value> {
        let value = match value.deref() {
            ValueSerialized::Element { value } => Value::Element {
                value: self.intern_string(&value),
            },
            ValueSerialized::Map { default, values } => Value::Map {
                default: self.intern_value(default),
                values: values
                    .into_iter()
                    .map(|(key, value)| (self.intern_string(&key), self.intern_value(value)))
                    .collect(),
            },
        };

        Rc::new(value)
    }

    fn intern_variable(&mut self, variable: &VariableSerialized) -> Variable {
        Variable {
            default: self.intern_value(&variable.default),
            type_: self.intern_type(&variable.type_),
        }
    }

    fn new() -> Self {
        GameSerializedState {
            strings: vec![
                ("begin".into(), LABEL_BEGIN),
                ("end".into(), LABEL_END),
                ("keeper".into(), LABEL_KEEPER),
                ("player".into(), LABEL_PLAYER),
            ]
            .into_iter()
            .collect(),
        }
    }
}

#[derive(Clone)]
pub struct State {
    position: EdgeName,
    values: Rc<BTreeMap<Id, Rc<Value>>>,
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
            Expression::Cast { rhs, .. } => {
                // TODO: Type check.
                self.eval(game, rhs)
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
                            values.insert(value.clone(), set.clone());
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
                    .map(|(name, variable)| (name.clone(), variable.default.clone()))
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
                    edge.generate()
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
    queue: Vec<(&'a Edge, Rc<BTreeMap<Id, Rc<Value>>>)>,
    reachables: BTreeMap<(Id, Id), bool>,
    values: Rc<BTreeMap<Id, Rc<Value>>>,
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

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum TypeSerialized {
    Arrow {
        lhs: Box<TypeSerialized>,
        rhs: Box<TypeSerialized>,
    },
    Set {
        values: Vec<Box<ValueSerialized>>,
    },
}

#[derive(Clone)]
pub enum Value {
    Map {
        default: Rc<Value>,
        values: BTreeMap<Id, Rc<Value>>,
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

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum ValueSerialized {
    Map {
        #[serde(rename = "defaultValue")]
        default: Box<ValueSerialized>,
        values: BTreeMap<String, Box<ValueSerialized>>,
    },
    Element {
        value: String,
    },
}

pub struct Variable {
    default: Rc<Value>,
    type_: Box<Type>,
}

#[derive(Deserialize)]
struct VariableSerialized {
    #[serde(rename = "defaultValue")]
    default: Box<ValueSerialized>,
    #[serde(rename = "type")]
    type_: Box<TypeSerialized>,
}
