use crate::ast::{Edge, Error, Game, Node, SetWithIdx};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

type Id = Arc<str>;
type RemovedAndMoved<T> = (Vec<usize>, Vec<T>);

impl Game<Id> {
    pub fn compact_skip_edges(&mut self) -> Result<(), Error<Arc<str>>> {
        self.make_bindings_unique();

        loop {
            let to_remove = self.skip_all_obsolete();
            let mut changed = !to_remove.is_empty();
            self.remove_edges(to_remove);

            let (to_remove, to_move) = self.skip_edge_backward();
            changed |= !to_move.is_empty();
            for (x_y_idxs, z) in to_move {
                for x_y_idx in x_y_idxs {
                    Arc::make_mut(&mut self.edges[x_y_idx]).rhs = z.clone();
                }
            }
            self.remove_edges(to_remove);

            let (to_remove, to_move) = self.skip_edge_forward();
            changed |= !to_move.is_empty();
            for (x, y_z_idxs) in to_move {
                for y_z_idx in y_z_idxs {
                    Arc::make_mut(&mut self.edges[y_z_idx]).lhs = x.clone();
                }
            }
            self.remove_edges(to_remove);

            if !changed {
                break;
            }
        }

        Ok(())
    }

    fn remove_edges(&mut self, mut to_remove: Vec<usize>) {
        to_remove.sort_unstable();
        for idx in to_remove.into_iter().rev() {
            self.edges.remove(idx);
        }
    }

    /// Before:
    ///
    ///   x ----> y ----> z
    ///
    /// After:
    ///
    ///   x ------------> z
    ///           y ----> z
    ///
    /// Conditions:
    ///   1. x -> y == Skip
    ///   2. y has no other outgoing edges
    ///   3. y is not a reachability target
    ///   4. x -> z will not connect two separate binds
    ///   5. x -> y != Assignment of `player` OR z has no bindings
    ///   6. If all edges incoming to y satisfy 6. then remove y -> z
    fn skip_edge_backward(&self) -> RemovedAndMoved<(Vec<usize>, Node<Id>)> {
        let reachability_targets = self.reachability_targets();
        let next_edges = self.next_edges_with_idx();
        let prev_edges = self.prev_edges_with_idx();
        let mut to_remove = vec![];
        let mut to_add = vec![];
        let mut used_nodes = BTreeSet::new();
        for (z, z_in) in &prev_edges {
            if used_nodes.contains(z) {
                continue;
            }
            for (y_z_idx, y_z) in z_in {
                let y = &y_z.lhs;
                if !y_z.label.is_skip() // (1)
                    || used_nodes.contains(y) // For safety, this node could have been modified
                    || reachability_targets.contains(y) // (3)
                    || next_edges.get(y).is_none_or(|n| n.len() != 1)
                // (2)
                {
                    continue;
                }

                if let Some(y_in) = prev_edges.get(y) {
                    let to_move = y_in
                        .iter()
                        .filter(|(x_y_idx, x_y)| {
                            are_bindings_safe(&x_y.lhs, y, z) // (4)
                                && (!z.has_bindings() || !x_y.label.is_player_assignment()) // (5)
                                && !to_remove.contains(x_y_idx) // For safety, this edge could have been removed
                        })
                        .map(|(x_y_idx, _)| *x_y_idx)
                        .collect::<Vec<_>>();
                    if to_move.len() == y_in.len() {
                        // (6)
                        to_remove.push(*y_z_idx);
                    }
                    if !to_move.is_empty() {
                        used_nodes.insert(y);
                        used_nodes.insert(z);
                        to_add.push((to_move, (*z).clone()));
                    }
                }
            }
        }
        (to_remove, to_add)
    }

