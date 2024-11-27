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

impl<Id> Action<Id> {
    pub fn assignment(variable: Id, rvalue: RValue<Id>) -> Self {
        Self::Assignment { variable, rvalue }
    }

    pub fn check(negated: bool, rule: Rule<Id>) -> Self {
        Self::Check { negated, rule }
    }

    pub fn comparison(lhs: RValue<Id>, rhs: RValue<Id>, operator: ComparisonOperator) -> Self {
        Self::Comparison { lhs, rhs, operator }
    }

    pub fn off(piece: Id) -> Self {
        Self::Off { piece }
    }

    pub fn on(pieces: Vec<Id>) -> Self {
        Self::On { pieces }
    }

    pub fn shift(label: Id) -> Self {
        Self::Shift { label }
    }

    pub fn switch(player: Option<Id>) -> Self {
        Self::Switch { player }
    }
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

impl<Id> Atom<Id> {
    pub fn action(content: Action<Id>, power: bool) -> Self {
        Self {
            content: ActionOrRule::Action(content),
            power,
        }
    }

    pub fn rule(content: Rule<Id>, power: bool) -> Self {
        Self {
            content: ActionOrRule::Rule(content),
            power,
        }
    }
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

impl<Id> Edge<Id> {
    pub fn new(label: Id, node: Id) -> Self {
        Self { label, node }
    }
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
    pub pieces: Vec<Id>,
    pub variables: Vec<Variable<Id>>,
    pub players: Vec<Variable<Id>>,
    pub board: Vec<Node<Id>>,
    pub rules: Rule<Id>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Node<Id> {
    node: Id,
    piece: Id,
    edges: Vec<Edge<Id>>,
}

impl<Id> Node<Id> {
    pub fn new(node: Id, piece: Id, edges: Vec<Edge<Id>>) -> Self {
        Self { node, piece, edges }
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Rule<Id> {
    pub elements: Vec<Vec<Atom<Id>>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum RValue<Id> {
    Expression(Expression<Id>),
    Number(usize),
    String(Id),
}

impl<Id> RValue<Id> {
    pub fn expression(
        lhs: Arc<Self>,
        rhs: Arc<Self>,
        operator: ExpressionOperator,
    ) -> Self {
        Self::Expression(Expression { lhs, rhs, operator })
    }

    pub fn number(value: usize) -> Self {
        Self::Number(value)
    }

    pub fn string(value: Id) -> Self {
        Self::String(value)
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Variable<Id> {
    name: Id,
    bound: usize,
}

impl<Id> Variable<Id> {
    pub fn new(name: Id, bound: usize) -> Self {
        Self { name, bound }
    }
}
