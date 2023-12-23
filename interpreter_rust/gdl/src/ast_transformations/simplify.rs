use crate::ast::{Game, Term};

impl<Symbol: Clone + PartialEq> Game<Symbol> {
    pub fn simplify(&self) -> Self {
        let mut rules = self.0.clone();

        loop {
            let mut any_simplification_happened = false;
            for i in 0..rules.len() {
                let (xs, ys) = rules.split_at_mut(i);
                let (rule, ys) = ys.split_first_mut().unwrap();
                rule.predicates.retain(|(_, predicate)| {
                    let is_constant = matches!(**predicate, Term::Custom(_, _) | Term::Role(_))
                        && xs
                            .iter()
                            .chain(ys.iter())
                            .any(|rule| rule.predicates.is_empty() && rule.term == *predicate);
                    any_simplification_happened = any_simplification_happened || is_constant;
                    !is_constant
                });
            }

            if !any_simplification_happened {
                break;
            }
        }

        Self(rules)
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::infix::game;
    use nom::combinator::all_consuming;

    fn parse(input: &str) -> Game<&str> {
        all_consuming(game)(&input).unwrap().1
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual).simplify();
                let mut expect = parse($expect);

                // TODO: `&str` is not `Ord`.
                actual.0.sort_unstable_by_key(|x| format!("{x:?}"));
                expect.0.sort_unstable_by_key(|x| format!("{x:?}"));

                assert_eq!(actual.as_infix().to_string(), expect.as_infix().to_string());
            }
        };
    }

    test!(
        one_variable_one_precondition,
        "a(1) b(1) :- a(1)",
        "a(1) b(1)"
    );

    test!(
        one_variable_two_preconditions,
        "a(1) b(1) :- a(1) & a(1)",
        "a(1) b(1)"
    );

    test!(
        two_variables_one_precondition,
        "a(1, 2) b(1, 2) :- a(1, 2)",
        "a(1, 2) b(1, 2)"
    );

    test!(
        two_variables_two_preconditions,
        "a(1, 2) b(1, 2) :- a(1, 2) & a(1, 2)",
        "a(1, 2) b(1, 2)"
    );

    test!(chain_1, "a :- b b :- c c :- d d :- e e", "a b c d e");
    test!(chain_2, "e d :- e c :- d b :- c a :- b", "a b c d e");
}
