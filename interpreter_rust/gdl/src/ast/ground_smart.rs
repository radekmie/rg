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
            if domain_map.is_empty() {
                rules.insert(rule);
                continue;
            }

            let mut grounded = BTreeSet::from([rule]);
            for (variable, domain) in &domain_map {
                let mut new_grounded = BTreeSet::new();
                for value in domain {
                    let unification = Unification::NotEmpty(vec![(variable, value)]);
                    for rule in &grounded {
                        let Some(mut rule) = rule.substitute(&unification) else {
                            continue;
                        };

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

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = Game::from($actual).ground_smart(&"distinct", &"or");
                let mut expect = Game::from($expect);

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
        one_variable_complex,
        "a(b(1)) c(X) :- a(X)",
        "a(b(1))" // FIXME: Complex variables should propagate.
                  // "a(b(1)) c(b(1)) :- a(b(1))"
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
        chain,
        "
        lessThan(X, Z) :- succ(X, Y) & lessThan(Y, Z)
        lessThan(X, Y) :- succ(X, Y)

        succ(0, 1)
        succ(1, 2)
        succ(2, 3)
        succ(3, 4)
        succ(4, 5)
        ",
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
        "
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
        col(1, o) :- true(mark(1, 1, o)) & true(mark(2, 1, o)) & true(mark(3, 1, o))
        col(1, x) :- true(mark(1, 1, x)) & true(mark(2, 1, x)) & true(mark(3, 1, x))
        col(2, o) :- true(mark(1, 2, o)) & true(mark(2, 2, o)) & true(mark(3, 2, o))
        col(2, x) :- true(mark(1, 2, x)) & true(mark(2, 2, x)) & true(mark(3, 2, x))
        col(3, o) :- true(mark(1, 3, o)) & true(mark(2, 3, o)) & true(mark(3, 3, o))
        col(3, x) :- true(mark(1, 3, x)) & true(mark(2, 3, x)) & true(mark(3, 3, x))
        diag(o) :- true(mark(1, 1, o)) & true(mark(2, 2, o)) & true(mark(3, 3, o))
        diag(o) :- true(mark(1, 3, o)) & true(mark(2, 2, o)) & true(mark(3, 1, o))
        diag(x) :- true(mark(1, 1, x)) & true(mark(2, 2, x)) & true(mark(3, 3, x))
        diag(x) :- true(mark(1, 3, x)) & true(mark(2, 2, x)) & true(mark(3, 1, x))
        emptyCell(1, 1) :- ~true(mark(1, 1, o)) & ~true(mark(1, 1, x))
        emptyCell(1, 2) :- ~true(mark(1, 2, o)) & ~true(mark(1, 2, x))
        emptyCell(1, 3) :- ~true(mark(1, 3, o)) & ~true(mark(1, 3, x))
        emptyCell(2, 1) :- ~true(mark(2, 1, o)) & ~true(mark(2, 1, x))
        emptyCell(2, 2) :- ~true(mark(2, 2, o)) & ~true(mark(2, 2, x))
        emptyCell(2, 3) :- ~true(mark(2, 3, o)) & ~true(mark(2, 3, x))
        emptyCell(3, 1) :- ~true(mark(3, 1, o)) & ~true(mark(3, 1, x))
        emptyCell(3, 2) :- ~true(mark(3, 2, o)) & ~true(mark(3, 2, x))
        emptyCell(3, 3) :- ~true(mark(3, 3, o)) & ~true(mark(3, 3, x))
        goal(oPlayer, 0) :- line(x)
        goal(oPlayer, 0) :- ~line(x) & ~line(o) & open
        goal(oPlayer, 100) :- line(o)
        goal(oPlayer, 50) :- ~line(x) & ~line(o) & ~open
        goal(xPlayer, 0) :- line(o)
        goal(xPlayer, 0) :- ~line(x) & ~line(o) & open
        goal(xPlayer, 100) :- line(x)
        goal(xPlayer, 50) :- ~line(x) & ~line(o) & ~open
        index(1)
        index(2)
        index(3)
        init(control(xPlayer))
        legal(oPlayer, noop) :- true(control(xPlayer))
        legal(oPlayer, play(1, 1, o)) :- emptyCell(1, 1) & true(control(oPlayer))
        legal(oPlayer, play(1, 2, o)) :- emptyCell(1, 2) & true(control(oPlayer))
        legal(oPlayer, play(1, 3, o)) :- emptyCell(1, 3) & true(control(oPlayer))
        legal(oPlayer, play(2, 1, o)) :- emptyCell(2, 1) & true(control(oPlayer))
        legal(oPlayer, play(2, 2, o)) :- emptyCell(2, 2) & true(control(oPlayer))
        legal(oPlayer, play(2, 3, o)) :- emptyCell(2, 3) & true(control(oPlayer))
        legal(oPlayer, play(3, 1, o)) :- emptyCell(3, 1) & true(control(oPlayer))
        legal(oPlayer, play(3, 2, o)) :- emptyCell(3, 2) & true(control(oPlayer))
        legal(oPlayer, play(3, 3, o)) :- emptyCell(3, 3) & true(control(oPlayer))
        legal(xPlayer, noop) :- true(control(oPlayer))
        legal(xPlayer, play(1, 1, x)) :- emptyCell(1, 1) & true(control(xPlayer))
        legal(xPlayer, play(1, 2, x)) :- emptyCell(1, 2) & true(control(xPlayer))
        legal(xPlayer, play(1, 3, x)) :- emptyCell(1, 3) & true(control(xPlayer))
        legal(xPlayer, play(2, 1, x)) :- emptyCell(2, 1) & true(control(xPlayer))
        legal(xPlayer, play(2, 2, x)) :- emptyCell(2, 2) & true(control(xPlayer))
        legal(xPlayer, play(2, 3, x)) :- emptyCell(2, 3) & true(control(xPlayer))
        legal(xPlayer, play(3, 1, x)) :- emptyCell(3, 1) & true(control(xPlayer))
        legal(xPlayer, play(3, 2, x)) :- emptyCell(3, 2) & true(control(xPlayer))
        legal(xPlayer, play(3, 3, x)) :- emptyCell(3, 3) & true(control(xPlayer))
        line(o) :- col(1, o)
        line(o) :- col(2, o)
        line(o) :- col(3, o)
        line(o) :- diag(o)
        line(o) :- row(1, o)
        line(o) :- row(2, o)
        line(o) :- row(3, o)
        line(x) :- col(1, x)
        line(x) :- col(2, x)
        line(x) :- col(3, x)
        line(x) :- diag(x)
        line(x) :- row(1, x)
        line(x) :- row(2, x)
        line(x) :- row(3, x)
        next(control(oPlayer)) :- true(control(xPlayer))
        next(control(xPlayer)) :- true(control(oPlayer))
        next(mark(1, 1, o)) :- does(oPlayer, play(1, 1, o)) & role(oPlayer)
        next(mark(1, 1, o)) :- does(xPlayer, play(1, 1, o)) & role(xPlayer)
        next(mark(1, 1, o)) :- true(mark(1, 1, o))
        next(mark(1, 1, x)) :- does(oPlayer, play(1, 1, x)) & role(oPlayer)
        next(mark(1, 1, x)) :- does(xPlayer, play(1, 1, x)) & role(xPlayer)
        next(mark(1, 1, x)) :- true(mark(1, 1, x))
        next(mark(1, 2, o)) :- does(oPlayer, play(1, 2, o)) & role(oPlayer)
        next(mark(1, 2, o)) :- does(xPlayer, play(1, 2, o)) & role(xPlayer)
        next(mark(1, 2, o)) :- true(mark(1, 2, o))
        next(mark(1, 2, x)) :- does(oPlayer, play(1, 2, x)) & role(oPlayer)
        next(mark(1, 2, x)) :- does(xPlayer, play(1, 2, x)) & role(xPlayer)
        next(mark(1, 2, x)) :- true(mark(1, 2, x))
        next(mark(1, 3, o)) :- does(oPlayer, play(1, 3, o)) & role(oPlayer)
        next(mark(1, 3, o)) :- does(xPlayer, play(1, 3, o)) & role(xPlayer)
        next(mark(1, 3, o)) :- true(mark(1, 3, o))
        next(mark(1, 3, x)) :- does(oPlayer, play(1, 3, x)) & role(oPlayer)
        next(mark(1, 3, x)) :- does(xPlayer, play(1, 3, x)) & role(xPlayer)
        next(mark(1, 3, x)) :- true(mark(1, 3, x))
        next(mark(2, 1, o)) :- does(oPlayer, play(2, 1, o)) & role(oPlayer)
        next(mark(2, 1, o)) :- does(xPlayer, play(2, 1, o)) & role(xPlayer)
        next(mark(2, 1, o)) :- true(mark(2, 1, o))
        next(mark(2, 1, x)) :- does(oPlayer, play(2, 1, x)) & role(oPlayer)
        next(mark(2, 1, x)) :- does(xPlayer, play(2, 1, x)) & role(xPlayer)
        next(mark(2, 1, x)) :- true(mark(2, 1, x))
        next(mark(2, 2, o)) :- does(oPlayer, play(2, 2, o)) & role(oPlayer)
        next(mark(2, 2, o)) :- does(xPlayer, play(2, 2, o)) & role(xPlayer)
        next(mark(2, 2, o)) :- true(mark(2, 2, o))
        next(mark(2, 2, x)) :- does(oPlayer, play(2, 2, x)) & role(oPlayer)
        next(mark(2, 2, x)) :- does(xPlayer, play(2, 2, x)) & role(xPlayer)
        next(mark(2, 2, x)) :- true(mark(2, 2, x))
        next(mark(2, 3, o)) :- does(oPlayer, play(2, 3, o)) & role(oPlayer)
        next(mark(2, 3, o)) :- does(xPlayer, play(2, 3, o)) & role(xPlayer)
        next(mark(2, 3, o)) :- true(mark(2, 3, o))
        next(mark(2, 3, x)) :- does(oPlayer, play(2, 3, x)) & role(oPlayer)
        next(mark(2, 3, x)) :- does(xPlayer, play(2, 3, x)) & role(xPlayer)
        next(mark(2, 3, x)) :- true(mark(2, 3, x))
        next(mark(3, 1, o)) :- does(oPlayer, play(3, 1, o)) & role(oPlayer)
        next(mark(3, 1, o)) :- does(xPlayer, play(3, 1, o)) & role(xPlayer)
        next(mark(3, 1, o)) :- true(mark(3, 1, o))
        next(mark(3, 1, x)) :- does(oPlayer, play(3, 1, x)) & role(oPlayer)
        next(mark(3, 1, x)) :- does(xPlayer, play(3, 1, x)) & role(xPlayer)
        next(mark(3, 1, x)) :- true(mark(3, 1, x))
        next(mark(3, 2, o)) :- does(oPlayer, play(3, 2, o)) & role(oPlayer)
        next(mark(3, 2, o)) :- does(xPlayer, play(3, 2, o)) & role(xPlayer)
        next(mark(3, 2, o)) :- true(mark(3, 2, o))
        next(mark(3, 2, x)) :- does(oPlayer, play(3, 2, x)) & role(oPlayer)
        next(mark(3, 2, x)) :- does(xPlayer, play(3, 2, x)) & role(xPlayer)
        next(mark(3, 2, x)) :- true(mark(3, 2, x))
        next(mark(3, 3, o)) :- does(oPlayer, play(3, 3, o)) & role(oPlayer)
        next(mark(3, 3, o)) :- does(xPlayer, play(3, 3, o)) & role(xPlayer)
        next(mark(3, 3, o)) :- true(mark(3, 3, o))
        next(mark(3, 3, x)) :- does(oPlayer, play(3, 3, x)) & role(oPlayer)
        next(mark(3, 3, x)) :- does(xPlayer, play(3, 3, x)) & role(xPlayer)
        next(mark(3, 3, x)) :- true(mark(3, 3, x))
        open :- emptyCell(1, 1)
        open :- emptyCell(1, 2)
        open :- emptyCell(1, 3)
        open :- emptyCell(2, 1)
        open :- emptyCell(2, 2)
        open :- emptyCell(2, 3)
        open :- emptyCell(3, 1)
        open :- emptyCell(3, 2)
        open :- emptyCell(3, 3)
        role(oPlayer)
        role(xPlayer)
        row(1, o) :- true(mark(1, 1, o)) & true(mark(1, 2, o)) & true(mark(1, 3, o))
        row(1, x) :- true(mark(1, 1, x)) & true(mark(1, 2, x)) & true(mark(1, 3, x))
        row(2, o) :- true(mark(2, 1, o)) & true(mark(2, 2, o)) & true(mark(2, 3, o))
        row(2, x) :- true(mark(2, 1, x)) & true(mark(2, 2, x)) & true(mark(2, 3, x))
        row(3, o) :- true(mark(3, 1, o)) & true(mark(3, 2, o)) & true(mark(3, 3, o))
        row(3, x) :- true(mark(3, 1, x)) & true(mark(3, 2, x)) & true(mark(3, 3, x))
        terminal :- line(o)
        terminal :- line(x)
        terminal :- ~open
        "
    );
}
