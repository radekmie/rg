mod display;

use map_id::MapId;
use map_id_macro::MapId;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Action<Id> {
    Assignment {
        variable: Id,
        rvalue: RValue<Id>,
    },
    Check {
        negated: bool,
        rule: Rule<Id>,
    },
    Comparison {
        lhs: RValue<Id>,
        rhs: RValue<Id>,
        operator: ComparisonOperator,
    },
    Off {
        piece: Id,
    },
    On {
        pieces: Vec<Id>,
    },
    Shift {
        label: Id,
    },
    Switch {
        player: Option<Id>,
    },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ActionOrRule<Id> {
    Action(Action<Id>),
    Rule(Rule<Id>),
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Atom<Id> {
    content: ActionOrRule<Id>,
    power: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ComparisonOperator {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Ne,
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for ComparisonOperator {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Edge<Id> {
    label: Id,
    node: Id,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error<Id> {
    Todo(Id),
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Expression<Id> {
    lhs: Arc<RValue<Id>>,
    rhs: Arc<RValue<Id>>,
    operator: ExpressionOperator,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ExpressionOperator {
    Add,
    Div,
    Mul,
    Sub,
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for ExpressionOperator {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Game<Id> {
    pieces: Vec<Id>,
    variables: Vec<Variable<Id>>,
    players: Vec<Variable<Id>>,
    board: Vec<Node<Id>>,
    rules: Rule<Id>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Node<Id> {
    node: Id,
    piece: Id,
    edges: Vec<Edge<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Rule<Id> {
    elements: Vec<Vec<Atom<Id>>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum RValue<Id> {
    Expression(Expression<Id>),
    Number(usize),
    String(Id),
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Variable<Id> {
    name: Id,
    bound: usize,
}
