use super::unify::Unification;
use crate::ast::{AtomOrVariable, Predicate, Rule, Term};
use std::sync::Arc;

impl<Id: Clone + Ord> AtomOrVariable<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        use AtomOrVariable::{Atom, Variable};
        match self {
            Atom(id) => Atom(id.clone()),
            Variable(id) => u
                .get(id)
                .map_or_else(|| Variable(id.clone()), |id| Atom(id.clone())),
        }
    }
}

impl<Id: Clone + Ord> Predicate<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        Self {
            is_negated: self.is_negated,
            term: Arc::new(self.term.substitute(u)),
        }
    }
}

impl<Id: Clone + Ord> Rule<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        Self {
            term: Arc::new(self.term.substitute(u)),
            predicates: self
                .predicates
                .iter()
                .map(|predicate| predicate.substitute(u))
                .collect(),
        }
    }
}

impl<Id: Clone + Ord> Term<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };
        match self {
            Base(proposition) => Base(Arc::new(proposition.substitute(u))),
            Custom0(name) => Custom0(name.substitute(u)),
            CustomN(name, arguments) => CustomN(
                name.substitute(u),
                arguments
                    .iter()
                    .map(|argument| Arc::new(argument.substitute(u)))
                    .collect(),
            ),
            Does(role, action) => Does(role.substitute(u), Arc::new(action.substitute(u))),
            Goal(role, utility) => Goal(role.substitute(u), utility.substitute(u)),
            Init(proposition) => Init(Arc::new(proposition.substitute(u))),
            Input(role, action) => Input(role.substitute(u), Arc::new(action.substitute(u))),
            Legal(role, action) => Legal(role.substitute(u), Arc::new(action.substitute(u))),
            Next(proposition) => Next(Arc::new(proposition.substitute(u))),
            Role(role) => Role(role.substitute(u)),
            Terminal => Terminal,
            True(proposition) => True(Arc::new(proposition.substitute(u))),
        }
    }
}
