use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::rc::Rc;
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn expand_ors(&self) -> Self {
        Self(self.0.iter().flat_map(|rule| rule.expand_ors()).collect())
    }
}

impl Predicate<Arc<str>> {
    pub fn expand_ors(&self) -> Vec<Self> {
        self.term
            .expand_ors()
            .into_iter()
            .map(|term| Self {
                is_negated: self.is_negated,
                term: Rc::new(term),
            })
            .collect()
    }
}

impl Rule<Arc<str>> {
    pub fn expand_ors(&self) -> Vec<Self> {
        self.predicates
            .iter()
            .map(|predicate| predicate.expand_ors())
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
                self.term.expand_ors().into_iter().map(move |term| Self {
                    term: Rc::new(term),
                    predicates: predicates.clone(),
                })
            })
            .collect()
    }
}

impl Term<Arc<str>> {
    pub fn expand_ors(&self) -> Vec<Self> {
        use Term::*;
        match self {
            Base(proposition) => proposition
                .expand_ors()
                .into_iter()
                .map(|proposition| Base(Rc::new(proposition)))
                .collect(),
            Custom(AtomOrVariable::Atom(id), arguments) if &**id == "or" => arguments
                .iter()
                .map(|argument| (**argument).clone())
                .collect(),
            Custom(name, arguments) => arguments
                .iter()
                .map(|argument| argument.expand_ors())
                .fold(vec![vec![]], |xs, ys| {
                    let mut zs = vec![];
                    for x in xs {
                        for y in &ys {
                            let mut x = x.clone();
                            x.push(Rc::new(y.clone()));
                            zs.push(x);
                        }
                    }
                    zs
                })
                .into_iter()
                .map(move |terms| Custom(name.clone(), terms))
                .collect(),
            Does(role, action) => action
                .expand_ors()
                .into_iter()
                .map(|action| Does(role.clone(), Rc::new(action)))
                .collect(),
            Goal(role, utility) => vec![Goal(role.clone(), utility.clone())],
            Init(proposition) => proposition
                .expand_ors()
                .into_iter()
                .map(|proposition| Init(Rc::new(proposition)))
                .collect(),
            Input(role, action) => action
                .expand_ors()
                .into_iter()
                .map(|action| Input(role.clone(), Rc::new(action)))
                .collect(),
            Legal(role, action) => action
                .expand_ors()
                .into_iter()
                .map(|action| Legal(role.clone(), Rc::new(action)))
                .collect(),
            Next(proposition) => proposition
                .expand_ors()
                .into_iter()
                .map(|proposition| Next(Rc::new(proposition)))
                .collect(),
            Role(role) => vec![Role(role.clone())],
            Terminal => vec![Terminal],
            True(proposition) => proposition
                .expand_ors()
                .into_iter()
                .map(|proposition| True(Rc::new(proposition)))
                .collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::infix::game;
    use map_id::MapId;
    use nom::combinator::all_consuming;
    use std::sync::Arc;

    fn parse(input: &str) -> Game<Arc<str>> {
        all_consuming(game)(&input)
            .unwrap()
            .1
            .map_id(&mut |id| Arc::from(*id))
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual).symbolify();
                let mut expect = parse($expect);

                // TODO: `&str` is not `Ord`.
                actual.0.sort_unstable_by_key(|x| format!("{x:?}"));
                expect.0.sort_unstable_by_key(|x| format!("{x:?}"));

                assert_eq!(actual.as_infix().to_string(), expect.as_infix().to_string());
            }
        };
    }

    test!(atom, "a", "a");
    test!(unary, "a(x)", "a_x");
    test!(binary, "a(x, y)", "a_x_y");
    test!(nested_1, "a(b(x), y)", "a_b_x_y");
    test!(nested_2, "a(x, c(y))", "a_x_c_y");
    test!(nested_3, "a(b(x), c(y))", "a_b_x_c_y");
    test!(collision_1, "a(b(c))", "a_b_c");
    test!(collision_2, "a(b, c)", "a_b_c");
}
