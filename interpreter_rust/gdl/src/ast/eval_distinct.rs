use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn eval_distinct(&self) -> Self {
        Self(
            self.0
                .iter()
                .flat_map(|rule| rule.eval_distinct())
                .collect(),
        )
    }
}

impl Predicate<Arc<str>> {
    pub fn eval_distinct(&self) -> Option<bool> {
        self.term
            .eval_distinct()
            .map(|is_distinct| is_distinct != self.is_negated)
    }
}

impl Rule<Arc<str>> {
    pub fn eval_distinct(&self) -> Option<Self> {
        self.predicates
            .iter()
            .fold(Some(vec![]), |predicates, predicate| {
                match (predicates, predicate.eval_distinct()) {
                    (None, _) | (_, Some(false)) => None,
                    (Some(predicates), Some(true)) => Some(predicates),
                    (Some(mut predicates), None) => {
                        predicates.push(predicate.clone());
                        Some(predicates)
                    }
                }
            })
            .map(|predicates| Self {
                term: self.term.clone(),
                predicates,
            })
    }
}

impl Term<Arc<str>> {
    pub fn eval_distinct(&self) -> Option<bool> {
        match self {
            Self::Custom(AtomOrVariable::Atom(id), arguments) if &**id == "distinct" => {
                assert!(arguments.len() == 2);
                Some(*arguments[0] != *arguments[1])
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