    /// Before:
    ///
    ///   x ----> y ----> z
    ///
    /// After:
    ///
    ///   x ------------> z
    ///   x ----> y
    ///
    /// Conditions:
    ///   1. x -> y == Skip
    ///   2. y has no other incoming edges
    ///   3. y is not a reachability target
    ///   4. x -> z will not connect two separate binds
    ///   5. y -> z != Assignment of `player` OR x has no bindings
    ///   6. If all edges outgoing from y satisfy 6. then remove x -> y
    fn skip_edge_forward(&self) -> RemovedAndMoved<(Node<Id>, Vec<usize>)> {
        let reachability_targets = self.reachability_targets();
        let next_edges = self.next_edges_with_idx();
        let prev_edges = self.prev_edges_with_idx();
        let mut to_remove = vec![];
        let mut to_add = vec![];
        let mut used_nodes = BTreeSet::new();
        for (x, x_out) in &next_edges {
            if used_nodes.contains(x) {
                continue;
            }
            for (x_y_idx, x_y) in x_out {
                let y = &x_y.rhs;
                if !x_y.label.is_skip() // (1)
                    || used_nodes.contains(y) // For safety, this node could have been modified
                    || reachability_targets.contains(y) // (3)
                    || prev_edges.get(y).is_none_or(|n| n.len() != 1)
                // (2)
                {
                    continue;
                }

                if let Some(y_out) = next_edges.get(&y) {
                    let to_move = y_out
                        .iter()
                        .filter(|(y_z_idx, y_z)| {
                            are_bindings_safe(x, y, &y_z.rhs) // (4)
                                && (!x.has_bindings() || !y_z.label.is_player_assignment()) // (5)
                                && !to_remove.contains(y_z_idx) // For safety, this edge could have been removed
                        })
                        .map(|(y_z_idx, _)| *y_z_idx)
                        .collect::<Vec<_>>();
                    if to_move.len() == y_out.len() {
                        // (7)
                        to_remove.push(*x_y_idx);
                    }
                    if !to_move.is_empty() {
                        used_nodes.insert(y);
                        used_nodes.insert(x);
                        to_add.push(((*x).clone(), to_move));
                    }
                }
            }
        }
        (to_remove, to_add)
    }

    /// If x, y: ; exists, then all conditional edges from x to y can be removed.
    /// TODO: Should we not remove edges with tags?
    fn skip_all_obsolete(&self) -> Vec<usize> {
        let next_edges = self.next_edges_with_idx();
        let mut to_remove = vec![];
        for edges in next_edges.values() {
            for (_, edges) in group_by_rhs(edges) {
                if let Some((skip_idx, _)) = edges.iter().find(|(_, edge)| edge.label.is_skip()) {
                    edges
                        .iter()
                        .filter(|(idx, edge)| idx != skip_idx && !edge.label.is_assignment())
                        .for_each(|(idx, _)| to_remove.push(*idx));
                }
            }
        }

        to_remove
    }

    // TODO: Extract to a separate AST transform.
    fn make_bindings_unique(&mut self) {
        let edges_using_binding_map = get_edges_using_binding_map(&self.edges);

        let mut are_bindings_unique = true;
        for index in 0..self.edges.len() {
            for (binding, _) in self.edges[index].bindings() {
                let edges_using_binding = edges_using_binding_map.get(&(index, binding.clone()));
                for index in 0..self.edges.len() {
                    if edges_using_binding.is_none_or(|edges| !edges.contains(&index))
                        && self.edges[index].get_binding(binding).is_some()
                    {
                        are_bindings_unique = false;
                    }
                }
            }
        }

        if are_bindings_unique {
            return;
        }

        let mut counts: BTreeMap<_, usize> = BTreeMap::new();
        let mut mapped = BTreeSet::new();

        // Iterate over indexes to eliminate multiple ownership.
        let edges = &mut self.edges;
        for x in 0..edges.len() {
            for (binding, type_) in edges[x].clone().bindings() {
                if mapped.contains(&(x, binding.clone())) {
                    continue;
                }

                let type_id = type_.as_type_reference().cloned();
                let index = counts.entry(type_id.clone()).or_default();
                *index += 1;

                // TODO: All bindings equal to `fresh` should be renamed before for safety.
                let fresh: Arc<str> = Arc::from(type_id.map_or_else(
                    || format!("bind_{index}"),
                    |id| format!("bind_{id}_{index}"),
                ));

                let mapping = BTreeMap::from([(binding.clone(), (fresh.clone(), type_.clone()))]);
                for y in edges_using_binding_map
                    .get(&(x, binding.clone()))
                    .map(|x| x.iter())
                    .into_iter()
                    .flatten()
                    .cloned()
                {
                    mapped.insert((y, fresh.clone()));
                    edges[y] = Arc::from(edges[y].rename_variables(&mapping));
                }
            }
        }
    }
}

