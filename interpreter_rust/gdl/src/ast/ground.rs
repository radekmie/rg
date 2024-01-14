use super::unify::Unification;
use crate::ast::{AtomOrVariable, Game, Rule, Term};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl<Id: Clone + Ord> Game<Id> {
    pub fn ground(self) -> Self {
        let mut rules = self.0;
        rules.sort_unstable();
        for rule in &mut rules {
            rule.predicates.sort_unstable();
        }

        // Cached subterms partitioned into ones with and without variables.
        let mut subterms: Vec<_> = rules.iter().map(get_subterms).collect();

        // Rules grounded in this iteration.
        let mut rules_to_add = vec![];

        // Rules to be reviewed in this iteration.
        let mut rules_to_review: Vec<_> = (0..rules.len()).collect();

        // Static terms used to prune impossible substitutions.
        let static_terms = get_static_terms(&rules);

        loop {
            for i in 0..rules.len() {
                if !subterms[i].0.is_empty() {
                    continue;
                }

                for j in 0..rules.len() {
                    if i == j
                        || subterms[j].0.is_empty()
                        || (rules_to_review.binary_search(&i).is_err()
                            && rules_to_review.binary_search(&j).is_err())
                    {
                        continue;
                    }

                    if let Some(unification) = any_unification(&subterms[j].0, &subterms[i].1) {
                        let mut rule = rules[j].substitute(&unification);
                        rule.predicates.sort_unstable();
                        rule.predicates.dedup();

                        let has_failed_static_term = rule.predicates.iter().any(|predicate| {
                            if !predicate.has_variable() {
                                if let Term::Custom(AtomOrVariable::Atom(id), _) =
                                    predicate.term.as_ref()
                                {
                                    if let Some(terms) = static_terms.get(id) {
                                        if !terms.contains(&predicate.term) || predicate.is_negated
                                        {
                                            return true;
                                        }
                                    }
                                }
                            }

                            false
                        });

                        if !has_failed_static_term && rules.binary_search(&rule).is_err() {
                            if let Err(index) = rules_to_add.binary_search(&rule) {
                                rules_to_add.insert(index, rule);
                            }
                        }
                    }
                }
            }

            // No new rules means everything got grounded.
            if rules_to_add.is_empty() {
                break;
            }

            rules_to_review.clear();
            rules.reserve(rules_to_add.len());
            subterms.reserve(rules_to_add.len());

            for rule in rules_to_add.drain(..) {
                let index = rules.binary_search(&rule).unwrap_err();
                rules_to_review.push(index);
                subterms.insert(index, get_subterms(&rule));
                rules.insert(index, rule);
            }
        }

        rules.retain(|rule| !rule.has_variable());

        Self(rules)
    }
}

fn any_unification<'a, Id: Clone + Ord>(
    xs: &'a [Term<Id>],
    ys: &'a [Term<Id>],
) -> Option<Unification<'a, Id>> {
    for x in xs {
        for y in ys {
            let unification = x.unify(y);
            if !unification.is_empty() {
                return Some(unification);
            }
        }
    }

    None
}

