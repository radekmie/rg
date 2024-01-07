use super::unify::Unification;
use crate::ast::{AtomOrVariable, Predicate, Rule, Term};
use std::rc::Rc;

impl<Id: Clone + PartialEq> AtomOrVariable<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        use AtomOrVariable::{Atom, Variable};
        match self {
            Atom(id) => Atom(id.clone()),
            Variable(id) => u.get(id).map_or_else(
                || Variable(id.clone()),
                |term| match term {
                    Term::Custom(Atom(id), arguments) if arguments.is_empty() => Atom(id.clone()),
                    _ => panic!("Cannot substitute non-trivial term for an atom."),
                },
            ),
        }
    }
}

impl<Id: Clone + PartialEq> Predicate<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        Self {
            is_negated: self.is_negated,
            term: Rc::new(self.term.substitute(u)),
        }
    }
}

impl<Id: Clone + PartialEq> Rule<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        Self {
            term: Rc::new(self.term.substitute(u)),
            predicates: self
                .predicates
                .iter()
                .map(|predicate| predicate.substitute(u))
                .collect(),
        }
    }
}

impl<Id: Clone + PartialEq> Term<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        use Term::{Base, Custom, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True};
        match self {
            Base(proposition) => Base(Rc::new(proposition.substitute(u))),
            Custom(AtomOrVariable::Variable(id), arguments) if arguments.is_empty() => {
                u.get(id).unwrap_or(self).clone()
            }
            Custom(name, arguments) => Custom(
                name.substitute(u),
                arguments
                    .iter()
                    .map(|argument| Rc::new(argument.substitute(u)))
                    .collect(),
            ),
            Does(role, action) => Does(role.substitute(u), Rc::new(action.substitute(u))),
            Goal(role, utility) => Goal(role.substitute(u), utility.substitute(u)),
            Init(proposition) => Init(Rc::new(proposition.substitute(u))),
            Input(role, action) => Input(role.substitute(u), Rc::new(action.substitute(u))),
            Legal(role, action) => Legal(role.substitute(u), Rc::new(action.substitute(u))),
            Next(proposition) => Next(Rc::new(proposition.substitute(u))),
            Role(role) => Role(role.substitute(u)),
            Terminal => Terminal,
            True(proposition) => True(Rc::new(proposition.substitute(u))),
        }
    }
}
