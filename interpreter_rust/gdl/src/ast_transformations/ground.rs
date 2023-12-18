use crate::ast::Game;

impl<Symbol: Clone + Ord> Game<Symbol> {
    pub fn ground(&self) -> Self {
        let mut rules = self.0.clone();
        let mut subterms: Vec<Vec<_>> = rules
            .iter()
            .map(|rule| rule.subterms().into_iter().cloned().collect())
            .collect();

        loop {
            let mut any_grounding_happened = false;
            for i in 0..rules.len() {
                if rules[i].has_variable() {
                    continue;
                }

                for j in 0..rules.len() {
                    if i == j {
                        continue;
                    }

                    if let Some(mapping) = subterms[j]
                        .iter()
                        .flat_map(|lhs| subterms[i].iter().map(move |rhs| (lhs, rhs)))
                        .find_map(|(lhs, rhs)| lhs.unify(rhs).as_mapping())
                    {
                        let rule = rules[j].substitute(&mapping);
                        if !rules.contains(&rule) {
                            subterms.insert(j, rule.subterms().into_iter().cloned().collect());
                            rules.insert(j, rule);
                            any_grounding_happened = true;
                        }
                    }
                }
            }

            if !any_grounding_happened {
                break;
            }
        }

        rules.retain(|rule| !rule.has_variable());

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
        "a(1) a(2) b(1) :- a(1) & a(1) b(2) :- a(2) & a(2)"
    );

    test!(
        two_variables_one_precondition,
        "a(1, 2) a(3, 4) b(X, Y) :- a(X, Y)",
        "a(1, 2) a(3, 4) b(1, 2) :- a(1, 2) b(3, 4) :- a(3, 4)"
    );

    test!(
        two_variables_two_preconditions,
        "a(1, 2) a(3, 4) b(X, Y) :- a(X, Y) & a(X, Y)",
        "a(1, 2) a(3, 4) b(1, 2) :- a(1, 2) & a(1, 2) b(3, 4) :- a(3, 4) & a(3, 4)"
    );

    test!(
        two_variables_partial_unification,
        "a(1, 2) a(3, 4) b(1, Y) :- a(1, Y)",
        "a(1, 2) a(3, 4) b(1, 2) :- a(1, 2)"
    );

    test!(
        two_variables_cross_product_1,
        "a(1) a(2) b(X, Y) :- a(X) & a(Y)",
        "a(1) a(2) b(1, 1) :- a(1) & a(1) b(1, 2) :- a(1) & a(2) b(2, 1) :- a(2) & a(1) b(2, 2) :- a(2) & a(2)"
    );

    test!(
        two_variables_cross_product_2,
        "a(1) a(2) b(X) :- a(X) & a(Y)",
        "a(1) a(2) b(1) :- a(1) & a(1) b(1) :- a(1) & a(2) b(2) :- a(2) & a(1) b(2) :- a(2) & a(2)"
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

        base(cell(1, 1, b)) :- index(1) & index(1)
        base(cell(1, 2, b)) :- index(1) & index(2)
        base(cell(2, 1, b)) :- index(2) & index(1)
        base(cell(2, 2, b)) :- index(2) & index(2)

        base(cell(1, 1, x)) :- index(1) & index(1)
        base(cell(1, 2, x)) :- index(1) & index(2)
        base(cell(2, 1, x)) :- index(2) & index(1)
        base(cell(2, 2, x)) :- index(2) & index(2)

        base(cell(1, 1, o)) :- index(1) & index(1)
        base(cell(1, 2, o)) :- index(1) & index(2)
        base(cell(2, 1, o)) :- index(2) & index(1)
        base(cell(2, 2, o)) :- index(2) & index(2)

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
}