fn group_by_rhs<'a>(
    edges: &'a BTreeSet<(usize, &'a Arc<Edge<Id>>)>,
) -> BTreeMap<&'a Node<Id>, SetWithIdx<'a, &'a Arc<Edge<Id>>>> {
    let mut grouped: BTreeMap<_, SetWithIdx<_>> = BTreeMap::new();
    for (idx, edge) in edges {
        grouped.entry(&edge.rhs).or_default().insert((*idx, edge));
    }
    grouped
}

fn are_bindings_safe(a: &Node<Arc<str>>, b: &Node<Arc<str>>, c: &Node<Arc<str>>) -> bool {
    if b.has_bindings() {
        b.bindings().eq(a.bindings()) && b.bindings().eq(c.bindings())
    } else {
        !a.has_bindings() || !c.has_bindings()
    }
}

fn get_edges_using_binding_map(
    edges: &[Arc<Edge<Arc<str>>>],
) -> BTreeMap<(usize, Arc<str>), Rc<BTreeSet<usize>>> {
    let game = Game {
        edges: edges.to_vec(),
        ..Game::default()
    };

    let next_edges_idx = game.next_edges_idx();
    let prev_edges_idx = game.prev_edges_idx();

    let mut edges_using_binding_map = BTreeMap::new();
    for (index, edge) in edges.iter().enumerate() {
        for (binding, _) in edge.bindings() {
            if !edges_using_binding_map.contains_key(&(index, binding.clone())) {
                let mut queue = vec![index];
                let mut nodes_using_binding = BTreeSet::from([index]);
                let mut edges_using_binding = BTreeSet::from([index]);

                while let Some(index) = queue.pop() {
                    let Edge { lhs, rhs, .. } = edges[index].as_ref();
                    if rhs.has_binding(binding) {
                        for lhs_index in next_edges_idx.get(&rhs).into_iter().flatten().copied() {
                            if nodes_using_binding.insert(lhs_index) {
                                edges_using_binding.insert(lhs_index);
                                queue.push(lhs_index);
                            }
                        }
                    }

                    if lhs.has_binding(binding) {
                        for rhs_index in prev_edges_idx.get(&lhs).into_iter().flatten().copied() {
                            if nodes_using_binding.insert(rhs_index) {
                                edges_using_binding.insert(rhs_index);
                                queue.push(rhs_index);
                            }
                        }
                    }
                }

                let edges_using_binding = Rc::new(edges_using_binding);
                for index in edges_using_binding.iter() {
                    edges_using_binding_map
                        .entry((*index, binding.clone()))
                        .or_insert_with(|| edges_using_binding.clone());
                }
            }
        }
    }

    edges_using_binding_map
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
            begin, cond: ;
            cond, true: 1 == 1;
            cond, cond: 1 != 1;
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
            cond(x: X), cond(x: X): 1 != 1;
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
            begin, loop(bind_X_1: X)(bind_Y_1: Y): ;
            loop(bind_X_1: X)(bind_Y_1: Y), cond(bind_X_1: X)(bind_Z_1: Z): ;
            cond(bind_X_1: X)(bind_Z_1: Z), true(bind_X_1: X)(bind_Y_2: Y): 1 == 1;
            cond(bind_X_1: X)(bind_Z_1: Z), false(bind_X_1: X)(bind_Z_1: Z): 1 != 1;
            false(bind_X_1: X)(bind_Z_1: Z), loop(bind_X_1: X)(bind_Y_1: Y): ;
            true(bind_X_1: X)(bind_Y_2: Y), end: player = keeper;
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
            begin, b: ;
            b, c(bind_T_1: T): T(bind_T_1) != T(a);
            c(bind_T_1: T), d: v = T(bind_T_1);
            d, end: ;
            d, g(bind_T_2: T): T(bind_T_2) != T(a);
            g(bind_T_2: T), h: v = T(bind_T_2);
            h, b: ;
            h, end: ;
        "
    );

    test_transform!(
        compact_skip_edges,
        random5,
        "
            begin, a: 1 == 1;
            begin, b: ;
            b, a: 1 == 1;
        ",
        "
            begin, a: 1 == 1;
            begin, a: 1 == 1; // This should be removed in other transforms.
        "
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
            a, e: 1 == 1; // This should be removed in other transforms.
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
        sequence_of_binds,
        "
            begin, x1(position: Position): ;
            x1(position: Position), y: ;
            y, x2(position: Position): ;
            x2(position: Position), end: ;
        ",
        "
            begin, x1(bind_Position_1: Position): ;
            x1(bind_Position_1: Position), y: ;
            y, x2(bind_Position_2: Position): ;
            x2(bind_Position_2: Position), end: ;
        "
    );

    test_transform!(
        compact_skip_edges,
        canonical_form,
        "type T = { 0, 1 }; var t: T = 0; a, b(x: T): x == t; c, d(x: T): x == t;",
        "type T = { 0, 1 }; var t: T = 0; a, b(bind_T_1: T): T(bind_T_1) == t; c, d(bind_T_2: T): T(bind_T_2) == t;"
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
        skip_and_assign,
        "a, b: ;
        a, b: x = 1;
        b, c: 2 == 2;"
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

    test_transform!(
        compact_skip_edges,
        amazons_naive_loops_hrg,
        "
            begin, 1: position = direction[position];
            1, 2: position != null;
            2, 3: board[position] == empty;
            3, 4: ;
            3, 5: position = direction[position];
            5, 6: position != null;
            6, 7: board[position] == empty;
            7, 8: ;
            7, 9: position = direction[position];
            9, 10: position != null;
            10, 11: board[position] == empty;
            11, 12: ;
            11, 13: position = direction[position];
            13, 14: position != null;
            14, 15: board[position] == empty;
            15, 16: ;
            15, 17: position = direction[position];
            17, 18: position != null;
            18, 19: board[position] == empty;
            19, 20: ;
            19, 21: position = direction[position];
            21, 22: position != null;
            22, 23: board[position] == empty;
            23, 24: ;
            23, 25: position = direction[position];
            25, 26: position != null;
            26, 27: board[position] == empty;
            27, 28: ;
            27, 29: position = direction[position];
            29, 30: position != null;
            30, 31: board[position] == empty;
            31, 32: ;
            31, 33: position = direction[position];
            33, 34: position != null;
            34, 32: board[position] == empty;
            32, 28: ;
            28, 24: ;
            24, 20: ;
            20, 16: ;
            16, 12: ;
            12, 8: ;
            8, 4: ;
            4, end: ;
        ",
        "
            begin, 1: position = direction[position];
            1, 2: position != null;
            2, 3: board[position] == empty;
            3, end: ;
            3, 5: position = direction[position];
            5, 6: position != null;
            6, 7: board[position] == empty;
            7, end: ;
            7, 9: position = direction[position];
            9, 10: position != null;
            10, 11: board[position] == empty;
            11, end: ;
            11, 13: position = direction[position];
            13, 14: position != null;
            14, 15: board[position] == empty;
            15, end: ;
            15, 17: position = direction[position];
            17, 18: position != null;
            18, 19: board[position] == empty;
            19, end: ;
            19, 21: position = direction[position];
            21, 22: position != null;
            22, 23: board[position] == empty;
            23, end: ;
            23, 25: position = direction[position];
            25, 26: position != null;
            26, 27: board[position] == empty;
            27, end: ;
            27, 29: position = direction[position];
            29, 30: position != null;
            30, 31: board[position] == empty;
            31, end: ;
            31, 33: position = direction[position];
            33, 34: position != null;
            34, end: board[position] == empty;
        "
    );

    test_transform!(
        compact_skip_edges,
        tictactoe_hrg,
        "
            type Piece = { empty, x, o };
            type Player = { x, o };
            type Position = { p__0_0, p__0_1, p__0_2, p__1_0, p__1_1, p__1_2, p__2_0, p__2_1, p__2_2 };
            type Score = { 50, 0, 100 };
            type turn_return = { turn_call_1, turn_call_2 };
            type Bool = { 0, 1 };
            type Goals = Player -> Score;
            type Visibility = Player -> Bool;
            type PlayerOrSystem = { x, o, keeper, random };
            const next_d1: Position -> Position = { :p__1_1, p__0_1: p__0_1, p__0_2: p__0_2, p__1_0: p__1_0, p__1_1: p__2_2, p__1_2: p__1_2, p__2_0: p__2_0, p__2_1: p__2_1, p__2_2: p__0_0 };
            const next_d2: Position -> Position = { :p__0_0, p__0_1: p__0_1, p__0_2: p__1_1, p__1_0: p__1_0, p__1_1: p__2_0, p__1_2: p__1_2, p__2_0: p__0_2, p__2_1: p__2_1, p__2_2: p__2_2 };
            const next_h: Position -> Position = { :p__0_1, p__0_1: p__0_2, p__0_2: p__0_0, p__1_0: p__1_1, p__1_1: p__1_2, p__1_2: p__1_0, p__2_0: p__2_1, p__2_1: p__2_2, p__2_2: p__2_0 };
            const next_v: Position -> Position = { :p__1_0, p__0_1: p__1_1, p__0_2: p__1_2, p__1_0: p__2_0, p__1_1: p__2_1, p__1_2: p__2_2, p__2_0: p__0_0, p__2_1: p__0_1, p__2_2: p__0_2 };
            const op: Player -> Player = { :o, o: x };
            var board: Position -> Piece = { :empty };
            var turn_return: turn_return = turn_call_1;
            var me: Player = x;
            var position: Position = p__0_0;
            var goals: Goals = { :50 };
            var player: PlayerOrSystem = keeper;
            var visible: Visibility = { :1 };
            begin, rules_begin: ;
            rules_begin, turn_call_1: ;
            turn_call_1, rules_2: turn_return = turn_call_1;
            rules_2, rules_3: me = x;
            rules_3, turn_begin: ;
            turn_begin, turn_1: player = me;
            turn_1, turn_2(p: Position): ;
            turn_2(p: Position), turn_4(p: Position): board[p] == empty;
            turn_4(p: Position), turn_5(p: Position): board[p] = me;
            turn_5(p: Position), turn_6(p: Position): position = p;
            turn_6(p: Position), turn_7(p: Position): $ p;
            turn_7(p: Position), turn_3(p: Position): ;
            turn_3(p: Position), turn_8: ;
            turn_8, turn_9: player = keeper;
            win_call_1, turn_12_1: position = position;
            turn_12_1, win_begin: ;
            win_begin, win_2: position != next_d1[position];
            win_2, win_3: board[position] == board[next_d1[position]];
            win_3, win_4: board[position] == board[next_d1[next_d1[position]]];
            win_4, win_1: ;
            win_begin, win_5: position != next_d2[position];
            win_5, win_6: board[position] == board[next_d2[position]];
            win_6, win_7: board[position] == board[next_d2[next_d2[position]]];
            win_7, win_1: ;
            win_begin, win_8: board[position] == board[next_h[position]];
            win_8, win_9: board[position] == board[next_h[next_h[position]]];
            win_9, win_1: ;
            win_begin, win_10: board[position] == board[next_v[position]];
            win_10, win_11: board[position] == board[next_v[next_v[position]]];
            win_11, win_1: ;
            win_1, win_end: ;
            turn_9, turn_10: ? win_call_1 -> win_end;
            turn_9, turn_11: ! win_call_1 -> win_end;
            turn_10, turn_13: goals[me] = 100;
            turn_13, turn_14: goals[op[me]] = 0;
            turn_14, end: player = keeper;
            findNonempty_call_1, findNonempty_begin: ;
            findNonempty_begin, findNonempty_1(p: Position): ;
            findNonempty_1(p: Position), findNonempty_3(p: Position): board[p] == empty;
            findNonempty_3(p: Position), findNonempty_2(p: Position): ;
            findNonempty_2(p: Position), findNonempty_4: ;
            findNonempty_4, findNonempty_end: ;
            turn_11, turn_16: ? findNonempty_call_1 -> findNonempty_end;
            turn_11, turn_15: ! findNonempty_call_1 -> findNonempty_end;
            turn_15, end: player = keeper;
            turn_16, turn_end: ;
            turn_end, turn_return_1: ;
            turn_return_1, rules_4: turn_return == turn_call_1;
            rules_4, turn_call_2: ;
            turn_call_2, rules_5: turn_return = turn_call_2;
            rules_5, rules_6: me = o;
            rules_6, turn_begin: ;
            turn_end, turn_return_2: ;
            turn_return_2, rules_7: turn_return == turn_call_2;
            rules_7, rules_begin: ;
            rules_1, rules_end: ;
            rules_end, end: ;
        ",
        "
            type Piece = { empty, x, o };
            type Player = { x, o };
            type Position = { p__0_0, p__0_1, p__0_2, p__1_0, p__1_1, p__1_2, p__2_0, p__2_1, p__2_2 };
            type Score = { 50, 0, 100 };
            type turn_return = { turn_call_1, turn_call_2 };
            type Bool = { 0, 1 };
            type Goals = Player -> Score;
            type Visibility = Player -> Bool;
            type PlayerOrSystem = { x, o, keeper, random };
            const next_d1: Position -> Position = { :p__1_1, p__0_1: p__0_1, p__0_2: p__0_2, p__1_0: p__1_0, p__1_1: p__2_2, p__1_2: p__1_2, p__2_0: p__2_0, p__2_1: p__2_1, p__2_2: p__0_0 };
            const next_d2: Position -> Position = { :p__0_0, p__0_1: p__0_1, p__0_2: p__1_1, p__1_0: p__1_0, p__1_1: p__2_0, p__1_2: p__1_2, p__2_0: p__0_2, p__2_1: p__2_1, p__2_2: p__2_2 };
            const next_h: Position -> Position = { :p__0_1, p__0_1: p__0_2, p__0_2: p__0_0, p__1_0: p__1_1, p__1_1: p__1_2, p__1_2: p__1_0, p__2_0: p__2_1, p__2_1: p__2_2, p__2_2: p__2_0 };
            const next_v: Position -> Position = { :p__1_0, p__0_1: p__1_1, p__0_2: p__1_2, p__1_0: p__2_0, p__1_1: p__2_1, p__1_2: p__2_2, p__2_0: p__0_0, p__2_1: p__0_1, p__2_2: p__0_2 };
            const op: Player -> Player = { :o, o: x };
            var board: Position -> Piece = { :empty };
            var turn_return: turn_return = turn_call_1;
            var me: Player = x;
            var position: Position = p__0_0;
            var goals: Goals = { :50 };
            var player: PlayerOrSystem = keeper;
            var visible: Visibility = { :1 };
            begin, rules_begin: ;
            rules_begin, rules_2: turn_return = turn_call_1;
            rules_2, turn_begin: me = x;
            turn_begin, turn_1: player = me;
            turn_1, turn_2(bind_Position_1: Position): ;
            turn_2(bind_Position_1: Position), turn_4(bind_Position_1: Position): board[Position(bind_Position_1)] == empty;
            turn_4(bind_Position_1: Position), turn_5(bind_Position_1: Position): board[Position(bind_Position_1)] = me;
            turn_5(bind_Position_1: Position), turn_6(bind_Position_1: Position): position = Position(bind_Position_1);
            turn_6(bind_Position_1: Position), turn_3(bind_Position_1: Position): $ bind_Position_1;
            turn_3(bind_Position_1: Position), turn_8: ;
            turn_8, turn_9: player = keeper;
            win_call_1, win_begin: position = position;
            win_begin, win_2: position != next_d1[position];
            win_2, win_3: board[position] == board[next_d1[position]];
            win_3, win_end: board[position] == board[next_d1[next_d1[position]]];
            win_begin, win_5: position != next_d2[position];
            win_5, win_6: board[position] == board[next_d2[position]];
            win_6, win_end: board[position] == board[next_d2[next_d2[position]]];
            win_begin, win_8: board[position] == board[next_h[position]];
            win_8, win_end: board[position] == board[next_h[next_h[position]]];
            win_begin, win_10: board[position] == board[next_v[position]];
            win_10, win_end: board[position] == board[next_v[next_v[position]]];
            turn_9, turn_10: ? win_call_1 -> win_end;
            turn_9, turn_11: ! win_call_1 -> win_end;
            turn_10, turn_13: goals[me] = 100;
            turn_13, turn_14: goals[op[me]] = 0;
            turn_14, end: player = keeper;
            findNonempty_call_1, findNonempty_1(bind_Position_2: Position): ;
            findNonempty_1(bind_Position_2: Position), findNonempty_2(bind_Position_2: Position): board[Position(bind_Position_2)] == empty;
            findNonempty_2(bind_Position_2: Position), findNonempty_end: ;
            turn_11, turn_end: ? findNonempty_call_1 -> findNonempty_end;
            turn_11, turn_15: ! findNonempty_call_1 -> findNonempty_end;
            turn_15, end: player = keeper;
            turn_end, turn_call_2: turn_return == turn_call_1;
            turn_call_2, rules_5: turn_return = turn_call_2;
            rules_5, turn_begin: me = o;
            turn_end, rules_begin: turn_return == turn_call_2;
            rules_1, end: ;
        " // This last edge will be removed by prune_unreachable_nodes.
    );

    test_transform!(
        compact_skip_edges,
        tictactoe_hrg_loop,
        "
            win_call_1, win_begin: ;
            win_begin, win_2: position != next_d1[position];
            win_2, win_3: board[position] == board[next_d1[position]];
            win_3, win_end: board[position] == board[__gen_next_d1_next_d1[position]];
            win_begin, win_5: position != next_d2[position];
            win_5, win_6: board[position] == board[next_d2[position]];
            win_6, win_end: board[position] == board[__gen_next_d2_next_d2[position]];
            win_begin, win_8: board[position] == board[next_h[position]];
            win_8, win_end: board[position] == board[__gen_next_h_next_h[position]];
            win_begin, win_10: board[position] == board[next_v[position]];
            win_10, win_end: board[position] == board[__gen_next_v_next_v[position]];
        ",
        "
            win_call_1, win_2: position != next_d1[position];
            win_2, win_3: board[position] == board[next_d1[position]];
            win_3, win_end: board[position] == board[__gen_next_d1_next_d1[position]];
            win_call_1, win_5: position != next_d2[position];
            win_5, win_6: board[position] == board[next_d2[position]];
            win_6, win_end: board[position] == board[__gen_next_d2_next_d2[position]];
            win_call_1, win_8: board[position] == board[next_h[position]];
            win_8, win_end: board[position] == board[__gen_next_h_next_h[position]];
            win_call_1, win_10: board[position] == board[next_v[position]];
            win_10, win_end: board[position] == board[__gen_next_v_next_v[position]];
        "
    );
}
