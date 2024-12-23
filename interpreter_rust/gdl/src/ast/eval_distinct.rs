use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::sync::Arc;

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn eval_distinct(self, distinct: &Id, or: &Id) -> Self {
        Self(
            self.0
                .into_iter()
                .filter_map(|rule| rule.eval_distinct(distinct, or))
                .collect(),
        )
    }
}

impl<Id: Clone + PartialEq> Predicate<Id> {
    pub fn eval_distinct(&mut self, distinct: &Id, or: &Id) -> Option<bool> {
        Arc::make_mut(&mut self.term)
            .eval_distinct(distinct, or)
            .map(|is_distinct| is_distinct != self.is_negated)
    }
}

impl<Id: Clone + PartialEq> Rule<Id> {
    pub fn eval_distinct(self, distinct: &Id, or: &Id) -> Option<Self> {
        self.predicates
            .into_iter()
            .try_fold(vec![], |mut predicates, mut predicate| {
                match predicate.eval_distinct(distinct, or) {
                    Some(false) => None,
                    Some(true) => Some(predicates),
                    None => {
                        predicates.push(predicate);
                        Some(predicates)
                    }
                }
            })
            .map(|predicates| Self {
                term: self.term,
                predicates,
            })
    }
}

impl<Id: Clone + PartialEq> Term<Id> {
    pub fn eval_distinct(&mut self, distinct: &Id, or: &Id) -> Option<bool> {
        if let Self::CustomN(AtomOrVariable::Atom(id), arguments) = self {
            if id == distinct {
                if let [lhs, rhs] = &arguments[..] {
                    if !lhs.has_variable() && !rhs.has_variable() {
                        return Some(lhs != rhs);
                    }
                }
            } else if id == or {
                let arguments = arguments
                    .iter_mut()
                    .try_fold(vec![], |mut arguments, argument| {
                        match Arc::make_mut(argument).eval_distinct(distinct, or) {
                            Some(false) => Some(arguments),
                            Some(true) => None,
                            None => {
                                arguments.push(argument);
                                Some(arguments)
                            }
                        }
                    });

                match arguments {
                    None => return Some(true),
                    Some(mut arguments) => match arguments.len() {
                        0 => return Some(false),
                        1 => *self = Arc::unwrap_or_clone(arguments.pop().unwrap().clone()),
                        _ => {}
                    },
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::game;
    use nom::combinator::all_consuming;

    fn parse(input: &str) -> Game<&str> {
        all_consuming(game)(input).unwrap().1
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual).eval_distinct(&"distinct", &"or");
                let mut expect = parse($expect);

                actual.0.sort_unstable();
                expect.0.sort_unstable();

                assert_eq!(actual.as_infix().to_string(), expect.as_infix().to_string());
            }
        };
    }

    test!(negative, "a :- distinct(1, 1)", "");
    test!(positive, "a :- distinct(1, 2)", "a");

    test!(unknown_lhs, "a :- distinct(X, 2)", "a :- distinct(X, 2)");
    test!(unknown_rhs, "a :- distinct(1, Y)", "a :- distinct(1, Y)");
    test!(unknown_both, "a :- distinct(X, Y)", "a :- distinct(X, Y)");

    test!(
        or_negative_lhs,
        "a :- or(distinct(1, 1), distinct(X, Y))",
        "a :- distinct(X, Y)"
    );
    test!(
        or_negative_rhs,
        "a :- or(distinct(X, Y), distinct(1, 1))",
        "a :- distinct(X, Y)"
    );
    test!(
        or_positive_lhs,
        "a :- or(distinct(1, 2), distinct(X, Y))",
        "a"
    );
    test!(
        or_positive_rhs,
        "a :- or(distinct(X, Y), distinct(1, 2))",
        "a"
    );
}
