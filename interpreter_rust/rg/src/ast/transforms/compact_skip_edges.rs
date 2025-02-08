use crate::ast::{Edge, Error, Game, Node, SetWithIdx};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type RemovedAndMoved<T> = (Vec<usize>, Vec<T>);

impl Game<Id> {
    pub fn compact_skip_edges(&mut self) -> Result<(), Error<Arc<str>>> {
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
    ///   5. If all edges incoming to y satisfy 6. then remove y -> z
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
                        // For safety, this edge could have been removed
                        .filter(|(x_y_idx, _)| !to_remove.contains(x_y_idx))
                        .map(|(x_y_idx, _)| *x_y_idx)
                        .collect::<Vec<_>>();
                    if to_move.len() == y_in.len() {
                        // (5)
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
    ///   5. If all edges outgoing from y satisfy 6. then remove x -> y
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
                        // For safety, this edge could have been removed
                        .filter(|(y_z_idx, _)| !to_remove.contains(y_z_idx))
                        .map(|(y_z_idx, _)| *y_z_idx)
                        .collect::<Vec<_>>();
                    if to_move.len() == y_out.len() {
                        // (5)
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
        "begin, b: player = x; b, c: ; c, end: ;",
        "begin, end: player = x;"
    );

    test_transform!(
        compact_skip_edges,
        player_assignment_suffix,
        "begin, b: ; b, c: ; c, end: player = x;",
        "begin, end: player = x;"
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
        complex_loop,
        "
            type T = { a, b };
            var v: T = a;
            begin, a: ;
            a, b: ;
            b, c: T(t) != T(a);
            c, d: v = t;
            d, e: ;
            d, f: ;
            e, g: T(t) != T(a);
            f, end: ;
            g, h: v = t;
            h, a: ;
            h, end: ;
        ",
        "
            type T = { a, b };
            var v: T = a;
            begin, b: ;
            b, c: T(t) != T(a);
            c, d: v = t;
            d, end: ;
            d, g: T(t) != T(a);
            g, h: v = t;
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
            begin, x1: 1 == 1;
            x1, x3: p != null;
            x3, x4: position = p;
            x4, x2: 1 == 1;
            x2, end: 1 == 1;
        ",
        "
            begin, x1: 1 == 1;
            x1, x3: p != null;
            x3, x4: position = p;
            x4, x2: 1 == 1;
            x2, end: 1 == 1;
        "
    );

    test_transform!(
        compact_skip_edges,
        linear_unordered,
        "
            begin, x1: 1 == 1;
            x2, end: 1 == 1;
            x1, x4: p != null;
            x4, x5: position = p;
            x5, x2: 1 == 1;
        ",
        "
            begin, x1: 1 == 1;
            x2, end: 1 == 1;
            x1, x4: p != null;
            x4, x5: position = p;
            x5, x2: 1 == 1;
        "
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
            turn_1, turn_2: ;
            turn_2, turn_4: board[p] == empty;
            turn_4, turn_5: board[p] = me;
            turn_5, turn_6: position = p;
            turn_6, turn_7: $ p;
            turn_7, turn_3: ;
            turn_3, turn_8: ;
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
            findNonempty_begin, findNonempty_1: ;
            findNonempty_1, findNonempty_3: board[p] == empty;
            findNonempty_3, findNonempty_2: ;
            findNonempty_2, findNonempty_4: ;
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
            turn_begin, turn_2: player = me;
            turn_2, turn_4: board[p] == empty;
            turn_4, turn_5: board[p] = me;
            turn_5, turn_6: position = p;
            turn_6, turn_3: $ p;
            turn_3, turn_9: player = keeper;
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
            findNonempty_call_1, findNonempty_end: board[p] == empty;
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
