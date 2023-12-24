use super::unify::Unification;
use crate::ast::{AtomOrVariable, Predicate, Rule, Term};
use std::rc::Rc;

impl<Id: Clone + Ord> AtomOrVariable<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        use AtomOrVariable::*;
        match self {
            atom @ Atom(_) => atom.clone(),
            var @ Variable(symbol) => match u.get(symbol) {
                Some(term) => match term {
                    Term::Custom(atom @ Atom(_), None) => atom.clone(),
                    _ => panic!("Cannot substitute non-trivial term for an atom."),
                },
                None => var.clone(),
            },
        }
    }
}

impl<Id: Clone + Ord> Predicate<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        Self {
            is_negated: self.is_negated,
            term: Rc::new(self.term.substitute(u)),
        }
    }
}

impl<Id: Clone + Ord> Rule<Id> {
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

impl<Id: Clone + Ord> Term<Id> {
    pub fn substitute(&self, u: &Unification<Id>) -> Self {
        use Term::*;
        match self {
            Base(proposition) => Base(Rc::new(proposition.substitute(u))),
            Custom(AtomOrVariable::Variable(symbol), None) => u.get(symbol).unwrap_or(self).clone(),
            Custom(name, arguments) => Custom(
                name.substitute(u),
                arguments.as_ref().map(|arguments| {
                    arguments
                        .iter()
                        .map(|argument| Rc::new(argument.substitute(u)))
                        .collect()
                }),
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
