use crate::ast::unify::Unification;
use crate::ast::Game;
use std::collections::BTreeSet;

impl<Id: Clone + Ord> Game<Id> {
    pub fn ground_smart(self, distinct: &Id, or: &Id) -> Self {
        let domain_graph = self.domain_graph();
        let static_terms = self.static_terms();

        let mut rules = BTreeSet::new();
        for rule in self.0 {
            let domain_map = rule.domain_map(distinct, or, &domain_graph);
            let mut grounded = BTreeSet::from([rule]);
            for (variable, domain) in &domain_map {
                let mut new_grounded = BTreeSet::new();
                for value in domain {
                    let unification = Unification::NotEmpty(vec![(variable, value)]);
                    for rule in &grounded {
                        let mut rule = rule.substitute(&unification);
                        if rule.prune_static_terms(&static_terms) {
                            continue;
                        }

                        let Some(mut rule) = rule.eval_distinct(distinct, or) else {
                            continue;
                        };

                        rule.predicates.sort_unstable();
                        rule.predicates.dedup();
                        new_grounded.insert(rule);
                    }
                }

                grounded = new_grounded;
            }

            rules.extend(grounded);
        }

        Self(rules.into_iter().collect())
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
                let mut actual = parse($actual).ground_smart(&"distinct", &"or");
                let mut expect = parse($expect);

                actual.0.sort_unstable();
                expect.0.sort_unstable();

                assert_eq!(actual.as_infix().to_string(), expect.as_infix().to_string());
            }
        };
    }

    test!(
        one_variable_one_precondition,
        "a(1) a(2) b(X) :- a(X)",
        "a(1) a(2) b(1) b(2)"
    );

    test!(
        one_variable_two_preconditions,
        "a(1) a(2) b(X) :- a(X) & a(X)",
        "a(1) a(2) b(1) b(2)"
    );

    test!(
        two_variables_one_precondition,
        "a(1, 2) a(3, 4) b(X, Y) :- a(X, Y)",
        "a(1, 2) a(3, 4) b(1, 2) b(3, 4)"
    );

    test!(
        two_variables_two_preconditions,
        "a(1, 2) a(3, 4) b(X, Y) :- a(X, Y) & a(X, Y)",
        "a(1, 2) a(3, 4) b(1, 2) b(3, 4)"
    );

    test!(
        two_variables_partial_unification,
        "a(1, 2) a(3, 4) b(1, Y) :- a(1, Y)",
        "a(1, 2) a(3, 4) b(1, 2)"
    );

    test!(
        two_variables_cross_product_1,
        "a(1) a(2) b(X, Y) :- a(X) & a(Y)",
        "a(1) a(2) b(1, 1) b(1, 2) b(2, 1) b(2, 2)"
    );

    test!(
        two_variables_cross_product_2,
        "a(1) a(2) b(X) :- a(X) & a(Y)",
        "a(1) a(2) b(1) b(2)"
    );

    test!(
        nested_simple,
        "a(1) b(X) :- c(d(1, X)) e(d(X, Y)) :- a(X) & f(Y) f(2) :- a(1)",
        "a(1) b(2) :- c(d(1, 2)) e(d(1, 2)) :- f(2) f(2) :- a(1)"
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

        base(cell(1, 1, b))
        base(cell(1, 2, b))
        base(cell(2, 1, b))
        base(cell(2, 2, b))

        base(cell(1, 1, o))
        base(cell(1, 2, o))
        base(cell(2, 1, o))
        base(cell(2, 2, o))

        base(cell(1, 1, x))
        base(cell(1, 2, x))
        base(cell(2, 1, x))
        base(cell(2, 2, x))

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
        include_str!("../../../../games/kif/ticTacToe.kif"),
        "
        base(cell(1, 1, b))
        base(cell(1, 1, o))
        base(cell(1, 1, x))
        base(cell(1, 2, b))
        base(cell(1, 2, o))
        base(cell(1, 2, x))
        base(cell(1, 3, b))
        base(cell(1, 3, o))
        base(cell(1, 3, x))
        base(cell(2, 1, b))
        base(cell(2, 1, o))
        base(cell(2, 1, x))
        base(cell(2, 2, b))
        base(cell(2, 2, o))
        base(cell(2, 2, x))
        base(cell(2, 3, b))
        base(cell(2, 3, o))
        base(cell(2, 3, x))
        base(cell(3, 1, b))
        base(cell(3, 1, o))
        base(cell(3, 1, x))
        base(cell(3, 2, b))
        base(cell(3, 2, o))
        base(cell(3, 2, x))
        base(cell(3, 3, b))
        base(cell(3, 3, o))
        base(cell(3, 3, x))
        base(control(oplayer)) :- role(oplayer)
        base(control(xplayer)) :- role(xplayer)
        open :- true(cell(1, 1, b))
        open :- true(cell(1, 2, b))
        open :- true(cell(1, 3, b))
        open :- true(cell(2, 1, b))
        open :- true(cell(2, 2, b))
        open :- true(cell(2, 3, b))
        open :- true(cell(3, 1, b))
        open :- true(cell(3, 2, b))
        open :- true(cell(3, 3, b))
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
        goal(oplayer, 50) :- ~line(x) & ~line(o) & ~open
        goal(xplayer, 0) :- line(o)
        goal(xplayer, 100) :- line(x)
        goal(xplayer, 50) :- ~line(x) & ~line(o) & ~open
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
        input(oplayer, noop) :- role(oplayer)
        input(oplayer, mark(1, 1)) :- role(oplayer)
        input(oplayer, mark(1, 2)) :- role(oplayer)
        input(oplayer, mark(1, 3)) :- role(oplayer)
        input(oplayer, mark(2, 1)) :- role(oplayer)
        input(oplayer, mark(2, 2)) :- role(oplayer)
        input(oplayer, mark(2, 3)) :- role(oplayer)
        input(oplayer, mark(3, 1)) :- role(oplayer)
        input(oplayer, mark(3, 2)) :- role(oplayer)
        input(oplayer, mark(3, 3)) :- role(oplayer)
        input(xplayer, noop) :- role(xplayer)
        input(xplayer, mark(1, 1)) :- role(xplayer)
        input(xplayer, mark(1, 2)) :- role(xplayer)
        input(xplayer, mark(1, 3)) :- role(xplayer)
        input(xplayer, mark(2, 1)) :- role(xplayer)
        input(xplayer, mark(2, 2)) :- role(xplayer)
        input(xplayer, mark(2, 3)) :- role(xplayer)
        input(xplayer, mark(3, 1)) :- role(xplayer)
        input(xplayer, mark(3, 2)) :- role(xplayer)
        input(xplayer, mark(3, 3)) :- role(xplayer)
        legal(oplayer, noop) :- true(control(xplayer))
        legal(oplayer, mark(1, 1)) :- true(cell(1, 1, b)) & true(control(oplayer))
        legal(oplayer, mark(1, 2)) :- true(cell(1, 2, b)) & true(control(oplayer))
        legal(oplayer, mark(1, 3)) :- true(cell(1, 3, b)) & true(control(oplayer))
        legal(oplayer, mark(2, 1)) :- true(cell(2, 1, b)) & true(control(oplayer))
        legal(oplayer, mark(2, 2)) :- true(cell(2, 2, b)) & true(control(oplayer))
        legal(oplayer, mark(2, 3)) :- true(cell(2, 3, b)) & true(control(oplayer))
        legal(oplayer, mark(3, 1)) :- true(cell(3, 1, b)) & true(control(oplayer))
        legal(oplayer, mark(3, 2)) :- true(cell(3, 2, b)) & true(control(oplayer))
        legal(oplayer, mark(3, 3)) :- true(cell(3, 3, b)) & true(control(oplayer))
        legal(xplayer, noop) :- true(control(oplayer))
        legal(xplayer, mark(1, 1)) :- true(cell(1, 1, b)) & true(control(xplayer))
        legal(xplayer, mark(1, 2)) :- true(cell(1, 2, b)) & true(control(xplayer))
        legal(xplayer, mark(1, 3)) :- true(cell(1, 3, b)) & true(control(xplayer))
        legal(xplayer, mark(2, 1)) :- true(cell(2, 1, b)) & true(control(xplayer))
        legal(xplayer, mark(2, 2)) :- true(cell(2, 2, b)) & true(control(xplayer))
        legal(xplayer, mark(2, 3)) :- true(cell(2, 3, b)) & true(control(xplayer))
        legal(xplayer, mark(3, 1)) :- true(cell(3, 1, b)) & true(control(xplayer))
        legal(xplayer, mark(3, 2)) :- true(cell(3, 2, b)) & true(control(xplayer))
        legal(xplayer, mark(3, 3)) :- true(cell(3, 3, b)) & true(control(xplayer))
        next(cell(1, 1, b)) :- does(oplayer, mark(1, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(oplayer, mark(1, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(oplayer, mark(2, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(oplayer, mark(2, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(oplayer, mark(2, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(oplayer, mark(3, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(oplayer, mark(3, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(oplayer, mark(3, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(1, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(1, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(2, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(2, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(2, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(3, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(3, 2)) & true(cell(1, 1, b))
        next(cell(1, 1, b)) :- does(xplayer, mark(3, 3)) & true(cell(1, 1, b))
        next(cell(1, 1, o)) :- does(oplayer, mark(1, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, o)) :- true(cell(1, 1, o))
        next(cell(1, 1, x)) :- does(xplayer, mark(1, 1)) & true(cell(1, 1, b))
        next(cell(1, 1, x)) :- true(cell(1, 1, x))
        next(cell(1, 2, b)) :- does(oplayer, mark(1, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(oplayer, mark(1, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(oplayer, mark(2, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(oplayer, mark(2, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(oplayer, mark(2, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(oplayer, mark(3, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(oplayer, mark(3, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(oplayer, mark(3, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(1, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(1, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(2, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(2, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(2, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(3, 1)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(3, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, b)) :- does(xplayer, mark(3, 3)) & true(cell(1, 2, b))
        next(cell(1, 2, o)) :- does(oplayer, mark(1, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, o)) :- true(cell(1, 2, o))
        next(cell(1, 2, x)) :- does(xplayer, mark(1, 2)) & true(cell(1, 2, b))
        next(cell(1, 2, x)) :- true(cell(1, 2, x))
        next(cell(1, 3, b)) :- does(oplayer, mark(1, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(oplayer, mark(1, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(oplayer, mark(2, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(oplayer, mark(2, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(oplayer, mark(2, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(oplayer, mark(3, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(oplayer, mark(3, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(oplayer, mark(3, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(1, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(1, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(2, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(2, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(2, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(3, 1)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(3, 2)) & true(cell(1, 3, b))
        next(cell(1, 3, b)) :- does(xplayer, mark(3, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, o)) :- does(oplayer, mark(1, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, o)) :- true(cell(1, 3, o))
        next(cell(1, 3, x)) :- does(xplayer, mark(1, 3)) & true(cell(1, 3, b))
        next(cell(1, 3, x)) :- true(cell(1, 3, x))
        next(cell(2, 1, b)) :- does(oplayer, mark(1, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(oplayer, mark(1, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(oplayer, mark(1, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(oplayer, mark(2, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(oplayer, mark(2, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(oplayer, mark(3, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(oplayer, mark(3, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(oplayer, mark(3, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(1, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(1, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(1, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(2, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(2, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(3, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(3, 2)) & true(cell(2, 1, b))
        next(cell(2, 1, b)) :- does(xplayer, mark(3, 3)) & true(cell(2, 1, b))
        next(cell(2, 1, o)) :- does(oplayer, mark(2, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, o)) :- true(cell(2, 1, o))
        next(cell(2, 1, x)) :- does(xplayer, mark(2, 1)) & true(cell(2, 1, b))
        next(cell(2, 1, x)) :- true(cell(2, 1, x))
        next(cell(2, 2, b)) :- does(oplayer, mark(1, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(oplayer, mark(1, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(oplayer, mark(1, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(oplayer, mark(2, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(oplayer, mark(2, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(oplayer, mark(3, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(oplayer, mark(3, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(oplayer, mark(3, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(1, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(1, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(1, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(2, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(2, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(3, 1)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(3, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, b)) :- does(xplayer, mark(3, 3)) & true(cell(2, 2, b))
        next(cell(2, 2, o)) :- does(oplayer, mark(2, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, o)) :- true(cell(2, 2, o))
        next(cell(2, 2, x)) :- does(xplayer, mark(2, 2)) & true(cell(2, 2, b))
        next(cell(2, 2, x)) :- true(cell(2, 2, x))
        next(cell(2, 3, b)) :- does(oplayer, mark(1, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(oplayer, mark(1, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(oplayer, mark(1, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(oplayer, mark(2, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(oplayer, mark(2, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(oplayer, mark(3, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(oplayer, mark(3, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(oplayer, mark(3, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(1, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(1, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(1, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(2, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(2, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(3, 1)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(3, 2)) & true(cell(2, 3, b))
        next(cell(2, 3, b)) :- does(xplayer, mark(3, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, o)) :- does(oplayer, mark(2, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, o)) :- true(cell(2, 3, o))
        next(cell(2, 3, x)) :- does(xplayer, mark(2, 3)) & true(cell(2, 3, b))
        next(cell(2, 3, x)) :- true(cell(2, 3, x))
        next(cell(3, 1, b)) :- does(oplayer, mark(1, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(oplayer, mark(1, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(oplayer, mark(1, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(oplayer, mark(2, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(oplayer, mark(2, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(oplayer, mark(2, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(oplayer, mark(3, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(oplayer, mark(3, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(1, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(1, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(1, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(2, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(2, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(2, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(3, 2)) & true(cell(3, 1, b))
        next(cell(3, 1, b)) :- does(xplayer, mark(3, 3)) & true(cell(3, 1, b))
        next(cell(3, 1, o)) :- does(oplayer, mark(3, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, o)) :- true(cell(3, 1, o))
        next(cell(3, 1, x)) :- does(xplayer, mark(3, 1)) & true(cell(3, 1, b))
        next(cell(3, 1, x)) :- true(cell(3, 1, x))
        next(cell(3, 2, b)) :- does(oplayer, mark(1, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(oplayer, mark(1, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(oplayer, mark(1, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(oplayer, mark(2, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(oplayer, mark(2, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(oplayer, mark(2, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(oplayer, mark(3, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(oplayer, mark(3, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(1, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(1, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(1, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(2, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(2, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(2, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(3, 1)) & true(cell(3, 2, b))
        next(cell(3, 2, b)) :- does(xplayer, mark(3, 3)) & true(cell(3, 2, b))
        next(cell(3, 2, o)) :- does(oplayer, mark(3, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, o)) :- true(cell(3, 2, o))
        next(cell(3, 2, x)) :- does(xplayer, mark(3, 2)) & true(cell(3, 2, b))
        next(cell(3, 2, x)) :- true(cell(3, 2, x))
        next(cell(3, 3, b)) :- does(oplayer, mark(1, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(oplayer, mark(1, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(oplayer, mark(1, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(oplayer, mark(2, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(oplayer, mark(2, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(oplayer, mark(2, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(oplayer, mark(3, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(oplayer, mark(3, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(1, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(1, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(1, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(2, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(2, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(2, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(3, 1)) & true(cell(3, 3, b))
        next(cell(3, 3, b)) :- does(xplayer, mark(3, 2)) & true(cell(3, 3, b))
        next(cell(3, 3, o)) :- does(oplayer, mark(3, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, o)) :- true(cell(3, 3, o))
        next(cell(3, 3, x)) :- does(xplayer, mark(3, 3)) & true(cell(3, 3, b))
        next(cell(3, 3, x)) :- true(cell(3, 3, x))
        next(control(oplayer)) :- true(control(xplayer))
        next(control(xplayer)) :- true(control(oplayer))
        role(oplayer)
        role(xplayer)
        terminal :- ~open
        terminal :- line(o)
        terminal :- line(x)
        "
    );
}
