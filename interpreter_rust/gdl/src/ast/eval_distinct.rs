use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};

impl Game<&str> {
    pub fn eval_distinct(&self) -> Self {
        Self(self.0.iter().filter_map(Rule::eval_distinct).collect())
    }
}

impl Predicate<&str> {
    pub fn eval_distinct(&self) -> Option<bool> {
        self.term
            .eval_distinct()
            .map(|is_distinct| is_distinct != self.is_negated)
    }
}

impl Rule<&str> {
    pub fn eval_distinct(&self) -> Option<Self> {
        self.predicates
            .iter()
            .try_fold(vec![], |mut predicates, predicate| {
                match predicate.eval_distinct() {
                    Some(false) => None,
                    Some(true) => Some(predicates),
                    None => {
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

impl Term<&str> {
    pub fn eval_distinct(&self) -> Option<bool> {
        match self {
            Self::Custom(AtomOrVariable::Atom(id), arguments) if *id == "distinct" => {
                assert!(arguments.len() == 2);
                Some(arguments[0] != arguments[1])
            }
            _ => None,
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
                let mut actual = parse($actual).eval_distinct();
                let mut expect = parse($expect);

                actual.0.sort_unstable();
                expect.0.sort_unstable();

                assert_eq!(actual.as_infix().to_string(), expect.as_infix().to_string());
            }
        };
    }

    test!(negative, "a :- distinct(1, 1)", "");
    test!(positive, "a :- distinct(1, 2)", "a");
}
