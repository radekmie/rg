use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::iter::repeat_n;
use std::sync::Arc;
use utils::cartesian::cartesian;

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn expand_ors(self, or: &Id) -> Self {
        let rules = Vec::with_capacity(self.0.len());
        Self(self.0.into_iter().fold(rules, |mut rules, rule| {
            match rule.expand_ors(or) {
                Err(rule) => rules.push(rule),
                Ok(expanded) => rules.extend(expanded),
            }

            rules
        }))
    }
}

impl<Id: Clone + PartialEq> Predicate<Id> {
    pub fn expand_ors(self, or: &Id) -> Result<Vec<Self>, Self> {
        let Self { term, is_negated } = self;
        match term.expand_ors(or) {
            Err(term) => Err(Self { term, is_negated }),
            Ok(terms) => Ok(terms
                .into_iter()
                .map(|term| Self { term, is_negated })
                .collect()),
        }
    }
}

impl<Id: Clone + PartialEq> Rule<Id> {
    pub fn expand_ors(self, or: &Id) -> Result<Vec<Self>, Self> {
        if !self.has_custom_term(or) {
            return Err(self);
        }

        let terms = self.term.expand_ors(or).unwrap_or_else(|x| vec![x]);
        let predicatess = self
            .predicates
            .into_iter()
            .map(|predicate| predicate.expand_ors(or).unwrap_or_else(|x| vec![x]))
            .fold(vec![vec![]], cartesian);

        let mut rules = Vec::with_capacity(terms.len() * predicatess.len());
        for (terms, predicates) in repeat_n(terms, predicatess.len()).zip(predicatess) {
            for (predicates, term) in repeat_n(predicates, terms.len()).zip(terms) {
                rules.push(Self { term, predicates });
            }
        }

        Ok(rules)
    }
}

impl<Id: Clone + PartialEq> Term<Id> {
    pub fn expand_ors(self: &Arc<Self>, or: &Id) -> Result<Vec<Arc<Self>>, Arc<Self>> {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };

        match self.as_ref() {
            Base(proposition) => proposition.expand_ors_with(or, Base),
            Custom0(_) => Err(self.clone()),
            CustomN(AtomOrVariable::Atom(id), arguments) if id == or => match &arguments[..] {
                [argument] => argument.expand_ors(or),
                arguments => Ok(arguments
                    .iter()
                    .flat_map(|argument| argument.expand_ors(or).unwrap_or_else(|x| vec![x]))
                    .collect()),
            },
            CustomN(name, arguments) => {
                if !self.has_custom_term(or) {
                    return Err(self.clone());
                }

                Ok(arguments
                    .iter()
                    .map(|argument| argument.expand_ors(or).unwrap_or_else(|x| vec![x]))
                    .fold(vec![vec![]], cartesian)
                    .into_iter()
                    .map(|arguments| CustomN(name.clone(), arguments))
                    .map(Arc::new)
                    .collect())
            }
            Does(role, action) => action.expand_ors_with(or, |action| Does(role.clone(), action)),
            Goal(_, _) => Err(self.clone()),
            Init(proposition) => proposition.expand_ors_with(or, Init),
            Input(role, action) => action.expand_ors_with(or, |action| Input(role.clone(), action)),
            Legal(role, action) => action.expand_ors_with(or, |action| Legal(role.clone(), action)),
            Next(proposition) => proposition.expand_ors_with(or, Next),
            Role(_) => Err(self.clone()),
            Terminal => Err(self.clone()),
            True(proposition) => proposition.expand_ors_with(or, True),
        }
    }

    fn expand_ors_with<With: Fn(Arc<Self>) -> Self>(
        self: &Arc<Self>,
        or: &Id,
        with: With,
    ) -> Result<Vec<Arc<Self>>, Arc<Self>> {
        self.expand_ors(or)
            .map(|terms| terms.into_iter().map(&with).map(Arc::new).collect())
            .map_err(&with)
            .map_err(Arc::new)
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = Game::from($actual).expand_ors(&"or");
                let mut expect = Game::from($expect);

                actual.0.sort_unstable();
                expect.0.sort_unstable();

                assert_eq!(actual.as_infix().to_string(), expect.as_infix().to_string());
            }
        };
    }

    test!(simple, "a :- or(b, c)", "a :- b a :- c");
    test!(
        nested,
        "a :- or(b, or(or(c, d), e))",
        "a :- b a :- c a :- d a :- e"
    );
}
