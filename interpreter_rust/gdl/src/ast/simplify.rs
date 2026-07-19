use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::collections::BTreeSet;
use std::sync::Arc;

impl<Id: Clone + Ord> Game<Id> {
    pub fn simplify(mut self) -> Self {
        self.remove_constant_predicates();
        self.remove_impossible_rules();
        self.remove_illegal_rules();
        self
    }

    fn remove_constant_predicates(&mut self) {
        self.0.sort_unstable();
        let mut rules: Vec<_> = self.0.extract_if(.., |rule| !rule.is_const()).collect();
        let mut const_rules = vec![];
        let mut const_terms: BTreeSet<_> = self.0.iter().map(|rule| rule.term.clone()).collect();

        loop {
            let mut any_rule_simplified = false;
            rules
                .extract_if(.., |rule| {
                    let removed = rule
                        .predicates
                        .extract_if(.., |predicate| {
                            predicate.can_be_const() && const_terms.contains(&predicate.term)
                        })
                        .count();

                    if removed != 0 {
                        any_rule_simplified = true;
                        if rule.is_const() {
                            const_terms.insert(rule.term.clone());
                            return true;
                        }
                    }

                    false
                })
                .for_each(|rule| const_rules.push(rule));

            if !any_rule_simplified {
                break;
            }

            // This helps with chained predicates, as having a fixed order may
            // result in a lot of additional iterations needed.
            rules.reverse();
        }

        self.0.extend(const_rules);
        self.0.extend(rules);
    }

    // Calculates all `legal` and removes rules that use unknown ones in `does`.
    fn remove_illegal_rules(&mut self) {
        let role_actions: Vec<_> = self
            .0
            .iter()
            .filter_map(|rule| match rule.term.as_ref() {
                Term::Legal(AtomOrVariable::Atom(role), action) if !action.has_variable() => {
                    Some((role.clone(), action.clone()))
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        self.0.retain(|rule| {
            rule.predicates
                .iter()
                .all(|predicate| match predicate.term.as_ref() {
                    Term::Does(AtomOrVariable::Atom(r1), a1) if !a1.has_variable() => role_actions
                        .binary_search_by(|(r2, a2)| r1.cmp(r2).then_with(|| a1.cmp(a2)).reverse())
                        .is_ok(),
                    _ => true,
                })
        });
    }

    fn remove_impossible_rules(&mut self) {
        let mut rules: Vec<_> = self.0.extract_if(.., |rule| rule.can_be_pruned()).collect();
        let possible_goals: BTreeSet<_> = self.0.iter().filter_map(Rule::as_prunable).collect();

        loop {
            let mut possible_goals = possible_goals.clone();
            possible_goals.extend(rules.iter().filter_map(Rule::as_prunable));

            let removed = rules
                .extract_if(.., |rule| {
                    rule.predicates.iter().any(|predicate| {
                        predicate.can_be_pruned() && !possible_goals.contains(&predicate.term)
                    })
                })
                .count();

            if removed == 0 {
                break;
            }
        }

        self.0.extend(rules);
    }
}

impl<Id> Predicate<Id> {
    fn can_be_const(&self) -> bool {
        self.term.can_be_const()
    }

    fn can_be_pruned(&self) -> bool {
        self.term.can_be_pruned()
    }
}

impl<Id> Rule<Id> {
    fn as_prunable(&self) -> Option<Arc<Term<Id>>> {
        self.term.can_be_pruned().then(|| self.term.clone())
    }

    fn can_be_pruned(&self) -> bool {
        self.predicates.iter().any(Predicate::can_be_pruned)
    }

    fn is_const(&self) -> bool {
        self.term.can_be_const() && self.predicates.is_empty()
    }
}

impl<Id> Term<Id> {
    fn can_be_const(&self) -> bool {
        matches!(self, Self::Custom0(_) | Self::CustomN(_, _) | Self::Role(_))
    }

    fn can_be_pruned(&self) -> bool {
        matches!(self, Self::Custom0(_) | Self::CustomN(_, _))
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = Game::from($actual).simplify();
                let mut expect = Game::from($expect);

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

    test!(
        chain,
        "
        lessThan(0, 1)
        lessThan(0, 1) :- lessThan(1, 1)
        lessThan(0, 2) :- lessThan(1, 2)
        lessThan(0, 3) :- lessThan(1, 3)
        lessThan(0, 4) :- lessThan(1, 4)
        lessThan(0, 5) :- lessThan(1, 5)
        lessThan(1, 1) :- lessThan(2, 1)
        lessThan(1, 2)
        lessThan(1, 2) :- lessThan(2, 2)
        lessThan(1, 3) :- lessThan(2, 3)
        lessThan(1, 4) :- lessThan(2, 4)
        lessThan(1, 5) :- lessThan(2, 5)
        lessThan(2, 1) :- lessThan(3, 1)
        lessThan(2, 2) :- lessThan(3, 2)
        lessThan(2, 3)
        lessThan(2, 3) :- lessThan(3, 3)
        lessThan(2, 4) :- lessThan(3, 4)
        lessThan(2, 5) :- lessThan(3, 5)
        lessThan(3, 1) :- lessThan(4, 1)
        lessThan(3, 2) :- lessThan(4, 2)
        lessThan(3, 3) :- lessThan(4, 3)
        lessThan(3, 4)
        lessThan(3, 4) :- lessThan(4, 4)
        lessThan(3, 5) :- lessThan(4, 5)
        lessThan(4, 5)
        succ(0, 1)
        succ(1, 2)
        succ(2, 3)
        succ(3, 4)
        succ(4, 5)
        ",
        "
        lessThan(0, 1)
        lessThan(0, 2)
        lessThan(0, 3)
        lessThan(0, 4)
        lessThan(0, 5)
        lessThan(1, 2)
        lessThan(1, 3)
        lessThan(1, 4)
        lessThan(1, 5)
        lessThan(2, 3)
        lessThan(2, 4)
        lessThan(2, 5)
        lessThan(3, 4)
        lessThan(3, 5)
        lessThan(4, 5)
        succ(0, 1)
        succ(1, 2)
        succ(2, 3)
        succ(3, 4)
        succ(4, 5)
        "
    );
}
