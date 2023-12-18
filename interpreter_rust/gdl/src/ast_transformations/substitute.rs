use crate::ast::{AtomOrVariable, Rule, Term};
use std::collections::BTreeMap;
use std::rc::Rc;

type Mapping<Symbol> = BTreeMap<Symbol, Term<Symbol>>;

impl<Symbol: Clone + Ord> AtomOrVariable<Symbol> {
    pub fn substitute(&self, mapping: &Mapping<Symbol>) -> Self {
        use AtomOrVariable::*;
        match self {
            atom @ Atom(_) => atom.clone(),
            var @ Variable(symbol) => match mapping.get(symbol) {
                Some(term) => match term {
                    Term::Custom(atom @ Atom(_), None) => atom.clone(),
                    _ => panic!("Cannot substitute non-trivial term for an atom."),
                },
                None => var.clone(),
            },
        }
    }
}

impl<Symbol: Clone + Ord> Rule<Symbol> {
    pub fn substitute(&self, mapping: &Mapping<Symbol>) -> Self {
        Self {
            term: Rc::new(self.term.substitute(mapping)),
            predicates: self.predicates.as_ref().map(|predicates| {
                predicates
                    .iter()
                    .map(|(is_negated, predicate)| {
                        (*is_negated, Rc::new(predicate.substitute(mapping)))
                    })
                    .collect()
            }),
        }
    }
}

impl<Symbol: Clone + Ord> Term<Symbol> {
    pub fn substitute(&self, mapping: &Mapping<Symbol>) -> Self {
        use Term::*;
        match self {
            Base(proposition) => Base(Rc::new(proposition.substitute(mapping))),
            Custom(AtomOrVariable::Variable(symbol), None) => {
                mapping.get(symbol).unwrap_or(self).clone()
            }
            Custom(name, arguments) => Custom(
                name.substitute(mapping),
                arguments.as_ref().map(|arguments| {
                    arguments
                        .iter()
                        .map(|argument| Rc::new(argument.substitute(mapping)))
                        .collect()
                }),
            ),
            Does(role, action) => Does(
                role.substitute(mapping),
                Rc::new(action.substitute(mapping)),
            ),
            Goal(role, utility) => Goal(role.substitute(mapping), utility.substitute(mapping)),
            Init(proposition) => Init(Rc::new(proposition.substitute(mapping))),
            Input(role, action) => Input(
                role.substitute(mapping),
                Rc::new(action.substitute(mapping)),
            ),
            Legal(role, action) => Legal(
                role.substitute(mapping),
                Rc::new(action.substitute(mapping)),
            ),
            Next(proposition) => Next(Rc::new(proposition.substitute(mapping))),
            Role(role) => Role(role.substitute(mapping)),
            Terminal => Terminal,
            True(proposition) => True(Rc::new(proposition.substitute(mapping))),
        }
    }
}
