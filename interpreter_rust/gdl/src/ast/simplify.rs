use crate::ast::{Game, Predicate, Rule, Term};
use std::collections::BTreeSet;

impl<Id: Clone + Ord> Game<Id> {
    pub fn simplify(self) -> Self {
        let mut rules = self.0;
        rules.sort_unstable();
        rules.dedup();

        let mut consts_map = vec![false; rules.len()];
        let mut consts_set = BTreeSet::new();

        loop {
            let mut any_rule_simplified = false;
            for (index, rule) in rules.iter_mut().enumerate() {
                if consts_map[index] {
                    continue;
                }

                let mut rule_simplified = false;
                rule.predicates.retain(|predicate| {
                    let is_const = predicate.can_be_const() && consts_set.contains(&predicate.term);
                    rule_simplified |= is_const;
                    !is_const
                });

                any_rule_simplified |= rule_simplified;
                if rule.is_const() {
                    any_rule_simplified = true;
                    consts_map[index] = true;
                    consts_set.insert(rule.term.clone());
                }
            }

            if !any_rule_simplified {
                break;
            }
        }

        Self(rules)
    }
}

impl<Id> Predicate<Id> {
    fn can_be_const(&self) -> bool {
        self.term.can_be_const()
    }
}

impl<Id> Rule<Id> {
    fn is_const(&self) -> bool {
        self.term.can_be_const() && self.predicates.is_empty()
    }
}

impl<Id> Term<Id> {
    fn can_be_const(&self) -> bool {
        matches!(self, Self::Custom0(_) | Self::CustomN(_, _) | Self::Role(_))
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
                let mut actual = parse($actual).simplify();
                let mut expect = parse($expect);

                actual.0.sort_unstable();
                expect.0.sort_unstable();

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
