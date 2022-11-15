use crate::{
    Edge, EdgeLabel, Expression, Game, Id, Type, Value, Variable, LABEL_BEGIN, LABEL_END,
    LABEL_KEEPER, LABEL_PLAYER,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, ops::Deref, rc::Rc};

#[derive(Debug, Deserialize, PartialEq)]
struct EdgeSerialized {
    label: EdgeLabelSerialized,
    next: String,
}

#[derive(Debug, Deserialize, PartialEq)]
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
        lhs: String,
        rhs: String,
        negated: bool,
    },
    Skip,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "kind")]
enum ExpressionSerialized {
    Access {
        lhs: Box<ExpressionSerialized>,
        rhs: Box<ExpressionSerialized>,
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

#[derive(Debug, Deserialize, PartialEq)]
pub struct GameSerialized {
    constants: BTreeMap<String, Box<ValueSerialized>>,
    edges: BTreeMap<String, Vec<EdgeSerialized>>,
    types: BTreeMap<String, TypeSerialized>,
    variables: BTreeMap<String, VariableSerialized>,
}

impl GameSerialized {
    pub fn from_ist(source: &str) -> Self {
        serde_json::from_str::<GameSerialized>(source).unwrap()
    }

    pub fn from_ist_file(source_file: &str) -> Self {
        let source = fs::read_to_string(source_file).unwrap();
        GameSerialized::from_ist(&source)
    }
}

impl From<GameSerialized> for Game {
    fn from(game_serialized: GameSerialized) -> Game {
        let mut state = GameSerializedState::new();
        Game {
            constants: game_serialized
                .constants
                .into_iter()
                .map(|(key, value)| (state.intern_string(&key), state.intern_value(&value)))
                .collect(),
            edges: game_serialized
                .edges
                .into_iter()
                .map(|(key, edges)| {
                    (
                        state.intern_string(&key),
                        edges
                            .into_iter()
                            .map(|edge| state.intern_edge(edge))
                            .collect(),
                    )
                })
                .collect(),
            types: game_serialized
                .types
                .into_iter()
                .map(|(key, type_)| (state.intern_string(&key), *state.intern_type(&type_)))
                .collect(),
            variables: game_serialized
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
            label: self.intern_edge_label(edge.label),
            next: self.intern_string(&edge.next),
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
                lhs: self.intern_string(&lhs),
                rhs: self.intern_string(&rhs),
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
            ExpressionSerialized::ConstantReference { identifier } => {
                Expression::ConstantReference {
                    identifier: self.intern_string(identifier),
                }
            }
            ExpressionSerialized::Literal { value } => Expression::Literal {
                value: self.intern_value(value),
            },
            ExpressionSerialized::VariableReference { identifier } => {
                Expression::VariableReference {
                    identifier: self.intern_string(identifier),
                }
            }
        };

        Box::new(expression)
    }

    fn intern_string(&mut self, string: &str) -> Id {
        if self.strings.len() == Id::MAX as usize {
            panic!("Maximum number of interned strings reached! Increase Id size.")
        }

        let next_id = (self.strings.len() + 1) as Id;
        *self.strings.entry(string.to_string()).or_insert(next_id)
    }

    fn intern_type(&mut self, type_: &TypeSerialized) -> Box<Type> {
        let type_ = match type_.deref() {
            TypeSerialized::Arrow { lhs, rhs } => Type::Arrow {
                lhs: self.intern_type(lhs),
                rhs: self.intern_type(rhs),
            },
            TypeSerialized::Set { values } => Type::Set {
                values: values
                    .iter()
                    .map(|value| self.intern_value(value))
                    .collect(),
            },
        };

        Box::new(type_)
    }

    fn intern_value(&mut self, value: &ValueSerialized) -> Rc<Value> {
        let value = match value.deref() {
            ValueSerialized::Element { value } => Value::Element {
                value: self.intern_string(value),
            },
            ValueSerialized::Map { default, values } => Value::Map {
                default: self.intern_value(default),
                values: values
                    .iter()
                    .map(|(key, value)| (self.intern_string(key), self.intern_value(value)))
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

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "kind")]
enum TypeSerialized {
    Arrow {
        lhs: Box<TypeSerialized>,
        rhs: Box<TypeSerialized>,
    },
    Set {
        values: Vec<ValueSerialized>,
    },
}

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Deserialize, PartialEq)]
struct VariableSerialized {
    #[serde(rename = "defaultValue")]
    default: Box<ValueSerialized>,
    #[serde(rename = "type")]
    type_: Box<TypeSerialized>,
}
