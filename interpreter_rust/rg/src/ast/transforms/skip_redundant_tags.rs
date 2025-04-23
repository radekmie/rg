use crate::ast::{Edge, Error, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type EdgeAndPath<Id> = (Arc<Edge<Id>>, Vec<Arc<Edge<Id>>>);

impl Game<Id> {
    pub fn skip_redundant_tags(&mut self) -> Result<(), Error<Id>> {
        // If the game uses the `visible`, then we leave the tags as they are,
        // just in case. In the future, we could make the analysis smarter.
        let visible = Id::from("visible");
        if self
            .edges
            .iter()
            .all(|edge| !edge.label.has_variable(&visible))
        {
            while let Some(indexes) = self.find_redundant_tags() {
                for index in indexes {
                    Arc::make_mut(&mut self.edges[index]).skip();
                }
            }
        }

        Ok(())
    }

    fn find_redundant_tags(&self) -> Option<Vec<usize>> {
        let next_edges = self.next_edges();
        let prev_edges = self.prev_edges();
        let mut indexes: Option<Vec<_>> = None;
        for (index, edge) in self.edges.iter().enumerate() {
            let is_tag = edge.label.is_tag();
            if is_tag || edge.label.is_tag_variable() {
                let prevs_nexts: Vec<_> = find_prevs(&edge.lhs, &prev_edges, is_tag)
                    .into_iter()
                    .flat_map(|(prev, _)| find_nexts(&prev.rhs, &next_edges, is_tag))
                    .collect();

                // If all successors of all predecessors match this tag...
                if prevs_nexts.iter().all(|x| is_tag_matching(edge, x)) {
                    // ...it's redundant.
                    indexes.get_or_insert_default().push(index);
                    continue;
                }

                // If it's a normal tag...
                if is_tag {
                    let (xs, ys): (Vec<_>, _) =
                        prevs_nexts.into_iter().partition(|(next, _)| edge == next);
                    // ...and there's exactly one path to it...
                    if let [(_, path_x)] = &xs[..] {
                        // ...and all other paths are disjoint...
                        if ys.iter().all(|(_, path_y)| is_disjoint(path_x, path_y)) {
                            // ...it's redundant.
                            indexes.get_or_insert_default().push(index);
                        }
                    }
                }
            }
        }

        indexes
    }
}

fn find_prevs(
    node: &Node<Id>,
    prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    ignore_tag_variables: bool,
) -> Vec<EdgeAndPath<Id>> {
    let mut seen = BTreeSet::new();
    let mut queue = vec![(node, vec![])];
    let mut prevs = vec![];
    while let Some((rhs, path)) = queue.pop() {
        if let Some(edges) = prev_edges.get(&rhs) {
            for edge in edges {
                if edge.label.is_player_assignment()
                    || edge.label.is_tag()
                    || !ignore_tag_variables && edge.label.is_tag_variable()
                {
                    prevs.push(((*edge).clone(), path.clone()));
                } else if seen.insert(&edge.lhs) {
                    let mut path = path.clone();
                    path.push((*edge).clone());
                    queue.push((&edge.lhs, path));
                }
            }
        }
    }

    prevs
}

fn find_nexts(
    node: &Node<Id>,
    next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    ignore_tag_variables: bool,
) -> Vec<EdgeAndPath<Id>> {
    let mut seen = BTreeSet::new();
    let mut queue = vec![(node, vec![])];
    let mut nexts = vec![];
    while let Some((lhs, path)) = queue.pop() {
        if let Some(edges) = next_edges.get(&lhs) {
            for edge in edges {
                if edge.label.is_player_assignment()
                    || edge.label.is_tag()
                    || !ignore_tag_variables && edge.label.is_tag_variable()
                {
                    nexts.push(((*edge).clone(), path.clone()));
                } else if seen.insert(&edge.rhs) {
                    let mut path = path.clone();
                    path.push((*edge).clone());
                    queue.push((&edge.rhs, path));
                }
            }
        }
    }

    nexts
}

fn is_disjoint(xs: &[Arc<Edge<Id>>], ys: &[Arc<Edge<Id>>]) -> bool {
    // TODO: This could use `@disjoint`.
    xs.iter()
        .any(|x| ys.iter().any(|y| x.label.is_negated(&y.label)))
}

fn is_tag_matching(edge: &Arc<Edge<Id>>, edge_and_path: &EdgeAndPath<Id>) -> bool {
    edge.label == edge_and_path.0.label
        && edge.label.as_tag_variable().is_none_or(|identifier| {
            !edge_and_path.1.iter().any(|edge| {
                edge.label
                    .as_var_assignment()
                    .is_some_and(|x| x == identifier)
            })
        })
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        skip_redundant_tags,
        basic,
        "begin, q1: $a; q1, end: player = keeper;",
        "begin, q1: ; q1, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        simple_one,
        "begin, q1: player = keeper; q1, q2: $a; q2, end: player = keeper;",
        "begin, q1: player = keeper; q1, q2: ; q2, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        simple_two,
        "begin, q1: player = keeper; q1, q2: $a; q2, q3: $b; q3, end: player = keeper;",
        "begin, q1: player = keeper; q1, q2: ; q2, q3: ; q3, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        simple_tag_variable,
        "
            begin, q1: player = keeper;
            q1, q2: coord = Coord(*);
            q2, q3: $$ coord;
            q3, end: player = keeper;
        "
    );

    test_transform!(
        skip_redundant_tags,
        simple_tag_variable_before,
        "
            begin, q1: player = keeper;
            q1, q2: $ static;
            q2, q3: coord = Coord(*);
            q3, q4: $$ coord;
            q4, end: player = keeper;
        ",
        "
            begin, q1: player = keeper;
            q1, q2: ;
            q2, q3: coord = Coord(*);
            q3, q4: $$ coord;
            q4, end: player = keeper;
        "
    );

    test_transform!(
        skip_redundant_tags,
        simple_tag_variable_after,
        "
            begin, q1: player = keeper;
            q1, q2: coord = Coord(*);
            q2, q3: $$ coord;
            q3, q4: $ static;
            q4, end: player = keeper;
        ",
        "
            begin, q1: player = keeper;
            q1, q2: coord = Coord(*);
            q2, q3: $$ coord;
            q3, q4: ;
            q4, end: player = keeper;
        "
    );

    test_transform!(
        skip_redundant_tags,
        split_equal,
        "begin, q1: player = keeper; q1, q2: $a; q2, end: player = keeper; q1, q3: $a; q3, end: player = keeper;",
        "begin, q1: player = keeper; q1, q2: ; q2, end: player = keeper; q1, q3: ; q3, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        split_different,
        "begin, q1: player = keeper; q1, q2: $a; q2, end: player = keeper; q1, q3: $b; q3, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        split_common_unique,
        "
            begin, a: player = keeper;
            a, b1: ;
                b1, c1: $$ v;
                c1, d1: $ t1;
                d1, x: ;
            a, b2: ;
                b2, c2: $$ v;
                c2, d2: $ t2;
                d2, x: ;
            a, b3: ;
                b3, c3: $$ v;
                c3, d3: $ t3;
                d3, x: ;
            x, end: player = keeper;
        ",
        "
            begin, a: player = keeper;
            a, b1: ;
                b1, c1: ;
                c1, d1: $ t1;
                d1, x: ;
            a, b2: ;
                b2, c2: ;
                c2, d2: $ t2;
                d2, x: ;
            a, b3: ;
                b3, c3: ;
                c3, d3: $ t3;
                d3, x: ;
            x, end: player = keeper;
        "
    );

    test_transform!(
        skip_redundant_tags,
        split_unique_common,
        "
            begin, a: player = keeper;
            a, b1: ;
                b1, c1: $ t1;
                c1, d1: $$ v;
                d1, x: ;
            a, b2: ;
                b2, c2: $ t2;
                c2, d2: $$ v;
                d2, x: ;
            a, b3: ;
                b3, c3: $ t3;
                c3, d3: $$ v;
                d3, x: ;
            x, end: player = keeper;
        ",
        "
            begin, a: player = keeper;
            a, b1: ;
                b1, c1: $ t1;
                c1, d1: ;
                d1, x: ;
            a, b2: ;
                b2, c2: $ t2;
                c2, d2: ;
                d2, x: ;
            a, b3: ;
                b3, c3: $ t3;
                c3, d3: ;
                d3, x: ;
            x, end: player = keeper;
        "
    );

    test_transform!(
        skip_redundant_tags,
        tictactoe_rbg,
        "
            begin, 2: goals[xplayer] = 50;
            2, 3: $$ coord;
            3, 1: $ index_1;
            1, 5: goals[oplayer] = 50;
            5, 6: $$ coord;
            6, 8: $ index_2;
            8, 12: $$ coord;
            12, 11: $ index_3;
            11, 181: ! 10_17_10 -> 10_17_17;
                181, end: player = keeper;
            11, 188: ? 10_17_10 -> 10_17_17;
                188, 10: player = xplayer;
                10, 14: coord = Coord(*);
                14, 13: coord != Coord(null);
                13, 15: board[coord] == e;
                15, 18: $$ coord;
                18, 17: $ index_4;
                17, 20: player = keeper;
                20, 23: $$ coord;
                23, 22: $ index_5;
                22, 19: board[coord] = x;
                19, 24: ! 26 -> 27;
                    24, 99: $$ coord;
                    99, 98: $ index_9;
                    98, 181: ! 97_104_97 -> 97_104_104;
                    98, 192: ? 97_104_97 -> 97_104_104;
                        192, 97: player = oplayer;
                        97, 101: coord = Coord(*);
                        101, 100: coord != Coord(null);
                        100, 102: board[coord] == e;
                        102, 105: $$ coord;
                        105, 104: $ index_10;
                        104, 107: player = keeper;
                        107, 110: $$ coord;
                        110, 109: $ index_11;
                        109, 106: board[coord] = o;
                        106, 8: ! 113 -> 114;
                        106, 173: ? 113 -> 114;
                            173, 175: goals[oplayer] = 100;
                            175, 176: $$ coord;
                            176, 174: $ index_12;
                            174, 178: goals[xplayer] = 0;
                            178, 179: $$ coord;
                            179, 177: $ index_13;
                            177, 182: $$ coord;
                            182, 181: $ index_14;
                19, 86: ? 26 -> 27;
                    86, 88: goals[xplayer] = 100;
                    88, 89: $$ coord;
                    89, 87: $ index_6;
                    87, 91: goals[oplayer] = 0;
                    91, 92: $$ coord;
                    92, 90: $ index_7;
                    90, 95: $$ coord;
                    95, 181: $ index_8;
        ",
        "
            begin, 2: goals[xplayer] = 50;
            2, 3: ;
            3, 1: ;
            1, 5: goals[oplayer] = 50;
            5, 6: ;
            6, 8: ;
            8, 12: ;
            12, 11: ;
            11, 181: ! 10_17_10 -> 10_17_17;
                181, end: player = keeper;
            11, 188: ? 10_17_10 -> 10_17_17;
                188, 10: player = xplayer;
                10, 14: coord = Coord(*);
                14, 13: coord != Coord(null);
                13, 15: board[coord] == e;
                15, 18: $$ coord;
                18, 17: ;
                17, 20: player = keeper;
                20, 23: ;
                23, 22: ;
                22, 19: board[coord] = x;
                19, 24: ! 26 -> 27;
                    24, 99: ;
                    99, 98: ;
                        98, 181: ! 97_104_97 -> 97_104_104;
                        98, 192: ? 97_104_97 -> 97_104_104;
                            192, 97: player = oplayer;
                            97, 101: coord = Coord(*);
                            101, 100: coord != Coord(null);
                            100, 102: board[coord] == e;
                            102, 105: $$ coord;
                            105, 104: ;
                            104, 107: player = keeper;
                            107, 110: ;
                            110, 109: ;
                            109, 106: board[coord] = o;
                            106, 8: ! 113 -> 114;
                            106, 173: ? 113 -> 114;
                                173, 175: goals[oplayer] = 100;
                                175, 176: ;
                                176, 174: ;
                                174, 178: goals[xplayer] = 0;
                                178, 179: ;
                                179, 177: ;
                                177, 182: ;
                                182, 181: ;
                19, 86: ? 26 -> 27;
                    86, 88: goals[xplayer] = 100;
                    88, 89: ;
                    89, 87: ;
                    87, 91: goals[oplayer] = 0;
                    91, 92: ;
                    92, 90: ;
                    90, 95: ;
                    95, 181: ;
        "
    );
}
