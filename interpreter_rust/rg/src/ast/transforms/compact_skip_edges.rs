use crate::ast::{Edge, Error, Game, Node, Type};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type EdgeCounts = BTreeMap<Node<Arc<str>>, (usize, usize)>;

impl Game<Arc<str>> {
    pub fn compact_skip_edges(&mut self) -> Result<(), Error<Arc<str>>> {
        // If multiple edge series use the same binding names, rename them to
        // unique `bind_N` for `N = 1, 2, ...`.
        if !self.are_bindings_unique() {
            self.make_bindings_unique();
        }

        while let Some(x) = self.find_obsolete_edge() {
            self.edges.remove(x);
        }

        let mut edge_counts: EdgeCounts = BTreeMap::new();
        for Edge { lhs, rhs, .. } in &self.edges {
            edge_counts.entry(lhs.clone()).or_default().0 += 1;
            edge_counts.entry(rhs.clone()).or_default().1 += 1;
        }

        macro_rules! change_edge_count {
            ($node:expr, $position:tt, $fn:tt) => {
                if let Some(counts) = edge_counts.get_mut($node) {
                    counts.$position = counts.$position.$fn(1);
                }
            };
        }

        macro_rules! remove_edge {
            ($index:expr) => {
                let edge = self.edges.remove($index);
                change_edge_count!(&edge.lhs, 0, saturating_sub);
                change_edge_count!(&edge.rhs, 0, saturating_sub);
            };
        }

        while let Some((xs, y)) = self.compact_skip_edge_backward(&edge_counts) {
            for x in xs {
                change_edge_count!(&self.edges[x].rhs, 1, saturating_sub);
                change_edge_count!(&self.edges[y].rhs, 1, saturating_add);
                self.edges[x].rhs = self.edges[y].rhs.clone();
            }

            remove_edge!(y);
        }

        while let Some((x, ys)) = self.compact_skip_edge_forward(&edge_counts) {
            for y in ys {
                change_edge_count!(&self.edges[y].lhs, 0, saturating_sub);
                change_edge_count!(&self.edges[x].lhs, 0, saturating_add);
                self.edges[y].lhs = self.edges[x].lhs.clone();
            }

            remove_edge!(x);
        }

        while let Some(x) = self.compact_skip_edge_single(&edge_counts) {
            remove_edge!(x);
        }

        // Rename `bind_N: T` into `t: T` if `t` is not referenced.
        self.make_bindings_canonical();

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
    fn compact_skip_edge_backward(&self, edge_counts: &EdgeCounts) -> Option<(Vec<usize>, usize)> {
        for (y_index, y) in self.edges.iter().enumerate() {
            if y.label.is_skip()
                && !y.lhs.has_bindings()
                && edge_counts[&y.lhs].0 == 1
                && !self.is_reachability_target(&y.lhs)
            {
                for x in &self.edges {
                    if x.rhs == y.lhs
                        && (!y.rhs.has_bindings() || !x.label.is_player_assignment())
                        && self.incoming_edges(&y.lhs).all(|z| z.lhs == x.lhs)
                    {
                        let x_indexes = self
                            .edges
                            .iter()
                            .enumerate()
                            .filter(|(_, z)| z.lhs == x.lhs && z.rhs == x.rhs)
                            .map(|(index, _)| index)
                            .collect();
                        return Some((x_indexes, y_index));
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
    ///   5. y != Assignment of `player` OR a has no bindings
    fn compact_skip_edge_forward(&self, edge_counts: &EdgeCounts) -> Option<(usize, Vec<usize>)> {
        for (x_index, x) in self.edges.iter().enumerate() {
            if x.label.is_skip()
                && !x.rhs.has_bindings()
                && edge_counts[&x.rhs].1 == 1
                && !self.is_reachability_target(&x.rhs)
            {
                for y in &self.edges {
                    if x.rhs == y.lhs
                        && (!x.lhs.has_bindings() || !y.label.is_player_assignment())
                        && self.outgoing_edges(&x.rhs).all(|z| z.rhs == y.rhs)
                    {
                        let y_indexes = self
                            .edges
                            .iter()
                            .enumerate()
                            .filter(|(_, z)| z.lhs == y.lhs && z.rhs == y.rhs)
                            .map(|(index, _)| index)
                            .collect();
                        return Some((x_index, y_indexes));
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
    fn compact_skip_edge_single(&self, edge_counts: &EdgeCounts) -> Option<usize> {
        for (x_index, x) in self.edges.iter().enumerate() {
            if x.label.is_skip()
                && !x.lhs.has_bindings()
                && !x.lhs.is_begin()
                && edge_counts[&x.lhs].0 == 1
                && edge_counts[&x.lhs].1 == 0
                && !self.is_reachability_target(&x.lhs)
            {
                return Some(x_index);
            }
        }

        None
    }

    // If there is a skip edge from a to b, all other edges from a to b are obsolete.
    fn find_obsolete_edge(&self) -> Option<usize> {
        for (x_index, x) in self
            .edges
            .iter()
            .enumerate()
            .filter(|(_, e)| e.label.is_skip())
        {
            for (y_index, y) in self.edges.iter().enumerate() {
                if x.lhs == y.lhs && x.rhs == y.rhs && x_index != y_index {
                    return Some(y_index);
                }
            }
        }

        None
    }

    fn are_bindings_unique(&self) -> bool {
        for edge_index in 0..self.edges.len() {
            for (binding, _) in self.edges[edge_index].bindings() {
                let edges_using_binding = get_edges_using_binding(&self.edges, edge_index, binding);
                for edge_index in 0..self.edges.len() {
                    if !edges_using_binding.contains(&edge_index)
                        && self.edges[edge_index].get_binding(binding).is_some()
                    {
                        return false;
                    }
                }
            }
        }

        true
    }

    // TODO: Extract to a separate AST transform.
    fn make_bindings_canonical(&mut self) {
        // Iterate over indexes to eliminate multiple ownership.
        let edges = &mut self.edges;
        for x in 0..edges.len() {
            for (binding, type_) in edges[x].clone().bindings() {
                let Type::TypeReference { ref identifier } = type_.as_ref() else {
                    continue;
                };

                if !binding.starts_with("bind_") {
                    continue;
                }

                let fresh: Arc<str> = Arc::from(identifier.to_lowercase());
                if *binding == fresh {
                    continue;
                }

                let edges_to_rename = get_edges_using_binding(edges, x, binding);
                if edges_to_rename
                    .iter()
                    .any(|index| edges[*index].label.has_variable(&fresh))
                {
                    continue;
                }

                let mapping = BTreeMap::from([(binding.clone(), (fresh, type_.clone()))]);
                for y in edges_to_rename {
                    edges[y] = edges[y].rename_variables(&mapping);
                }
            }
        }
    }

    // TODO: Extract to a separate AST transform.
    fn make_bindings_unique(&mut self) {
        let mut index = 0;
        let mut mapped = BTreeSet::new();

        // Iterate over indexes to eliminate multiple ownership.
        let edges = &mut self.edges;
        for x in 0..edges.len() {
            for (binding, type_) in edges[x].clone().bindings() {
                if mapped.contains(&(x, binding.clone())) {
                    continue;
                }

                index += 1;

                // TODO: All `bind_*` bindings should be renamed before for safety.
                let fresh: Arc<str> = Arc::from(format!("bind_{index}"));
                let mapping = BTreeMap::from([(binding.clone(), (fresh.clone(), type_.clone()))]);
                for y in get_edges_using_binding(edges, x, binding) {
                    mapped.insert((y, fresh.clone()));
                    edges[y] = edges[y].rename_variables(&mapping);
                }
            }
        }
    }
}

fn get_edges_using_binding(
    edges: &[Edge<Arc<str>>],
    starting_edge_index: usize,
    binding: &Arc<str>,
) -> BTreeSet<usize> {
    let mut edges_using_binding = BTreeSet::from([starting_edge_index]);
    loop {
        let mut nothing_changed = true;
        for x in 0..edges.len() {
            if !edges_using_binding.contains(&x)
                && edges_using_binding.iter().any(|&y| {
                    let x = &edges[x];
                    let y = &edges[y];
                    x.lhs.has_binding(binding) && (x.lhs == y.lhs || x.lhs == y.rhs)
                        || x.rhs.has_binding(binding) && (x.rhs == y.lhs || x.rhs == y.rhs)
                })
            {
                nothing_changed = false;
                edges_using_binding.insert(x);
            }
        }

        if nothing_changed {
            break;
        }
    }

    edges_using_binding
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(compact_skip_edges, empty, "begin, end: ;", "begin, end: ;");

    test_transform!(
        compact_skip_edges,
        prefix,
        "begin, b: ; b, c: 1 == 1; c, end: 2 == 2;",
        "begin, c: 1 == 1; c, end: 2 == 2;"
    );

    test_transform!(
        compact_skip_edges,
        infix,
        "begin, b: 1 == 1; b, c: ; c, end: 2 == 2;",
        "begin, c: 1 == 1; c, end: 2 == 2;"
    );

    test_transform!(
        compact_skip_edges,
        suffix,
        "begin, b: 1 == 1; b, c: 2 == 2; c, end: ;",
        "begin, b: 1 == 1; b, end: 2 == 2;"
    );

    test_transform!(
        compact_skip_edges,
        player_assignment_prefix,
        "begin, b: player = x; b, c(t:T): ; c(t:T), end: ;",
        "begin, b: player = x; b, c(t:T): ; c(t:T), end: ;"
    );

    test_transform!(
        compact_skip_edges,
        player_assignment_suffix,
        "begin, b(t:T): ; b(t:T), c: ; c, end: player = x;",
        "begin, b(t:T): ; b(t:T), c: ; c, end: player = x;"
    );

    test_transform!(
        compact_skip_edges,
        simple_loop,
        "
            begin, loop: ;
            loop, cond: ;
            cond, true: 1 == 1;
            cond, false: 1 != 1;
            false, loop: ;
            true, end: player = keeper;
        ",
        "
            begin, loop: ;
            loop, cond: ;
            cond, true: 1 == 1;
            cond, loop: 1 != 1;
            true, end: player = keeper;
        "
    );

    test_transform!(
        compact_skip_edges,
        simple_loop_with_binds_single,
        "
            type X = { x };
            begin, loop(x: X): ;
            loop(x: X), cond(x: X): ;
            cond(x: X), true(x: X): 1 == 1;
            cond(x: X), false(x: X): 1 != 1;
            false(x: X), loop(x: X): ;
            true(x: X), end: player = keeper;
        ",
        "
            type X = { x };
            begin, loop(x: X): ;
            loop(x: X), cond(x: X): ;
            cond(x: X), true(x: X): 1 == 1;
            cond(x: X), false(x: X): 1 != 1;
            false(x: X), loop(x: X): ;
            true(x: X), end: player = keeper;
        "
    );

    test_transform!(
        compact_skip_edges,
        simple_loop_with_binds_multiple,
        "
            type X = { x };
            type Y = { y };
            type Z = { z };
            begin, loop(x: X)(y: Y): ;
            loop(x: X)(y: Y), cond(x: X)(z: Z): ;
            cond(x: X)(z: Z), true(x: X)(y: Y): 1 == 1;
            cond(x: X)(z: Z), false(x: X)(z: Z): 1 != 1;
            false(x: X)(z: Z), loop(x: X)(y: Y): ;
            true(x: X)(y: Y), end: player = keeper;
        ",
        "
            type X = { x };
            type Y = { y };
            type Z = { z };
            begin, loop(x: X)(y: Y): ;
            loop(x: X)(y: Y), cond(x: X)(z: Z): ;
            cond(x: X)(z: Z), true(x: X)(y: Y): 1 == 1;
            cond(x: X)(z: Z), false(x: X)(z: Z): 1 != 1;
            false(x: X)(z: Z), loop(x: X)(y: Y): ;
            true(x: X)(y: Y), end: player = keeper;
        "
    );

    test_transform!(
        compact_skip_edges,
        complex_loop_with_binds,
        "
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
        ",
        "
            type T = { a, b };
            var v: T = a;
            begin, a: ;
            a, c(t: T): T(t) != T(a);
            c(t: T), d: v = T(t);
            d, end: ;
            d, g(t: T): T(t) != T(a);
            g(t: T), h: v = T(t);
            h, a: ;
            h, end: ;
        "
    );

    test_transform!(
        compact_skip_edges,
        random5,
        "begin, a: 1 == 1;
        begin, b: ;
        b, a: 1 == 1;"
    );

    test_transform!(
        compact_skip_edges,
        disconnected_reachability,
        "
            begin, foo: ? a -> e;
            begin, bar: ! a -> e;
            foo, end: ;
            bar, end: ;

            a, b: ;
            b, c: 1 == 1;
            c, e: ;
            b, d: 1 == 1;
            d, e: ;
        ",
        "
            begin, end: ? a -> e;
            begin, end: ! a -> e;
            a, e: 1 == 1;
        "
    );

    test_transform!(
        compact_skip_edges,
        linear_ordered,
        "
            begin, x1(p: Position): 1 == 1;
            x1(p: Position), x3(p: Position): p != null;
            x3(p: Position), x4(p: Position): position = p;
            x4(p: Position), x2(p: Position): 1 == 1;
            x2(p: Position), end: 1 == 1;
        ",
        "
            begin, x1(p: Position): 1 == 1;
            x1(p: Position), x3(p: Position): p != null;
            x3(p: Position), x4(p: Position): position = p;
            x4(p: Position), x2(p: Position): 1 == 1;
            x2(p: Position), end: 1 == 1;
        "
    );

    test_transform!(
        compact_skip_edges,
        linear_unordered,
        "
            begin, x1(p: Position): 1 == 1;
            x2(p: Position), end: 1 == 1;
            x1(p: Position), x4(p: Position): p != null;
            x4(p: Position), x5(p: Position): position = p;
            x5(p: Position), x2(p: Position): 1 == 1;
        ",
        "
            begin, x1(p: Position): 1 == 1;
            x2(p: Position), end: 1 == 1;
            x1(p: Position), x4(p: Position): p != null;
            x4(p: Position), x5(p: Position): position = p;
            x5(p: Position), x2(p: Position): 1 == 1;
        "
    );

    test_transform!(
        compact_skip_edges,
        canonical_form,
        "type T = { 0, 1 }; var t: T = 0; a, b(x: T): x == t; c, d(x: T): x == t;",
        "type T = { 0, 1 }; var t: T = 0; a, b(bind_1: T): T(bind_1) == t; c, d(bind_2: T): T(bind_2) == t;"
    );

    test_transform!(
        compact_skip_edges,
        multi_skip_edge,
        "a, b: ;
        a, b: ;
        a, c: 1 == 1;",
        "a, b: ;
        a, c: 1 == 1;"
    );

    test_transform!(
        compact_skip_edges,
        skip_and_comparison,
        "a, b: ;
        a, b: 1 == 1;
        b, c: 2 == 2;",
        "a, c: 2 == 2;"
    );

    test_transform!(
        compact_skip_edges,
        multi_skip_forward,
        "a, b: ;
        a, c: 1 == 1;
        b, c: 2 == 2;",
        "a, c: 1 == 1;
        a, c: 2 == 2;"
    );

    test_transform!(
        compact_skip_edges,
        compact_forward_multi,
        "a, b: ;
        b, c: 1 == 1;
        b, c: 2 == 2;",
        "a, c: 1 == 1;
        a, c: 2 == 2;"
    );

    test_transform!(
        compact_skip_edges,
        compact_backward_multi,
        "a, b: 1 == 1;
        a, b: 2 == 2;
        b, c: ;",
        "a, c: 1 == 1;
        a, c: 2 == 2;"
    );
}
