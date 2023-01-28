use crate::{Edge, EdgeLabel, Expression, Game, IdMap, Type, Value, Variable};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

#[derive(Debug, Deserialize, PartialEq)]
struct EdgeSerialized {
    label: EdgeLabelSerialized,
    next: String,
}

impl EdgeSerialized {
    fn deserialize(self, id_map: &mut IdMap) -> Edge {
        Edge {
            label: self.label.deserialize(id_map),
            next: id_map.intern(&self.next),
        }
    }
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

impl EdgeLabelSerialized {
    fn deserialize(self, id_map: &mut IdMap) -> EdgeLabel {
        match self {
            EdgeLabelSerialized::Assignment { lhs, rhs } => EdgeLabel::Assignment {
                lhs: lhs.deserialize(id_map),
                rhs: rhs.deserialize(id_map),
            },
            EdgeLabelSerialized::Comparison { lhs, rhs, negated } => EdgeLabel::Comparison {
                lhs: lhs.deserialize(id_map),
                rhs: rhs.deserialize(id_map),
                negated,
            },
            EdgeLabelSerialized::Reachability { lhs, rhs, negated } => EdgeLabel::Reachability {
                lhs: id_map.intern(&lhs),
                rhs: id_map.intern(&rhs),
                negated,
            },
            EdgeLabelSerialized::Skip => EdgeLabel::Skip,
        }
    }
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

impl ExpressionSerialized {
    fn deserialize(self, id_map: &mut IdMap) -> Expression {
        match self {
            ExpressionSerialized::Access { lhs, rhs } => Expression::Access {
                lhs: lhs.deserialize(id_map).into(),
                rhs: rhs.deserialize(id_map).into(),
            },
            ExpressionSerialized::ConstantReference { identifier } => {
                Expression::ConstantReference {
                    identifier: id_map.intern(&identifier),
                }
            }
            ExpressionSerialized::Literal { value } => Expression::Literal {
                value: value.deserialize(id_map).into(),
            },
            ExpressionSerialized::VariableReference { identifier } => {
                Expression::VariableReference {
                    identifier: id_map.intern(&identifier),
                }
            }
        }
    }
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

    pub fn deserialize(self) -> Game {
        let mut id_map = IdMap::default();
        Game {
            constants: self
                .constants
                .into_iter()
                .map(|(key, value)| (id_map.intern(&key), value.deserialize(&mut id_map).into()))
                .collect::<BTreeMap<_, _>>()
                .into(),
            edges: self
                .edges
                .into_iter()
                .map(|(key, edges)| {
                    (
                        id_map.intern(&key),
                        edges
                            .into_iter()
                            .map(|edge| edge.deserialize(&mut id_map))
                            .collect(),
                    )
                })
                .collect(),
            types: self
                .types
                .into_iter()
                .map(|(key, type_)| (id_map.intern(&key), type_.deserialize(&mut id_map)))
                .collect(),
            variables: self
                .variables
                .into_iter()
                .map(|(key, variable)| (id_map.intern(&key), variable.deserialize(&mut id_map)))
                .collect(),

            // It has to be moved last.
            id_map,
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

impl TypeSerialized {
    fn deserialize(self, id_map: &mut IdMap) -> Type {
        match self {
            TypeSerialized::Arrow { lhs, rhs } => Type::Arrow {
                lhs: lhs.deserialize(id_map).into(),
                rhs: rhs.deserialize(id_map).into(),
            },
            TypeSerialized::Set { values } => Type::Set {
                values: values
                    .into_iter()
                    .map(|value| value.deserialize(id_map).into())
                    .collect(),
            },
        }
    }
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

impl ValueSerialized {
    fn deserialize(self, id_map: &mut IdMap) -> Value {
        match self {
            ValueSerialized::Element { value } => Value::Element {
                value: id_map.intern(&value),
            },
            ValueSerialized::Map { default, values } => Value::Map {
                default: default.deserialize(id_map).into(),
                values: values
                    .into_iter()
                    .map(|(key, value)| (id_map.intern(&key), value.deserialize(id_map).into()))
                    .collect::<BTreeMap<_, _>>()
                    .into(),
            },
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
struct VariableSerialized {
    #[serde(rename = "defaultValue")]
    default: Box<ValueSerialized>,
    #[serde(rename = "type")]
    type_: Box<TypeSerialized>,
}

impl VariableSerialized {
    fn deserialize(self, id_map: &mut IdMap) -> Variable {
        Variable {
            default: self.default.deserialize(id_map).into(),
            type_: self.type_.deserialize(id_map).into(),
        }
    }
}
