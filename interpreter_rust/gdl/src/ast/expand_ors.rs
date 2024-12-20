use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::sync::Arc;

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn expand_ors(self, or: &Id) -> Self {
        Self(
            self.0
                .into_iter()
                .flat_map(|rule| rule.expand_ors(or))
                .collect(),
        )
    }
}

impl<Id: Clone + PartialEq> Predicate<Id> {
    pub fn expand_ors(&self, or: &Id) -> Vec<Self> {
        self.term
            .expand_ors(or)
            .into_iter()
            .map(|term| Self {
                is_negated: self.is_negated,
                term: Arc::new(term),
            })
            .collect()
    }
}

impl<Id: Clone + PartialEq> Rule<Id> {
    pub fn expand_ors(self, or: &Id) -> Vec<Self> {
        self.predicates
            .into_iter()
            .map(|predicate| predicate.expand_ors(or))
            .fold(vec![vec![]], |xs, ys| {
                let mut zs = vec![];
                for x in xs {
                    for y in &ys {
                        let mut x = x.clone();
                        x.push(y.clone());
                        zs.push(x);
                    }
                }
                zs
            })
            .into_iter()
            .flat_map(move |predicates| {
                self.term.expand_ors(or).into_iter().map(move |term| Self {
                    term: Arc::new(term),
                    predicates: predicates.clone(),
                })
            })
            .collect()
    }
}

impl<Id: Clone + PartialEq> Term<Id> {
    pub fn expand_ors(&self, or: &Id) -> Vec<Self> {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };
        match self {
            Base(proposition) => proposition
                .expand_ors(or)
                .into_iter()
                .map(|proposition| Base(Arc::new(proposition)))
                .collect(),
            Custom0(role) => vec![Custom0(role.clone())],
            CustomN(AtomOrVariable::Atom(id), arguments) if id == or => arguments
                .iter()
                .flat_map(|argument| argument.expand_ors(or))
                .collect(),
            CustomN(name, arguments) => arguments
                .iter()
                .map(|argument| argument.expand_ors(or))
                .fold(vec![vec![]], |xs, ys| {
                    let mut zs = vec![];
                    for x in xs {
                        for y in &ys {
                            let mut x = x.clone();
                            x.push(Arc::new(y.clone()));
                            zs.push(x);
                        }
                    }
                    zs
                })
                .into_iter()
                .map(move |terms| CustomN(name.clone(), terms))
                .collect(),
            Does(role, action) => action
                .expand_ors(or)
                .into_iter()
                .map(|action| Does(role.clone(), Arc::new(action)))
                .collect(),
            Goal(role, utility) => vec![Goal(role.clone(), utility.clone())],
            Init(proposition) => proposition
                .expand_ors(or)
                .into_iter()
                .map(|proposition| Init(Arc::new(proposition)))
                .collect(),
            Input(role, action) => action
                .expand_ors(or)
                .into_iter()
                .map(|action| Input(role.clone(), Arc::new(action)))
                .collect(),
            Legal(role, action) => action
                .expand_ors(or)
                .into_iter()
                .map(|action| Legal(role.clone(), Arc::new(action)))
                .collect(),
            Next(proposition) => proposition
                .expand_ors(or)
                .into_iter()
                .map(|proposition| Next(Arc::new(proposition)))
                .collect(),
            Role(role) => vec![Role(role.clone())],
            Terminal => vec![Terminal],
            True(proposition) => proposition
                .expand_ors(or)
                .into_iter()
                .map(|proposition| True(Arc::new(proposition)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::infix::game;
    use nom::combinator::all_consuming;

    fn parse(input: &str) -> Game<&str> {
        all_consuming(game)(input).unwrap().1
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual).expand_ors(&"or");
                let mut expect = parse($expect);

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
