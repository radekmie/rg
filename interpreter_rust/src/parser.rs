use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Deserialize, Serialize)]
pub struct Edge {
    label: Rc<EdgeLabel>,
    next: Id,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum EdgeLabel {
    Assignment {
        lhs: Rc<Expression>,
        rhs: Rc<Expression>,
    },
    Comparison {
        lhs: Rc<Expression>,
        rhs: Rc<Expression>,
        negated: bool,
    },
    Reachability {
        lhs: Id,
        rhs: Id,
        negated: bool,
    },
    Skip,
}

#[derive(Deserialize, Serialize)]
pub struct EdgeName {
    parts: Vec<EdgeNamePart>,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum EdgeNamePart {
    Binding {
        identifier: Id,
        #[serde(rename = "type")]
        type_: Rc<Type>,
    },
    Literal {
        identifier: Id,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Expression {
    Access {
        lhs: Rc<Expression>,
        rhs: Rc<Expression>,
    },
    Cast {
        lhs: Rc<Type>,
        rhs: Rc<Expression>,
    },
    Reference {
        identifier: Id,
    },
}

#[derive(Deserialize, Serialize)]
pub struct Game {
    constants: BTreeMap<Id, Rc<Value>>,
    edges: BTreeMap<Id, Vec<Rc<Edge>>>,
    types: BTreeMap<Id, Rc<Type>>,
    variables: BTreeMap<Id, Rc<Variable>>,
    // #[serde(skip)] id_map: IdMap,
}

#[derive(Clone, Copy, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Id(u16);

#[test]
fn foo() {
    let x = Id(1);
    assert_eq!(serde_json::to_string(&x).unwrap(), "".to_string());
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Type {
    Arrow { lhs: Id, rhs: Rc<Type> },
    Set { identifiers: Vec<Id> },
    TypeReference { identifier: Id },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum Value {
    Element { identifier: Id },
    Map { entries: Vec<ValueEntry> },
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "kind")]
pub enum ValueEntry {
    DefaultEntry { value: Rc<Value> },
    NamedEntry { identifier: Id, value: Rc<Value> },
}

#[derive(Deserialize, Serialize)]
pub struct Variable {
    default: Rc<Value>,
    #[serde(rename = "type")]
    type_: Rc<Type>,
}
