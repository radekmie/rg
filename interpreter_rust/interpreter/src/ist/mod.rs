mod display;
mod from;
mod state;
pub mod tools;

use map_id::MapId;
use map_id_macro::MapId;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

// Interned strings that the interpreter relies on.
pub type RuntimeId = u16;
pub const LABEL_BEGIN: RuntimeId = 0;
pub const LABEL_END: RuntimeId = 1;
pub const LABEL_KEEPER: RuntimeId = 2;

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub struct Edge<Id: Ord> {
    pub label: EdgeLabel<Id>,
    pub next: Id,
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub enum EdgeLabel<Id: Ord> {
    Assignment {
        lhs: Expression<Id>,
        rhs: Expression<Id>,
    },
    Comparison {
        lhs: Expression<Id>,
        rhs: Expression<Id>,
        negated: bool,
    },
    Reachability {
        lhs: Id,
        rhs: Id,
        negated: bool,
    },
    Skip,
    Tag {
        symbol: Id,
    },
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub enum Expression<Id: Ord> {
    Access { lhs: Rc<Self>, rhs: Rc<Self> },
    ConstantReference { index: usize },
    GoalsReference,
    Literal { value: Rc<Value<Id>> },
    PlayerReference,
    VariableReference { index: usize },
    VisibleReference,
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub struct Game<Id: Ord> {
    pub constants: Vec<Rc<Value<Id>>>,
    pub edges: BTreeMap<Id, Vec<Edge<Id>>>,
    pub initial_goals: Rc<Value<Id>>,
    pub initial_player: Rc<Value<Id>>,
    pub initial_values: Rc<Vec<Rc<Value<Id>>>>,
    pub initial_visible: Rc<Value<Id>>,
    pub repeats: BTreeMap<Id, Rc<Vec<usize>>>,
    pub uniques: BTreeSet<Id>,
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub enum Type<Id: Ord> {
    Arrow { lhs: Rc<Self>, rhs: Rc<Self> },
    Set { values: Vec<Rc<Value<Id>>> },
}

impl<Id: Ord> Type<Id> {
    pub fn size(&self) -> usize {
        match self {
            Self::Arrow { lhs, rhs } => lhs.size() * rhs.size(),
            Self::Set { values } => values.len(),
        }
    }
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub enum Value<Id: Ord> {
    Element {
        value: Id,
    },
    Map {
        default: Rc<Self>,
        values: Rc<BTreeMap<Id, Rc<Self>>>,
    },
}

impl Value<RuntimeId> {
    pub fn is_keeper(&self) -> bool {
        matches!(self, Self::Element { value } if *value == LABEL_KEEPER)
    }
}
