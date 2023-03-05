use crate::ast::{Error, Game};
use std::collections::BTreeMap;
use std::rc::Rc;

impl Game<Rc<str>> {
    pub fn compact_skip_edges(&mut self) -> Result<(), Error<Rc<str>>> {
        if !self.are_bindings_unique() {
            self.make_bindings_unique();
        }

        while let Some((x, y)) = self.compact_skip_edge_backward() {
            self.edges[x].rhs = self.edges[y].rhs.clone();
            self.edges.remove(y);
        }

        while let Some((x, y)) = self.compact_skip_edge_forward() {
            self.edges[y].lhs = self.edges[x].lhs.clone();
            self.edges.remove(x);
        }

        while let Some(x) = self.compact_skip_edge_single() {
            self.edges.remove(x);
        }

        Ok(())
    }

    /// Before:
    ///       x       y
    ///   a ----> b ----> c
    ///
    /// After:
    ///       x
    ///   a ----> c
    ///
    /// Conditions:
    ///   1. x != Assignment of `player` OR c has no bindings
    ///   2. y == Skip
    ///   3. b has no other incoming nor outgoing edges
    ///   4. b has no bindings
    ///   5. b is not a reachability target
    ///   6. there's no other edge between a and c (multiedges are not allowed)
    fn compact_skip_edge_backward(&self) -> Option<(usize, usize)> {
        for (y_index, y) in self.edges.iter().enumerate() {
            if y.label.is_skip()
                && !y.lhs.has_bindings()
                && self.outgoing_edges(&y.lhs).all(|z| z == y)
                && !self.is_reachability_target(&y.lhs)
            {
                for (x_index, x) in self.edges.iter().enumerate() {
                    if x.rhs == y.lhs
                        && (!y.rhs.has_bindings() || !x.label.is_player_assignment())
                        && !self.are_connected(&x.lhs, &y.rhs)
                        && self.incoming_edges(&y.lhs).all(|z| z == x)
                    {
                        return Some((x_index, y_index));
                    }
                }
            }
        }

        None
    }

    /// Before:
    ///       x       y
    ///   a ----> b ----> c
    ///
    /// After:
    ///       y
    ///   a ----> c
    ///
    /// Conditions:
    ///   1. x == Skip
    ///   2. b has no other incoming nor outgoing edges
    ///   3. b has no bindings
    ///   4. b is not a reachability target
    ///   5. there's no other edge between a and c (multiedges are not allowed)
    fn compact_skip_edge_forward(&self) -> Option<(usize, usize)> {
        for (x_index, x) in self.edges.iter().enumerate() {
            if x.label.is_skip()
                && !x.rhs.has_bindings()
                && self.incoming_edges(&x.rhs).all(|z| z == x)
                && !self.is_reachability_target(&x.rhs)
            {
                for (y_index, y) in self.edges.iter().enumerate() {
                    if x.rhs == y.lhs
                        && !self.are_connected(&x.lhs, &y.rhs)
                        && self.outgoing_edges(&x.rhs).all(|z| z == y)
                    {
                        return Some((x_index, y_index));
                    }
                }
            }
        }

        None
    }

    /// Before:
    ///       x
    ///   a ----> b
    ///
    /// After:
    ///
    ///   b
    ///
    /// Conditions:
    ///   1. x == Skip
    ///   2. a has no other incoming edges
    ///   3. a has no other outgoing edges
    ///   4. a has no bindings
    ///   5. a is not `begin`
    ///   6. a is not a reachability target
    fn compact_skip_edge_single(&self) -> Option<usize> {
        for (x_index, x) in self.edges.iter().enumerate() {
            if x.label.is_skip()
                && !x.lhs.has_bindings()
                && !x.lhs.is_begin()
                && self.incoming_edges(&x.lhs).next().is_none()
                && self.outgoing_edges(&x.lhs).all(|y| y == x)
                && !self.is_reachability_target(&x.lhs)
            {
                return Some(x_index);
            }
        }

        None
    }

    // TODO: This could be less restrictive and consider edges sharing the same
    // binding unique (e.g., "a(x: X), b(x: X): ;").
    fn are_bindings_unique(&self) -> bool {
        let mut binding_to_edge_name = BTreeMap::default();
        for edge in &self.edges {
            for edge_name in [&edge.lhs, &edge.rhs] {
                for (identifier, _) in edge_name.bindings() {
                    match binding_to_edge_name.get(identifier) {
                        Some(other) => {
                            if other != &edge_name {
                                return false;
                            }
                        }
                        _ => {
                            binding_to_edge_name.insert(identifier, edge_name);
                        }
                    }
                }
            }
        }

        true
    }

