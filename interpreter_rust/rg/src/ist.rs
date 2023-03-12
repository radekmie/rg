use map_id::MapId;
use map_id_macro::MapId;
use std::collections::BTreeMap;
use std::rc::Rc;

// Interned strings that the interpreter relies on.
pub type RuntimeId = u16;
pub const LABEL_BEGIN: RuntimeId = 0;
pub const LABEL_END: RuntimeId = 1;
pub const LABEL_GOALS: RuntimeId = 2;
pub const LABEL_KEEPER: RuntimeId = 3;
pub const LABEL_PLAYER: RuntimeId = 4;

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub struct Edge<Id: Ord> {
    pub label: EdgeLabel<Id>,
    pub next: Id,
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub enum EdgeLabel<Id: Ord> {
    Any {
        lhs: Id,
        rhs: Id,
    },
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
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub enum Expression<Id: Ord> {
    Access { lhs: Rc<Self>, rhs: Rc<Self> },
    ConstantReference { identifier: Id },
    Literal { value: Rc<Value<Id>> },
    VariableReference { identifier: Id },
}

impl Expression<RuntimeId> {
    pub fn is_player_reference(&self) -> bool {
        matches!(self, Self::VariableReference { identifier } if *identifier == LABEL_PLAYER)
    }
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub struct Game<Id: Ord> {
    pub constants: BTreeMap<Id, Rc<Value<Id>>>,
    pub edges: BTreeMap<Id, Vec<Edge<Id>>>,
    pub pragmas: Vec<Pragma<Id>>,
    pub types: BTreeMap<Id, Rc<Type<Id>>>,
    pub variables: BTreeMap<Id, Variable<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd)]
pub enum Pragma<Id> {
    Disjoint { edge_name: Id },
}

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub enum Type<Id: Ord> {
    Arrow { lhs: Rc<Self>, rhs: Rc<Self> },
    Set { values: Vec<Rc<Value<Id>>> },
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

#[derive(Clone, Debug, Eq, MapId, PartialEq, PartialOrd, Ord)]
pub struct Variable<Id: Ord> {
    pub default: Rc<Value<Id>>,
    pub type_: Rc<Type<Id>>,
}
