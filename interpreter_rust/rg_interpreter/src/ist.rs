use map_id::MapId;
use map_id_macro::MapId;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

// Interned strings that the interpreter relies on.
pub type RuntimeId = u32;
pub const LABEL_BEGIN: RuntimeId = 0;
pub const LABEL_END: RuntimeId = 1;
pub const LABEL_KEEPER: RuntimeId = 2;
pub const LABEL_RANDOM: RuntimeId = 3;

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
    AssignmentAny {
        lhs: Expression<Id>,
        rhs: Rc<Type<Id>>,
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
    TagVariable {
        index: usize,
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
    /// Nodes marked as `@disjoint` or `@disjointExhaustive` without other successors.
    pub disjoints: BTreeSet<Id>,
    pub edges: BTreeMap<Id, Vec<Edge<Id>>>,
    pub initial_goals: Rc<Value<Id>>,
    pub initial_player: Rc<Value<Id>>,
    pub initial_values: Rc<Vec<Rc<Value<Id>>>>,
    pub initial_visible: Rc<Value<Id>>,
    /// Nodes marked as `@repeat` with their variables.
    pub repeats: BTreeMap<Id, Rc<Vec<usize>>>,
    /// Nodes marked as `@unique`.
    pub uniques: Uniques<Id>,
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
pub enum Uniques<Id: Ord> {
    Negative(BTreeSet<Id>),
    NegativeAll,
    Positive(BTreeSet<Id>),
    PositiveAll,
}

impl<Id: Ord> Default for Uniques<Id> {
    fn default() -> Self {
        Self::Positive(BTreeSet::default())
    }
}

impl<Id: Ord + std::fmt::Debug> Uniques<Id> {
    pub fn contains(&self, value: &Id) -> bool {
        match self {
            Self::Negative(nodes) => !nodes.contains(value),
            Self::NegativeAll => false,
            Self::Positive(nodes) => nodes.contains(value),
            Self::PositiveAll => true,
        }
    }

    pub fn insert(&mut self, value: Id) {
        match self {
            Self::Negative(_) | Self::NegativeAll | Self::PositiveAll => unreachable!(),
            Self::Positive(nodes) => nodes.insert(value),
        };
    }

    /// Given all nodes, choose the optimal (smaller) variant.
    pub fn optimize(&mut self, mut all_nodes: BTreeSet<Id>) {
        match self {
            Self::Negative(_) | Self::NegativeAll | Self::PositiveAll => unreachable!(),
            Self::Positive(nodes) => {
                if nodes.is_empty() {
                    *self = Self::NegativeAll;
                } else if all_nodes.is_subset(nodes) {
                    *self = Self::PositiveAll;
                } else if nodes.len() > all_nodes.len() / 2 {
                    all_nodes.retain(|value| !nodes.contains(value));
                    *self = Self::Negative(all_nodes);
                }
            }
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

    pub fn is_random(&self) -> bool {
        matches!(self, Self::Element { value } if *value == LABEL_RANDOM)
    }

    pub fn is_system(&self) -> bool {
        matches!(self, Self::Element { value } if *value == LABEL_KEEPER || *value == LABEL_RANDOM)
    }
}