    fn make_bindings_unique(&mut self) {
        let mut index = 0;

        // Iterate over indexes to eliminate multiple ownership.
        let edges = &mut self.edges;
        for x in 0..edges.len() {
            let mapping = edges[x]
                .rhs
                .bindings()
                .map(|(binding, _)| {
                    index += 1;
                    (binding.clone(), Rc::from(format!("bind_{index}")))
                })
                .collect::<BTreeMap<_, _>>();

            if !mapping.is_empty() {
                for y in 0..edges.len() {
                    if x != y {
                        let rebind_lhs =
                            edges[x].rhs == edges[y].lhs || edges[x].lhs == edges[y].lhs;
                        let rebind_rhs =
                            edges[y].rhs == edges[x].lhs || edges[x].rhs == edges[y].rhs;

                        if rebind_lhs || rebind_rhs {
                            edges[y].label = edges[y].label.rename_variables(&mapping);
                        }

                        if rebind_lhs {
                            edges[y].lhs = edges[y].lhs.rename_variables(&mapping);
                        }

                        if rebind_rhs {
                            edges[y].rhs = edges[y].rhs.rename_variables(&mapping);
                        }
                    }
                }

                edges[x] = edges[x].rename_variables(&mapping);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parser::game;
    use map_id::MapId;
    use nom::combinator::all_consuming;
    use std::rc::Rc;

    fn parse(input: &str) -> Game<Rc<str>> {
        let (_, game) = all_consuming(game)(input).unwrap();
        game.map_id(&mut |id| Rc::from(*id))
    }

    macro_rules! test {
        ($name:ident { $($actual:tt)* } { $($expect:tt)* }) => {
            #[test]
            fn $name() {
                let mut actual = parse(stringify!($($actual)*));
                actual.compact_skip_edges().unwrap();
                let expect = parse(stringify!($($expect)*));

                assert_eq!(actual, expect, "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n");
            }
        };
    }

    test!(
        empty
        { begin, end: ; }
        { begin, end: ; }
    );

    test!(
        prefix
        { begin, b: ; b, c: 1 == 1; c, end: 2 == 2; }
        { begin, c: 1 == 1; c, end: 2 == 2; }
    );

    test!(
        infix
        { begin, b: 1 == 1; b, c: ; c, end: 2 == 2; }
        { begin, c: 1 == 1; c, end: 2 == 2; }
    );

    test!(
        suffix
        { begin, b: 1 == 1; b, c: 2 == 2; c, end: ; }
        { begin, b: 1 == 1; b, end: 2 == 2; }
    );

    test!(
        simple_loop
        {
            begin, loop: ;
            loop, cond: ;
            cond, true: 1 == 1;
            cond, false: 1 != 1;
            false, loop: ;
            true, end: player = keeper;
        }
        {
            begin, loop: ;
            loop, cond: ;
            cond, true: 1 == 1;
            cond, loop: 1 != 1;
            true, end: player = keeper;
        }
    );

    test!(
        simple_loop_with_binds
        {
            type X = { x };
            begin, loop(x: X): ;
            loop(x: X), cond(x: X): ;
            cond(x: X), true(x: X): 1 == 1;
            cond(x: X), false(x: X): 1 != 1;
            false(x: X), loop(x: X): ;
            true(x: X), end: player = keeper;
        }
        {
            type X = { x };
            begin, loop(bind_5: X): ;
            loop(bind_5: X), cond(bind_2: X): ;
            cond(bind_2: X), true(bind_3: X): 1 == 1;
            cond(bind_2: X), false(bind_4: X): 1 != 1;
            false(bind_4: X), loop(bind_5: X): ;
            true(bind_3: X), end: player = keeper;
        }
    );

    test!(
        complex_loop_with_binds
        {
            type T = { a, b };
            var v: T = a;
            begin, a: ;
            a, b: ;
            b, c(t: T): T(t) != T(a);
            c(t: T), d: v = t;
            d, e: ;
            d, f: ;
            e, g(t: T): T(t) != T(a);
            f, end: ;
            g(t: T), h: v = t;
            h, a: ;
            h, end: ;
        }
        {
            type T = { a, b };
            var v: T = a;
            begin, a: ;
            a, c(bind_1: T): T(bind_1) != T(a);
            c(bind_1: T), d: v = bind_1;
            d, end: ;
            d, g(bind_2: T): T(bind_2) != T(a);
            g(bind_2: T), h: v = bind_2;
            h, a: ;
            h, end: ;
        }
    );

    test!(
        disconnected_reachability
        {
            begin, foo: ? a -> e;
            begin, bar: ! a -> e;
            foo, end: ;
            bar, end: ;

            a, b: ;
            b, c: 1 == 1;
            c, e: ;
            b, d: 1 == 1;
            d, e: ;
        }
        {
            begin, end: ? a -> e;
            begin, bar: ! a -> e;
            bar, end: ;

            a, b: ;
            b, e: 1 == 1;
            b, d: 1 == 1;
            d, e: ;
        }
    );
}