fn get_static_terms<Id: Clone + Ord>(rules: &[Rule<Id>]) -> BTreeMap<Id, BTreeSet<Arc<Term<Id>>>> {
    rules
        .iter()
        .filter_map(|rule| {
            if rule.predicates.is_empty() {
                if let Term::Custom(AtomOrVariable::Atom(x), _) = rule.term.as_ref() {
                    return Some(x);
                }
            }

            None
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|x| {
            rules.iter().all(|rule| {
                match rule.term.as_ref() {
                    Term::Custom(AtomOrVariable::Atom(y), _) if *x == y => rule.predicates.is_empty(),
                    _ => true,
                }
            })
        })
        .map(|x| (
            x.clone(),
            rules
                .iter()
                .filter(move |rule| matches!(rule.term.as_ref(), Term::Custom(AtomOrVariable::Atom(y), _) if x == y))
                .map(|rule| rule.term.clone())
                .collect()
        ))
        .collect()
}

fn get_subterms<Id: Clone + PartialEq>(rule: &Rule<Id>) -> (Vec<Term<Id>>, Vec<Term<Id>>) {
    rule.subterms().cloned().partition(Term::has_variable)
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
                let mut actual = parse($actual).ground();
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
        "a(1) a(2) b(X) :- a(X)",
        "a(1) a(2) b(1) :- a(1) b(2) :- a(2)"
    );

    test!(
        one_variable_two_preconditions,
        "a(1) a(2) b(X) :- a(X) & a(X)",
        "a(1) a(2) b(1) :- a(1) b(2) :- a(2)"
    );

    test!(
        two_variables_one_precondition,
        "a(1, 2) a(3, 4) b(X, Y) :- a(X, Y)",
        "a(1, 2) a(3, 4) b(1, 2) :- a(1, 2) b(3, 4) :- a(3, 4)"
    );

    test!(
        two_variables_two_preconditions,
        "a(1, 2) a(3, 4) b(X, Y) :- a(X, Y) & a(X, Y)",
        "a(1, 2) a(3, 4) b(1, 2) :- a(1, 2) b(3, 4) :- a(3, 4)"
    );

    test!(
        two_variables_partial_unification,
        "a(1, 2) a(3, 4) b(1, Y) :- a(1, Y)",
        "a(1, 2) a(3, 4) b(1, 2) :- a(1, 2)"
    );

    test!(
        two_variables_cross_product_1,
        "a(1) a(2) b(X, Y) :- a(X) & a(Y)",
        "a(1) a(2) b(1, 1) :- a(1) b(1, 2) :- a(1) & a(2) b(2, 1) :- a(1) & a(2) b(2, 2) :- a(2)"
    );

    test!(
        two_variables_cross_product_2,
        "a(1) a(2) b(X) :- a(X) & a(Y)",
        "a(1) a(2) b(1) :- a(1) b(1) :- a(1) & a(2) b(2) :- a(1) & a(2) b(2) :- a(2)"
    );

    test!(
        nested_simple,
        "a(1) b(X) :- c(d(1, X)) e(d(X, Y)) :- a(X) & f(Y) f(2) :- a(1)",
        "a(1) b(2) :- c(d(1, 2)) e(d(1, 2)) :- a(1) & f(2) f(2) :- a(1)"
    );

    test!(
        nested_complex,
        "
        index(1)
        index(2)

        base(cell(X, Y, b)) :- index(X) & index(Y)
        base(cell(X, Y, x)) :- index(X) & index(Y)
        base(cell(X, Y, o)) :- index(X) & index(Y)

        diagonal(X) :- true(cell(1, 1, X)) & true(cell(2, 2, X))
        diagonal(X) :- true(cell(1, 2, X)) & true(cell(2, 1, X))

        column(N, X) :- true(cell(1, N, X)) & true(cell(2, N, X))

        row(M, X) :- true(cell(M, 1, X)) & true(cell(M, 2, X))

        line(X) :- diagonal(X)
        line(X) :- column(M, X)
        line(X) :- row(M, X)
        ",
        "
        index(1)
        index(2)

        base(cell(1, 1, b)) :- index(1)
        base(cell(1, 2, b)) :- index(1) & index(2)
        base(cell(2, 1, b)) :- index(1) & index(2)
        base(cell(2, 2, b)) :- index(2)

        base(cell(1, 1, x)) :- index(1)
        base(cell(1, 2, x)) :- index(1) & index(2)
        base(cell(2, 1, x)) :- index(1) & index(2)
        base(cell(2, 2, x)) :- index(2)

        base(cell(1, 1, o)) :- index(1)
        base(cell(1, 2, o)) :- index(1) & index(2)
        base(cell(2, 1, o)) :- index(1) & index(2)
        base(cell(2, 2, o)) :- index(2)

        diagonal(b) :- true(cell(1, 1, b)) & true(cell(2, 2, b))
        diagonal(x) :- true(cell(1, 1, x)) & true(cell(2, 2, x))
        diagonal(o) :- true(cell(1, 1, o)) & true(cell(2, 2, o))
        diagonal(b) :- true(cell(1, 2, b)) & true(cell(2, 1, b))
        diagonal(x) :- true(cell(1, 2, x)) & true(cell(2, 1, x))
        diagonal(o) :- true(cell(1, 2, o)) & true(cell(2, 1, o))

        column(1, b) :- true(cell(1, 1, b)) & true(cell(2, 1, b))
        column(2, b) :- true(cell(1, 2, b)) & true(cell(2, 2, b))
        column(1, x) :- true(cell(1, 1, x)) & true(cell(2, 1, x))
        column(2, x) :- true(cell(1, 2, x)) & true(cell(2, 2, x))
        column(1, o) :- true(cell(1, 1, o)) & true(cell(2, 1, o))
        column(2, o) :- true(cell(1, 2, o)) & true(cell(2, 2, o))

        row(1, b) :- true(cell(1, 1, b)) & true(cell(1, 2, b))
        row(2, b) :- true(cell(2, 1, b)) & true(cell(2, 2, b))
        row(1, x) :- true(cell(1, 1, x)) & true(cell(1, 2, x))
        row(2, x) :- true(cell(2, 1, x)) & true(cell(2, 2, x))
        row(1, o) :- true(cell(1, 1, o)) & true(cell(1, 2, o))
        row(2, o) :- true(cell(2, 1, o)) & true(cell(2, 2, o))

        line(b) :- diagonal(b)
        line(x) :- diagonal(x)
        line(o) :- diagonal(o)
        line(b) :- column(1, b)
        line(b) :- column(2, b)
        line(x) :- column(1, x)
        line(x) :- column(2, x)
        line(o) :- column(1, o)
        line(o) :- column(2, o)
        line(b) :- row(1, b)
        line(b) :- row(2, b)
        line(x) :- row(1, x)
        line(x) :- row(2, x)
        line(o) :- row(1, o)
        line(o) :- row(2, o)
        "
    );

    test!(
        tictactoe,
        "
        role(xplayer)
        role(oplayer)
        index(1)
        index(2)
        index(3)
        base(cell(X, Y, b)) :- index(X) & index(Y)
        base(cell(X, Y, x)) :- index(X) & index(Y)
        base(cell(X, Y, o)) :- index(X) & index(Y)
        base(control(P)) :- role(P)
        input(P, mark(X, Y)) :- index(X) & index(Y) & role(P)
        input(P, noop) :- role(P)
        init(cell(1, 1, b))
        init(cell(1, 2, b))
        init(cell(1, 3, b))
        init(cell(2, 1, b))
        init(cell(2, 2, b))
        init(cell(2, 3, b))
        init(cell(3, 1, b))
        init(cell(3, 2, b))
        init(cell(3, 3, b))
        init(control(xplayer))
        next(cell(M, N, x)) :- does(xplayer, mark(M, N)) & true(cell(M, N, b))
        next(cell(M, N, o)) :- does(oplayer, mark(M, N)) & true(cell(M, N, b))
        next(cell(M, N, W)) :- true(cell(M, N, W)) & distinct(W, b)
        next(cell(M, N, b)) :- does(W, mark(J, K)) & true(cell(M, N, b)) & or(distinct(M, J), distinct(N, K))
        next(control(xplayer)) :- true(control(oplayer))
        next(control(oplayer)) :- true(control(xplayer))
        row(M, X) :- true(cell(M, 1, X)) & true(cell(M, 2, X)) & true(cell(M, 3, X))
        column(N, X) :- true(cell(1, N, X)) & true(cell(2, N, X)) & true(cell(3, N, X))
        diagonal(X) :- true(cell(1, 1, X)) & true(cell(2, 2, X)) & true(cell(3, 3, X))
        diagonal(X) :- true(cell(1, 3, X)) & true(cell(2, 2, X)) & true(cell(3, 1, X))
        line(X) :- row(M, X)
        line(X) :- column(M, X)
        line(X) :- diagonal(X)
        open :- true(cell(M, N, b))
        legal(W, mark(X, Y)) :- true(cell(X, Y, b)) & true(control(W))
        legal(xplayer, noop) :- true(control(oplayer))
        legal(oplayer, noop) :- true(control(xplayer))
        goal(xplayer, 100) :- line(x)
        goal(xplayer, 50) :- ~line(x) & not(line(o)) & ~open
        goal(xplayer, 0) :- line(o)
        goal(oplayer, 100) :- line(o)
        goal(oplayer, 50) :- ~line(x) & not(line(o)) & ~open
        goal(oplayer, 0) :- line(x)
        terminal :- line(x)
        terminal :- line(o)
        terminal :- ~open
        ",
        "
        base(cell(1, 1, b)) :- index(1)
        base(cell(1, 1, o)) :- index(1)
        base(cell(1, 1, x)) :- index(1)
        base(cell(1, 2, b)) :- index(1) & index(2)
        base(cell(1, 2, o)) :- index(1) & index(2)
        base(cell(1, 2, x)) :- index(1) & index(2)
        base(cell(1, 3, b)) :- index(1) & index(3)
        base(cell(1, 3, o)) :- index(1) & index(3)
        base(cell(1, 3, x)) :- index(1) & index(3)
        base(cell(2, 1, b)) :- index(1) & index(2)
        base(cell(2, 1, o)) :- index(1) & index(2)
        base(cell(2, 1, x)) :- index(1) & index(2)
        base(cell(2, 2, b)) :- index(2)
        base(cell(2, 2, o)) :- index(2)
        base(cell(2, 2, x)) :- index(2)
        base(cell(2, 3, b)) :- index(2) & index(3)
        base(cell(2, 3, o)) :- index(2) & index(3)
        base(cell(2, 3, x)) :- index(2) & index(3)
        base(cell(3, 1, b)) :- index(1) & index(3)
        base(cell(3, 1, o)) :- index(1) & index(3)
        base(cell(3, 1, x)) :- index(1) & index(3)
        base(cell(3, 2, b)) :- index(2) & index(3)
        base(cell(3, 2, o)) :- index(2) & index(3)
        base(cell(3, 2, x)) :- index(2) & index(3)
        base(cell(3, 3, b)) :- index(3)
        base(cell(3, 3, o)) :- index(3)
        base(cell(3, 3, x)) :- index(3)
        base(control(oplayer)) :- role(oplayer)
        base(control(xplayer)) :- role(xplayer)
        column(1, b) :- true(cell(1, 1, b)) & true(cell(2, 1, b)) & true(cell(3, 1, b))
        column(1, o) :- true(cell(1, 1, o)) & true(cell(2, 1, o)) & true(cell(3, 1, o))
        column(1, x) :- true(cell(1, 1, x)) & true(cell(2, 1, x)) & true(cell(3, 1, x))
        column(2, b) :- true(cell(1, 2, b)) & true(cell(2, 2, b)) & true(cell(3, 2, b))
        column(2, o) :- true(cell(1, 2, o)) & true(cell(2, 2, o)) & true(cell(3, 2, o))
        column(2, x) :- true(cell(1, 2, x)) & true(cell(2, 2, x)) & true(cell(3, 2, x))
        column(3, b) :- true(cell(1, 3, b)) & true(cell(2, 3, b)) & true(cell(3, 3, b))
        column(3, o) :- true(cell(1, 3, o)) & true(cell(2, 3, o)) & true(cell(3, 3, o))
        column(3, x) :- true(cell(1, 3, x)) & true(cell(2, 3, x)) & true(cell(3, 3, x))
        diagonal(b) :- true(cell(1, 1, b)) & true(cell(2, 2, b)) & true(cell(3, 3, b))
        diagonal(b) :- true(cell(1, 3, b)) & true(cell(2, 2, b)) & true(cell(3, 1, b))
        diagonal(o) :- true(cell(1, 1, o)) & true(cell(2, 2, o)) & true(cell(3, 3, o))
        diagonal(o) :- true(cell(1, 3, o)) & true(cell(2, 2, o)) & true(cell(3, 1, o))
        diagonal(x) :- true(cell(1, 1, x)) & true(cell(2, 2, x)) & true(cell(3, 3, x))
        diagonal(x) :- true(cell(1, 3, x)) & true(cell(2, 2, x)) & true(cell(3, 1, x))
        index(1)
        index(2)
        index(3)
        line(b) :- column(1, b)
        line(b) :- column(2, b)
        line(b) :- column(3, b)
        line(b) :- diagonal(b)
        line(b) :- row(1, b)
        line(b) :- row(2, b)
        line(b) :- row(3, b)
        line(o) :- column(1, o)
        line(o) :- column(2, o)
        line(o) :- column(3, o)
        line(o) :- diagonal(o)
        line(o) :- row(1, o)
        line(o) :- row(2, o)
        line(o) :- row(3, o)
        line(x) :- column(1, x)
        line(x) :- column(2, x)
        line(x) :- column(3, x)
        line(x) :- diagonal(x)
        line(x) :- row(1, x)
        line(x) :- row(2, x)
        line(x) :- row(3, x)
        open :- true(cell(1, 1, b))
        open :- true(cell(1, 2, b))
        open :- true(cell(1, 3, b))
        open :- true(cell(2, 1, b))
        open :- true(cell(2, 2, b))
        open :- true(cell(2, 3, b))
        open :- true(cell(3, 1, b))
        open :- true(cell(3, 2, b))
        open :- true(cell(3, 3, b))
        row(1, b) :- true(cell(1, 1, b)) & true(cell(1, 2, b)) & true(cell(1, 3, b))
        row(1, o) :- true(cell(1, 1, o)) & true(cell(1, 2, o)) & true(cell(1, 3, o))
        row(1, x) :- true(cell(1, 1, x)) & true(cell(1, 2, x)) & true(cell(1, 3, x))
        row(2, b) :- true(cell(2, 1, b)) & true(cell(2, 2, b)) & true(cell(2, 3, b))
        row(2, o) :- true(cell(2, 1, o)) & true(cell(2, 2, o)) & true(cell(2, 3, o))
        row(2, x) :- true(cell(2, 1, x)) & true(cell(2, 2, x)) & true(cell(2, 3, x))
        row(3, b) :- true(cell(3, 1, b)) & true(cell(3, 2, b)) & true(cell(3, 3, b))
        row(3, o) :- true(cell(3, 1, o)) & true(cell(3, 2, o)) & true(cell(3, 3, o))
        row(3, x) :- true(cell(3, 1, x)) & true(cell(3, 2, x)) & true(cell(3, 3, x))
        goal(oplayer, 0) :- line(x)
        goal(oplayer, 100) :- line(o)
        goal(oplayer, 50) :- ~line(x) & not(line(o)) & ~open
        goal(xplayer, 0) :- line(o)
        goal(xplayer, 100) :- line(x)
        goal(xplayer, 50) :- ~line(x) & not(line(o)) & ~open
        init(cell(1, 1, b))
        init(cell(1, 2, b))
        init(cell(1, 3, b))
        init(cell(2, 1, b))
        init(cell(2, 2, b))
        init(cell(2, 3, b))
        init(cell(3, 1, b))
        init(cell(3, 2, b))
        init(cell(3, 3, b))
        init(control(xplayer))
        input(oplayer, mark(1, 1)) :- index(1) & role(oplayer)
        input(oplayer, mark(1, 2)) :- index(1) & index(2) & role(oplayer)
        input(oplayer, mark(1, 3)) :- index(1) & index(3) & role(oplayer)
        input(oplayer, mark(2, 1)) :- index(1) & index(2) & role(oplayer)
        input(oplayer, mark(2, 2)) :- index(2) & role(oplayer)
        input(oplayer, mark(2, 3)) :- index(2) & index(3) & role(oplayer)
        input(oplayer, mark(3, 1)) :- index(1) & index(3) & role(oplayer)
        input(oplayer, mark(3, 2)) :- index(2) & index(3) & role(oplayer)
        input(oplayer, mark(3, 3)) :- index(3) & role(oplayer)
        input(oplayer, noop) :- role(oplayer)
        input(xplayer, mark(1, 1)) :- index(1) & role(xplayer)
        input(xplayer, mark(1, 2)) :- index(1) & index(2) & role(xplayer)
        input(xplayer, mark(1, 3)) :- index(1) & index(3) & role(xplayer)
        input(xplayer, mark(2, 1)) :- index(1) & index(2) & role(xplayer)
        input(xplayer, mark(2, 2)) :- index(2) & role(xplayer)
        input(xplayer, mark(2, 3)) :- index(2) & index(3) & role(xplayer)
        input(xplayer, mark(3, 1)) :- index(1) & index(3) & role(xplayer)
        input(xplayer, mark(3, 2)) :- index(2) & index(3) & role(xplayer)
        input(xplayer, mark(3, 3)) :- index(3) & role(xplayer)
        input(xplayer, noop) :- role(xplayer)
        legal(oplayer, mark(1, 1)) :- true(cell(1, 1, b)) & true(control(oplayer))
        legal(oplayer, mark(1, 2)) :- true(cell(1, 2, b)) & true(control(oplayer))
        legal(oplayer, mark(1, 3)) :- true(cell(1, 3, b)) & true(control(oplayer))
        legal(oplayer, mark(2, 1)) :- true(cell(2, 1, b)) & true(control(oplayer))
        legal(oplayer, mark(2, 2)) :- true(cell(2, 2, b)) & true(control(oplayer))
        legal(oplayer, mark(2, 3)) :- true(cell(2, 3, b)) & true(control(oplayer))
        legal(oplayer, mark(3, 1)) :- true(cell(3, 1, b)) & true(control(oplayer))
        legal(oplayer, mark(3, 2)) :- true(cell(3, 2, b)) & true(control(oplayer))
        legal(oplayer, mark(3, 3)) :- true(cell(3, 3, b)) & true(control(oplayer))
        legal(oplayer, noop) :- true(control(xplayer))
        legal(xplayer, mark(1, 1)) :- true(cell(1, 1, b)) & true(control(xplayer))
        legal(xplayer, mark(1, 2)) :- true(cell(1, 2, b)) & true(control(xplayer))
        legal(xplayer, mark(1, 3)) :- true(cell(1, 3, b)) & true(control(xplayer))
        legal(xplayer, mark(2, 1)) :- true(cell(2, 1, b)) & true(control(xplayer))
        legal(xplayer, mark(2, 2)) :- true(cell(2, 2, b)) & true(control(xplayer))
        legal(xplayer, mark(2, 3)) :- true(cell(2, 3, b)) & true(control(xplayer))
        legal(xplayer, mark(3, 1)) :- true(cell(3, 1, b)) & true(control(xplayer))
        legal(xplayer, mark(3, 2)) :- true(cell(3, 2, b)) & true(control(xplayer))
        legal(xplayer, mark(3, 3)) :- true(cell(3, 3, b)) & true(control(xplayer))
        legal(xplayer, noop) :- true(control(oplayer))
        next(cell(1, 1, b)) :- distinct(b, b) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 1), distinct(1, 1)) & does(oplayer, mark(1, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 1), distinct(1, 1)) & does(xplayer, mark(1, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 1), distinct(1, 2)) & does(oplayer, mark(1, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 1), distinct(1, 2)) & does(xplayer, mark(1, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 1), distinct(1, 3)) & does(oplayer, mark(1, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 1), distinct(1, 3)) & does(xplayer, mark(1, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 2), distinct(1, 1)) & does(oplayer, mark(2, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 2), distinct(1, 1)) & does(xplayer, mark(2, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 2), distinct(1, 2)) & does(oplayer, mark(2, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 2), distinct(1, 2)) & does(xplayer, mark(2, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 2), distinct(1, 3)) & does(oplayer, mark(2, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 2), distinct(1, 3)) & does(xplayer, mark(2, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 3), distinct(1, 1)) & does(oplayer, mark(3, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 3), distinct(1, 1)) & does(xplayer, mark(3, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 3), distinct(1, 2)) & does(oplayer, mark(3, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 3), distinct(1, 2)) & does(xplayer, mark(3, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 3), distinct(1, 3)) & does(oplayer, mark(3, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- or(distinct(1, 3), distinct(1, 3)) & does(xplayer, mark(3, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, o)) :- distinct(o, b) & true(cell(1, 1, o))
        next(cell(1, 1, o)) :- does(oplayer, mark(1, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, x)) :- distinct(x, b) & true(cell(1, 1, x))
        next(cell(1, 1, x)) :- does(xplayer, mark(1, 1)) & true(cell(1, 1, b))
        next(cell(1, 2, b)) :- distinct(b, b) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 1), distinct(2, 1)) & does(oplayer, mark(1, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 1), distinct(2, 1)) & does(xplayer, mark(1, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 1), distinct(2, 2)) & does(oplayer, mark(1, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 1), distinct(2, 2)) & does(xplayer, mark(1, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 1), distinct(2, 3)) & does(oplayer, mark(1, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 1), distinct(2, 3)) & does(xplayer, mark(1, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 2), distinct(2, 1)) & does(oplayer, mark(2, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 2), distinct(2, 1)) & does(xplayer, mark(2, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 2), distinct(2, 2)) & does(oplayer, mark(2, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 2), distinct(2, 2)) & does(xplayer, mark(2, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 2), distinct(2, 3)) & does(oplayer, mark(2, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 2), distinct(2, 3)) & does(xplayer, mark(2, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 3), distinct(2, 1)) & does(oplayer, mark(3, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 3), distinct(2, 1)) & does(xplayer, mark(3, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 3), distinct(2, 2)) & does(oplayer, mark(3, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 3), distinct(2, 2)) & does(xplayer, mark(3, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 3), distinct(2, 3)) & does(oplayer, mark(3, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- or(distinct(1, 3), distinct(2, 3)) & does(xplayer, mark(3, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, o)) :- distinct(o, b) & true(cell(1, 2, o))
        next(cell(1, 2, o)) :- does(oplayer, mark(1, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, x)) :- distinct(x, b) & true(cell(1, 2, x))
        next(cell(1, 2, x)) :- does(xplayer, mark(1, 2)) & true(cell(1, 2, b))
        next(cell(1, 3, b)) :- distinct(b, b) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 1), distinct(3, 1)) & does(oplayer, mark(1, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 1), distinct(3, 1)) & does(xplayer, mark(1, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 1), distinct(3, 2)) & does(oplayer, mark(1, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 1), distinct(3, 2)) & does(xplayer, mark(1, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 1), distinct(3, 3)) & does(oplayer, mark(1, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 1), distinct(3, 3)) & does(xplayer, mark(1, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 2), distinct(3, 1)) & does(oplayer, mark(2, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 2), distinct(3, 1)) & does(xplayer, mark(2, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 2), distinct(3, 2)) & does(oplayer, mark(2, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 2), distinct(3, 2)) & does(xplayer, mark(2, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 2), distinct(3, 3)) & does(oplayer, mark(2, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 2), distinct(3, 3)) & does(xplayer, mark(2, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 3), distinct(3, 1)) & does(oplayer, mark(3, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 3), distinct(3, 1)) & does(xplayer, mark(3, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 3), distinct(3, 2)) & does(oplayer, mark(3, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 3), distinct(3, 2)) & does(xplayer, mark(3, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 3), distinct(3, 3)) & does(oplayer, mark(3, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- or(distinct(1, 3), distinct(3, 3)) & does(xplayer, mark(3, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, o)) :- distinct(o, b) & true(cell(1, 3, o))
        next(cell(1, 3, o)) :- does(oplayer, mark(1, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, x)) :- distinct(x, b) & true(cell(1, 3, x))
        next(cell(1, 3, x)) :- does(xplayer, mark(1, 3)) & true(cell(1, 3, b))
        next(cell(2, 1, b)) :- distinct(b, b) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 1), distinct(1, 1)) & does(oplayer, mark(1, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 1), distinct(1, 1)) & does(xplayer, mark(1, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 1), distinct(1, 2)) & does(oplayer, mark(1, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 1), distinct(1, 2)) & does(xplayer, mark(1, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 1), distinct(1, 3)) & does(oplayer, mark(1, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 1), distinct(1, 3)) & does(xplayer, mark(1, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 2), distinct(1, 1)) & does(oplayer, mark(2, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 2), distinct(1, 1)) & does(xplayer, mark(2, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 2), distinct(1, 2)) & does(oplayer, mark(2, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 2), distinct(1, 2)) & does(xplayer, mark(2, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 2), distinct(1, 3)) & does(oplayer, mark(2, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 2), distinct(1, 3)) & does(xplayer, mark(2, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 3), distinct(1, 1)) & does(oplayer, mark(3, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 3), distinct(1, 1)) & does(xplayer, mark(3, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 3), distinct(1, 2)) & does(oplayer, mark(3, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 3), distinct(1, 2)) & does(xplayer, mark(3, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 3), distinct(1, 3)) & does(oplayer, mark(3, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- or(distinct(2, 3), distinct(1, 3)) & does(xplayer, mark(3, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, o)) :- distinct(o, b) & true(cell(2, 1, o))
        next(cell(2, 1, o)) :- does(oplayer, mark(2, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, x)) :- distinct(x, b) & true(cell(2, 1, x))
        next(cell(2, 1, x)) :- does(xplayer, mark(2, 1)) & true(cell(2, 1, b))
        next(cell(2, 2, b)) :- distinct(b, b) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 1), distinct(2, 1)) & does(oplayer, mark(1, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 1), distinct(2, 1)) & does(xplayer, mark(1, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 1), distinct(2, 2)) & does(oplayer, mark(1, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 1), distinct(2, 2)) & does(xplayer, mark(1, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 1), distinct(2, 3)) & does(oplayer, mark(1, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 1), distinct(2, 3)) & does(xplayer, mark(1, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 2), distinct(2, 1)) & does(oplayer, mark(2, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 2), distinct(2, 1)) & does(xplayer, mark(2, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 2), distinct(2, 2)) & does(oplayer, mark(2, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 2), distinct(2, 2)) & does(xplayer, mark(2, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 2), distinct(2, 3)) & does(oplayer, mark(2, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 2), distinct(2, 3)) & does(xplayer, mark(2, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 3), distinct(2, 1)) & does(oplayer, mark(3, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 3), distinct(2, 1)) & does(xplayer, mark(3, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 3), distinct(2, 2)) & does(oplayer, mark(3, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 3), distinct(2, 2)) & does(xplayer, mark(3, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 3), distinct(2, 3)) & does(oplayer, mark(3, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- or(distinct(2, 3), distinct(2, 3)) & does(xplayer, mark(3, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, o)) :- distinct(o, b) & true(cell(2, 2, o))
        next(cell(2, 2, o)) :- does(oplayer, mark(2, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, x)) :- distinct(x, b) & true(cell(2, 2, x))
        next(cell(2, 2, x)) :- does(xplayer, mark(2, 2)) & true(cell(2, 2, b))
        next(cell(2, 3, b)) :- distinct(b, b) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 1), distinct(3, 1)) & does(oplayer, mark(1, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 1), distinct(3, 1)) & does(xplayer, mark(1, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 1), distinct(3, 2)) & does(oplayer, mark(1, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 1), distinct(3, 2)) & does(xplayer, mark(1, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 1), distinct(3, 3)) & does(oplayer, mark(1, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 1), distinct(3, 3)) & does(xplayer, mark(1, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 2), distinct(3, 1)) & does(oplayer, mark(2, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 2), distinct(3, 1)) & does(xplayer, mark(2, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 2), distinct(3, 2)) & does(oplayer, mark(2, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 2), distinct(3, 2)) & does(xplayer, mark(2, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 2), distinct(3, 3)) & does(oplayer, mark(2, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 2), distinct(3, 3)) & does(xplayer, mark(2, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 3), distinct(3, 1)) & does(oplayer, mark(3, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 3), distinct(3, 1)) & does(xplayer, mark(3, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 3), distinct(3, 2)) & does(oplayer, mark(3, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 3), distinct(3, 2)) & does(xplayer, mark(3, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 3), distinct(3, 3)) & does(oplayer, mark(3, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- or(distinct(2, 3), distinct(3, 3)) & does(xplayer, mark(3, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, o)) :- distinct(o, b) & true(cell(2, 3, o))
        next(cell(2, 3, o)) :- does(oplayer, mark(2, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, x)) :- distinct(x, b) & true(cell(2, 3, x))
        next(cell(2, 3, x)) :- does(xplayer, mark(2, 3)) & true(cell(2, 3, b))
        next(cell(3, 1, b)) :- distinct(b, b) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 1), distinct(1, 1)) & does(oplayer, mark(1, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 1), distinct(1, 1)) & does(xplayer, mark(1, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 1), distinct(1, 2)) & does(oplayer, mark(1, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 1), distinct(1, 2)) & does(xplayer, mark(1, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 1), distinct(1, 3)) & does(oplayer, mark(1, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 1), distinct(1, 3)) & does(xplayer, mark(1, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 2), distinct(1, 1)) & does(oplayer, mark(2, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 2), distinct(1, 1)) & does(xplayer, mark(2, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 2), distinct(1, 2)) & does(oplayer, mark(2, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 2), distinct(1, 2)) & does(xplayer, mark(2, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 2), distinct(1, 3)) & does(oplayer, mark(2, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 2), distinct(1, 3)) & does(xplayer, mark(2, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 3), distinct(1, 1)) & does(oplayer, mark(3, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 3), distinct(1, 1)) & does(xplayer, mark(3, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 3), distinct(1, 2)) & does(oplayer, mark(3, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 3), distinct(1, 2)) & does(xplayer, mark(3, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 3), distinct(1, 3)) & does(oplayer, mark(3, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- or(distinct(3, 3), distinct(1, 3)) & does(xplayer, mark(3, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, o)) :- distinct(o, b) & true(cell(3, 1, o))
        next(cell(3, 1, o)) :- does(oplayer, mark(3, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, x)) :- distinct(x, b) & true(cell(3, 1, x))
        next(cell(3, 1, x)) :- does(xplayer, mark(3, 1)) & true(cell(3, 1, b))
        next(cell(3, 2, b)) :- distinct(b, b) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 1), distinct(2, 1)) & does(oplayer, mark(1, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 1), distinct(2, 1)) & does(xplayer, mark(1, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 1), distinct(2, 2)) & does(oplayer, mark(1, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 1), distinct(2, 2)) & does(xplayer, mark(1, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 1), distinct(2, 3)) & does(oplayer, mark(1, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 1), distinct(2, 3)) & does(xplayer, mark(1, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 2), distinct(2, 1)) & does(oplayer, mark(2, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 2), distinct(2, 1)) & does(xplayer, mark(2, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 2), distinct(2, 2)) & does(oplayer, mark(2, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 2), distinct(2, 2)) & does(xplayer, mark(2, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 2), distinct(2, 3)) & does(oplayer, mark(2, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 2), distinct(2, 3)) & does(xplayer, mark(2, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 3), distinct(2, 1)) & does(oplayer, mark(3, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 3), distinct(2, 1)) & does(xplayer, mark(3, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 3), distinct(2, 2)) & does(oplayer, mark(3, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 3), distinct(2, 2)) & does(xplayer, mark(3, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 3), distinct(2, 3)) & does(oplayer, mark(3, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- or(distinct(3, 3), distinct(2, 3)) & does(xplayer, mark(3, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, o)) :- distinct(o, b) & true(cell(3, 2, o))
        next(cell(3, 2, o)) :- does(oplayer, mark(3, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, x)) :- distinct(x, b) & true(cell(3, 2, x))
        next(cell(3, 2, x)) :- does(xplayer, mark(3, 2)) & true(cell(3, 2, b))
        next(cell(3, 3, b)) :- distinct(b, b) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 1), distinct(3, 1)) & does(oplayer, mark(1, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 1), distinct(3, 1)) & does(xplayer, mark(1, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 1), distinct(3, 2)) & does(oplayer, mark(1, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 1), distinct(3, 2)) & does(xplayer, mark(1, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 1), distinct(3, 3)) & does(oplayer, mark(1, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 1), distinct(3, 3)) & does(xplayer, mark(1, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 2), distinct(3, 1)) & does(oplayer, mark(2, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 2), distinct(3, 1)) & does(xplayer, mark(2, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 2), distinct(3, 2)) & does(oplayer, mark(2, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 2), distinct(3, 2)) & does(xplayer, mark(2, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 2), distinct(3, 3)) & does(oplayer, mark(2, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 2), distinct(3, 3)) & does(xplayer, mark(2, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 3), distinct(3, 1)) & does(oplayer, mark(3, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 3), distinct(3, 1)) & does(xplayer, mark(3, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 3), distinct(3, 2)) & does(oplayer, mark(3, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 3), distinct(3, 2)) & does(xplayer, mark(3, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 3), distinct(3, 3)) & does(oplayer, mark(3, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- or(distinct(3, 3), distinct(3, 3)) & does(xplayer, mark(3, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, o)) :- distinct(o, b) & true(cell(3, 3, o))
        next(cell(3, 3, o)) :- does(oplayer, mark(3, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, x)) :- distinct(x, b) & true(cell(3, 3, x))
        next(cell(3, 3, x)) :- does(xplayer, mark(3, 3)) & true(cell(3, 3, b))
        next(control(oplayer)) :- true(control(xplayer))
        next(control(xplayer)) :- true(control(oplayer))
        role(oplayer)
        role(xplayer)
        terminal :- line(o)
        terminal :- line(x)
        terminal :- ~open
        "
    );
}
