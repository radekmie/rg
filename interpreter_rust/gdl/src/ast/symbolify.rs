use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::rc::Rc;

impl Game<String> {
    pub fn symbolify(&self) -> Self {
        Self(self.0.iter().map(|rule| rule.symbolify()).collect())
    }
}

impl Predicate<String> {
    pub fn symbolify(&self) -> Self {
        Self {
            is_negated: self.is_negated,
            term: Rc::new(self.term.symbolify()),
        }
    }
}

impl Rule<String> {
    pub fn symbolify(&self) -> Self {
        Self {
            term: Rc::new(self.term.symbolify()),
            predicates: self
                .predicates
                .iter()
                .map(|predicate| predicate.symbolify())
                .collect(),
        }
    }
}

impl Term<String> {
    pub fn symbolify(&self) -> Self {
        use Term::*;
        self.maybe_symbolify().map_or_else(
            || match self {
                Base(proposition) => Base(Rc::new(proposition.symbolify())),
                Custom(name, arguments) => Custom(
                    name.clone(),
                    arguments
                        .iter()
                        .map(|argument| Rc::new(argument.symbolify()))
                        .collect(),
                ),
                Does(role, action) => Does(role.clone(), Rc::new(action.symbolify())),
                Goal(role, utility) => Goal(role.clone(), utility.clone()),
                Init(proposition) => Init(Rc::new(proposition.symbolify())),
                Input(role, action) => Input(role.clone(), Rc::new(action.symbolify())),
                Legal(role, action) => Legal(role.clone(), Rc::new(action.symbolify())),
                Next(proposition) => Next(Rc::new(proposition.symbolify())),
                Role(role) => Role(role.clone()),
                Terminal => Terminal,
                True(proposition) => True(Rc::new(proposition.symbolify())),
            },
            |id| Custom(AtomOrVariable::Atom(id), vec![]),
        )
    }

    // TODO: We do not handle name collisions yet, so `a(b(c))` and `a(b, c)` do
    // have the same symbolified form of `a_b_c`.
    fn maybe_symbolify(&self) -> Option<String> {
        match self {
            // TODO: Those two could be simplified earlier.
            Self::Custom(AtomOrVariable::Atom(id), _) if id == "distinct" || id == "or" => None,
            Self::Custom(AtomOrVariable::Atom(id), arguments) => {
                let mut symbolified_id = id.clone();
                for argument in arguments {
                    symbolified_id.push('_');
                    symbolified_id.push_str(&argument.maybe_symbolify()?);
                }
                Some(symbolified_id)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::infix::game;
    use map_id::MapId;
    use nom::combinator::all_consuming;

    fn parse(input: &str) -> Game<String> {
        all_consuming(game)(&input)
            .unwrap()
            .1
            .map_id(&mut |id| String::from(*id))
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
