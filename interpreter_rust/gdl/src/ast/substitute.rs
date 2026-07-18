use super::unify::Unification;
use crate::ast::{AtomOrVariable, Predicate, Rule, Term};
use std::sync::Arc;

macro_rules! substitute_one {
    ($x:expr, $u:expr, $fn:expr) => {
        $x.substitute($u).map($fn)
    };
}

macro_rules! substitute_pair {
    ($x:expr, $y:expr, $u:expr, $fn:expr) => {
        match ($x.substitute($u), $y.substitute($u)) {
            (None, None) => None,
            (None, Some(term)) => Some($fn($x.clone(), term)),
            (Some(role), None) => Some($fn(role, $y.clone())),
            (Some(role), Some(term)) => Some($fn(role, term)),
        }
    };
}

macro_rules! substitute_many {
    ($x:expr, $ys:expr, $u:expr) => {{
        let substituted_x = $x.substitute($u);
        let substituted_ys: Vec<_> = $ys.iter().map(|y| y.substitute($u)).collect();
        if substituted_x.is_none() && substituted_ys.iter().all(Option::is_none) {
            return None;
        }

        Some((
            substituted_x.unwrap_or_else(|| $x.clone()),
            substituted_ys
                .into_iter()
                .zip($ys)
                .map(|(substituted_y, y)| substituted_y.unwrap_or_else(|| y.clone()))
                .collect(),
        ))
    }};
}

impl<Id: Clone + Ord> AtomOrVariable<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Option<Self> {
        use AtomOrVariable::{Atom, Variable};
        match self {
            Atom(_) => None,
            Variable(id) => u.get(id).cloned().map(Atom),
        }
    }
}

impl<Id: Clone + Ord> Predicate<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Option<Self> {
        substitute_one!(self.term, u, |term| Self {
            is_negated: self.is_negated,
            term,
        })
    }
}

impl<Id: Clone + Ord> Rule<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Option<Self> {
        substitute_many!(self.term, &self.predicates, u)
            .map(|(term, predicates)| Self { term, predicates })
    }
}

impl<Id: Clone + Ord> Term<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Option<Arc<Self>> {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };
        match self {
            Base(proposition) => substitute_one!(proposition, u, Base),
            Custom0(name) => substitute_one!(name, u, Custom0),
            CustomN(name, arguments) => substitute_many!(name, arguments, u)
                .map(|(name, arguments)| CustomN(name, arguments)),
            Does(role, action) => substitute_pair!(role, action, u, Does),
            Goal(role, utility) => substitute_pair!(role, utility, u, Goal),
            Init(proposition) => substitute_one!(proposition, u, Init),
            Input(role, action) => substitute_pair!(role, action, u, Input),
            Legal(role, action) => substitute_pair!(role, action, u, Legal),
            Next(proposition) => substitute_one!(proposition, u, Next),
            Role(role) => substitute_one!(role, u, Role),
            Terminal => None,
            True(proposition) => substitute_one!(proposition, u, True),
        }
        .map(Arc::new)
    }
}
