use crate::ast::{Edge, Error, Game, Label, Node, Pragma, PragmaAssignment, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn calculate_simple_apply(&mut self) -> Result<(), Error<Id>> {
        let mut simple_paths = self.calculate_simple_paths();
        SimplePath::merge_all(&mut simple_paths);
        SimplePath::remove_ambiguous(&mut simple_paths);
        SimplePath::propagate_exhaustiveness(&mut simple_paths);

        for simple_path in simple_paths {
            let pragma = simple_path.into_pragma();
            if let Err(index) = self.pragmas.binary_search(&pragma) {
                self.pragmas.insert(index, pragma);
            }
        }

        Ok(())
    }

    /// A list of "short" simple paths, i.e., with at most one tag.
    fn calculate_simple_paths(&self) -> Vec<SimplePath> {
        let next_edges = self.next_edges();
        let mut simple_paths = vec![];
        'outer: for (is_direct_from_player, node) in self.calculate_simple_paths_candidates() {
            let mut path_to_player = None;
            let mut paths_to_edges: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
            let mut paths_to_tags: BTreeMap<_, BTreeSet<(_, _, _)>> = BTreeMap::new();
            let mut queue = vec![(node.clone(), vec![], vec![])];
            while let Some((lhs, path, assignments)) = queue.pop() {
                let path_to_edge = paths_to_edges.entry(lhs.clone()).or_default();

                // Add only if it's not saturated yet.
                if path_to_edge.len() <= 1 {
                    path_to_edge.insert(assignments.clone());
                }

                for edge in next_edges.get(&lhs).into_iter().flatten() {
                    if path.iter().any(|x: &Arc<Edge<_>>| x.rhs == edge.rhs) {
                        continue;
                    }

                    let mut path = path.clone();
                    path.push((*edge).clone());

                    let mut assignments = assignments.clone();
                    match &edge.label {
                        Label::Assignment { lhs, rhs, .. } => {
                            assignments.push(PragmaAssignment {
                                lhs: lhs.clone(),
                                rhs: rhs.clone(),
                            });

                            if lhs.uncast().is_player_reference() {
                                if path_to_player.replace((path, assignments)).is_some() {
                                    // This will not be a simple path.
                                    continue 'outer;
                                }
                            } else {
                                queue.push((edge.rhs.clone(), path, assignments));
                            }
                        }
                        Label::Tag { symbol } => {
                            paths_to_tags.entry(symbol.clone()).or_default().insert((
                                edge.rhs.clone(),
                                path,
                                assignments,
                            ));
                        }
                        _ => {
                            queue.push((edge.rhs.clone(), path, assignments));
                        }
                    }
                }
            }

            let is_exhaustive = paths_to_edges
                .values()
                .all(|assignments_set| assignments_set.len() == 1);

            if let Some((path, assignments)) = path_to_player {
                simple_paths.push(SimplePath {
                    assignments,
                    is_direct_from_player,
                    is_exhaustive,
                    node: node.clone(),
                    path,
                    tags: vec![],
                    to_player: true,
                });
            }

            let tags_count = paths_to_tags.len();
            let mut simple_paths_to_tags = vec![];
            for (tag, mut paths) in paths_to_tags {
                let (_, mut path, assignments) = paths.pop_first().unwrap();

                // If there's exactly one path to a tag, it's trivially simple.
                if paths.is_empty() {
                    simple_paths_to_tags.push(SimplePath {
                        assignments,
                        is_direct_from_player,
                        is_exhaustive,
                        node: node.clone(),
                        path,
                        tags: vec![tag],
                        to_player: false,
                    });
                    continue;
                }

                // If there are more paths, it can be simple if:
                //   1. All paths end in the same node.
                if paths.iter().any(|x| x.1.last() != path.last()) {
                    continue;
                }

                if paths.iter().any(|x| x.2 != assignments) {
                    continue;
                }

                simple_paths_to_tags.push(SimplePath {
                    assignments,
                    is_direct_from_player,
                    is_exhaustive: true,
                    node: node.clone(),
                    path: vec![path.remove(0), path.pop().unwrap()],
                    tags: vec![tag],
                    to_player: false,
                });
            }

            // If not all tags were reachable, nothing is exhaustive.
            if simple_paths_to_tags.len() != tags_count {
                for simple_path in &mut simple_paths_to_tags {
                    simple_path.is_exhaustive = false;
                }
            }

            simple_paths.extend(simple_paths_to_tags);
        }

        simple_paths
    }

    /// A set of `Node`s that can start a simple path.
    fn calculate_simple_paths_candidates(&self) -> BTreeSet<(bool, Node<Id>)> {
        let mut candidates = BTreeSet::new();
        macro_rules! add_candidate {
            ($is_direct_from_player:expr, $node:expr) => {
                let mut key = ($is_direct_from_player, $node.clone());
                if $is_direct_from_player {
                    key.0 = false;
                    candidates.remove(&key);
                    key.0 = true;
                }
                candidates.insert(key);
            };
        }

        for edge in &self.edges {
            if edge.lhs.is_begin() {
                add_candidate!(true, &edge.lhs);
            }

            if edge.label.is_player_assignment() {
                add_candidate!(true, &edge.rhs);
            } else if edge.label.is_tag() {
                add_candidate!(false, &edge.rhs);
            } else if let Label::Reachability { lhs, .. } = &edge.label {
                add_candidate!(false, lhs);
            }
        }

        candidates
    }
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct SimplePath {
    assignments: Vec<PragmaAssignment<Id>>,
    is_direct_from_player: bool,
    is_exhaustive: bool,
    node: Node<Id>,
    path: Vec<Arc<Edge<Id>>>,
    tags: Vec<Id>,
    to_player: bool,
}

impl SimplePath {
    fn into_pragma(mut self) -> Pragma<Id> {
        if self.is_exhaustive {
            Pragma::SimpleApplyExhaustive {
                span: Span::none(),
                lhs: self.node,
                rhs: self.path.pop().unwrap().rhs.clone(),
                tags: self.tags,
                assignments: self.assignments,
            }
        } else {
            Pragma::SimpleApply {
                span: Span::none(),
                lhs: self.node,
                rhs: self.path.pop().unwrap().rhs.clone(),
                tags: self.tags,
                assignments: self.assignments,
            }
        }
    }

    /// It's allowed to merge
    ///   @simpleApply x1 x2 [...xtags] ...xassignments;
    ///   @simpleApply y1 y2 [...ytags] ...yassignments;
    /// Into
    ///   @simpleApply y1 x2 [...ytags ...xtags] ...yassignments ...xassignments;
    /// If there's no `player` assignment in the middle of the resulting path and
    /// if the merge will not get rid of any exhaustiveness.
    fn merge(&self, other: &Self) -> Option<Self> {
        if !self.to_player
            && (!self.is_exhaustive || other.is_exhaustive)
            && self.path.last().map(|node| &node.rhs) == Some(&other.node)
        {
            let mut clone = self.clone();
            clone.assignments.extend_from_slice(&other.assignments);
            clone.tags.extend_from_slice(&other.tags);
            clone.path.extend_from_slice(&other.path);
            clone.to_player |= other.to_player;
            Some(clone)
        } else {
            None
        }
    }

    /// Merge all paths pair-wise.
    fn merge_all(simple_paths: &mut Vec<Self>) {
        for index_x in (0..simple_paths.len()).rev() {
            let (prev, next) = simple_paths.split_at_mut(index_x);
            let (x, next) = next.split_first_mut().unwrap();

            // Merging is allowed only if there's one extension possiblity.
            if prev.iter().chain(next.iter()).any(|y| y.node == x.node) {
                continue;
            }

            let mut any_matched = false;
            for y in prev.iter_mut().chain(next) {
                if let Some(y_extended) = y.merge(x) {
                    *y = y_extended;
                    any_matched = true;
                }
            }

            if any_matched {
                simple_paths.swap_remove(index_x);
            }
        }
    }

    /// To be exhaustive, it has to follow another simple path _and_ be directly
    /// following from the player assignment.
    fn propagate_exhaustiveness(simple_paths: &mut [Self]) {
        loop {
            let mut changed = false;
            for index in 0..simple_paths.len() {
                if !simple_paths[index].is_direct_from_player {
                    simple_paths[index].is_direct_from_player = simple_paths.iter().any(|y| {
                        y.is_direct_from_player
                            && y.path.last().unwrap().rhs == simple_paths[index].node
                    });

                    if simple_paths[index].is_direct_from_player {
                        changed = true;
                    }
                }
            }

            if !changed {
                break;
            }
        }

        for simple_path in simple_paths {
            simple_path.is_exhaustive &= simple_path.is_direct_from_player;
        }
    }

    /// Remove multiple simple paths that start in one node and are ambiguous,
    /// i.e., have tag bindings at the same tag position following the same tag
    /// prefix and having the same suffix.
    // TODO: Add prefix tests (chess.hrg).
    // TODO: Add suffix tests (englishDraughts.hrg).
    // TODO: Add continuations tests (englishDraughts.hrg).
    fn remove_ambiguous(simple_paths: &mut Vec<Self>) {
        loop {
            let mut any_continuations_merged = false;
            for index in (0..simple_paths.len()).rev() {
                if index >= simple_paths.len() {
                    continue;
                }

                let x = &simple_paths[index];
                let indexes: Vec<_> = simple_paths
                    .iter()
                    .enumerate()
                    .filter(|(_, y)| y.node == x.node)
                    .map(|(index, _)| index)
                    .collect();
                if indexes.len() == 1 {
                    continue;
                }

                // We have to remove these, as they're not
                let all_continuations: Result<Vec<_>, _> = indexes
                    .iter()
                    .map(|index| {
                        let x = &simple_paths[*index];
                        let indexes: Vec<_> = simple_paths
                            .iter()
                            .enumerate()
                            .filter(|(_, y)| y.node == x.path.last().unwrap().rhs)
                            .map(|(index, _)| index)
                            .collect();

                        // No available continuations.
                        if indexes.is_empty() {
                            return Err(());
                        }

                        let merged: Vec<_> = indexes
                            .iter()
                            .filter_map(|index| x.merge(&simple_paths[*index]))
                            .collect();

                        // Not all continuations are allowed.
                        if merged.len() < indexes.len() {
                            return Err(());
                        }

                        Ok((*index, merged))
                    })
                    .collect();

                match all_continuations {
                    Err(()) => {
                        for index in indexes.into_iter().rev() {
                            simple_paths.swap_remove(index);
                        }
                    }
                    Ok(all_continuations) => {
                        let (indexes, merged): (Vec<_>, Vec<_>) =
                            all_continuations.into_iter().unzip();

                        for index in indexes.into_iter().rev() {
                            simple_paths.swap_remove(index);
                        }

                        simple_paths.extend(merged.into_iter().flatten());
                        any_continuations_merged = true;
                    }
                }
            }

            if !any_continuations_merged {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_simple_apply,
        no_binds_in_node,
        "
            begin, rules_begin: ;
            rules_begin, move_begin: player = me;
            move_begin, move_1(p: Position): ;
            move_1(p: Position), move_4(p: Position): p != null;
            move_4(p: Position), move_3(p: Position): board[p] == piece[me];
            move_3(p: Position), move_5(p: Position): board[p] = empty;
            move_5(p: Position), move_6(p: Position): position = direction[me][p];
            move_6(p: Position), move_7(p: Position): $ p;
            move_7(p: Position), move_2(p: Position): ;
            move_2(p: Position), move_8: ;
            move_8, move_10: ;
            move_10, move_12: position = left[direction[me][position]];
            move_12, move_11: $ L;
            move_10, move_14: position = right[direction[me][position]];
            move_14, move_11: $ R;
            move_11, move_9: ;
            move_8, move_18: position = left[left[position]];
            move_18, move_16: $ LL;
            move_8, move_21: position = right[right[position]];
            move_21, move_16: $ RR;
            move_16, move_9: ;
            move_9, move_23: position != null;
            move_23, move_end: board[position] != piece[me];
            move_end, turn_5: board[position] = piece[me];
            turn_5, turn_6: player = keeper;
        ",
        adds "
            @simpleApplyExhaustive begin move_begin [] player = me;
            @simpleApplyExhaustive move_begin move_7(p: Position) [p: Position] board[p] = empty, position = direction[me][p];
            @simpleApplyExhaustive move_7(p: Position) turn_6 [L] position = left[direction[me][position]], board[position] = piece[me], player = keeper;
            @simpleApplyExhaustive move_7(p: Position) turn_6 [LL] position = left[left[position]], board[position] = piece[me], player = keeper;
            @simpleApplyExhaustive move_7(p: Position) turn_6 [R] position = right[direction[me][position]], board[position] = piece[me], player = keeper;
            @simpleApplyExhaustive move_7(p: Position) turn_6 [RR] position = right[right[position]], board[position] = piece[me], player = keeper;
        "
    );

    test_transform!(
        calculate_simple_apply,
        no_free_binds,
        "
            begin, 4: ;
            4, 9(bind_2: Coord): Coord(bind_2) != Coord(null);
            9(bind_2: Coord), 11: coord = bind_2;
            11, 14: board[coord] == w;
            14, 15: board[coord] = e;
            15, 16(bind_3: Coord): bind_3 == coord;
            16(bind_3: Coord), 17: $ bind_3;
            17, 12: $ index_2;
            12, end: ;
        ",
        adds "@simpleApply 17 12 [index_2];"
    );

    test_transform!(
        calculate_simple_apply,
        with_loop,
        "
            begin, 4: ;
            4, 9(bind_2: Coord): Coord(bind_2) != Coord(null);
            9(bind_2: Coord), 11: coord = bind_2;
            11, 14: board[coord] == w;
            14, 15: board[coord] = e;
            15, 11: ;
            15, 16(bind_3: Coord): bind_3 == coord;
            16(bind_3: Coord), 17: $ bind_3;
            17, 12: $ index_2;
            12, end: ;
        ",
        adds "@simpleApply 17 12 [index_2];"
    );

    test_transform!(
        calculate_simple_apply,
        adjust_exhaustiveness,
        "
            begin, x1: ;
            begin, y1: ;
            x1, x2: ;
            x2, x3(_: Bool): ;
            x3(_: Bool), x4: ;
            x4, x5: $ x;
            y1, y2: $ y;
            x5, end: player = keeper;
            y2, end: player = keeper;
        ",
        adds "@simpleApply begin end [y] player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        same_player,
        "
            begin, x: ;
            begin, y: ;
            x, end: player = keeper;
            y, end: player = keeper;
        "
    );

    test_transform!(
        calculate_simple_apply,
        multiple_paths_with_expose,
        "
            begin, x: position = north[position];
            begin, y: position = south[position];
            x, show(p: Position): position == p;
            y, show(p: Position): position == p;
            show(p: Position), shown: $ p;
            shown, end: player = keeper;
        ",
        adds "@simpleApplyExhaustive begin end [p: Position] position = Position(p), player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        multiple_paths_with_expose_and_exit,
        "
            begin, x: position = north[position];
            begin, y: position = south[position];
            x, shown: ;
            x, show(p: Position): position == p;
            y, show(p: Position): position == p;
            show(p: Position), shown: $ p;
            shown, end: player = keeper;
        ",
        adds "
            @simpleApply begin end [] position = north[position], player = keeper;
            @simpleApply shown end [] player = keeper;
        "
    );

    test_transform!(
        calculate_simple_apply,
        multiple_paths_with_expose_and_a_different_assignment,
        "
            begin, x1: position = north[position];
            begin, y: position = south[position];
            x1, x2: other_variable = 1;
            x2, show(p: Position): position == p;
            y, show(p: Position): position == p;
            show(p: Position), shown: $ p;
            shown, end: player = keeper;
        ",
        adds "@simpleApply shown end [] player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        multiple_paths_with_expose_reversed,
        "
            begin, x: position = north[position];
            begin, y: position = south[position];
            x, show(p: Position): position == p;
            y, show(p: Position): p == position;
            show(p: Position), shown: $ p;
            shown, end: player = keeper;
        ",
        adds "@simpleApplyExhaustive begin end [p: Position] position = Position(p), player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        multiple_paths_with_expose_and_multiple_tags,
        "
            begin, x: position = north[position];
            begin, y: position = south[position];
            x, show(p: Position): position == p;
            y, show(p: Position): position == p;
            show(p: Position), shown: $ p;
            shown, end: player = keeper;
            begin, a(bool: Bool): ;
            a(bool: Bool), b: x = bool;
            b, c: ;
            c, d(p: Position): position == p;
            d(p: Position), other: $ p;
            other, end: player = keeper;
        ",
        adds "
            @simpleApply other end [] player = keeper;
            @simpleApply shown end [] player = keeper;
        "
    );

    test_transform!(
        calculate_simple_apply,
        multiple_paths_with_expose_and_different_continuations,
        "
            begin, x: position = north[position];
            begin, y: position = south[position];
            x, show(p: Position): position == p;
            y, show(p: Position): position == p;
            show(p: Position), shown: $ p;

            shown, x1: ;
            shown, y1: ;
            x1, x2: ;
            x2, x3(bool: Bool): ;
            x3(bool: Bool), x4: b = bool;
            x4, x5: $ x;
            y1, y2: $ y;
            x5, end: player = keeper;
            y2, end: player = keeper;
        ",
        adds "
            @simpleApply shown end [y] player = keeper;
            @simpleApply x5 end [] player = keeper;
            @simpleApplyExhaustive begin shown [p: Position] position = Position(p);
        "
    );

    test_transform!(
        calculate_simple_apply,
        contained_binds,
        "
            begin, 203: player = keeper;
            203, 208(bind_Coord_12: Coord): Coord(bind_Coord_12) != Coord(null);
            208(bind_Coord_12: Coord), 207: coord = Coord(bind_Coord_12);
            207, 211: board[coord] == b;
            211, 213: board[coord] = e;
            213, 214(bind_Coord_16: Coord): Coord(bind_Coord_16) == coord;
            214(bind_Coord_16: Coord), 215: $ bind_Coord_16;
            215, 210: $ index_9;
            210, end: player = keeper;
        ",
        adds "
            @simpleApply 215 end [index_9] player = keeper;
            @simpleApplyExhaustive begin 203 [] player = keeper;
        "
    );

    test_transform!(
        calculate_simple_apply,
        too_many_binds_1,
        "
            begin, 1(bind_Coord_1: Coord): coord = bind_Coord_1;
            1(bind_Coord_1: Coord), 2: ;
            2, 3: board[coord] = a;
            3, 4(bind_Coord_2: Coord): coord = bind_Coord_2;
            4(bind_Coord_2: Coord), 5: ;
            5, 6: board[coord] = b;
            6, 7: coord = left[coord];
            7, 8: board[coord] = c;
            8, 9(bind_Coord_3: Coord): bind_Coord_3 == coord;
            9(bind_Coord_3: Coord), 10: $ bind_Coord_3;
            10, end: player = keeper;
        ",
        adds "@simpleApply 10 end [] player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        too_many_binds_2,
        "
            begin, 1(bind_Coord_1: Coord): coord = bind_Coord_1;
            1(bind_Coord_1: Coord), 2: ;
            2, 3: ;
            3, 4(bind_Coord_2: Coord): coord = bind_Coord_2;
            4(bind_Coord_2: Coord), 5: ;
            5, 6: ;
            6, 7: coord = left[coord];
            7, 8: board[coord] = c;
            8, 9(bind_Coord_3: Coord): bind_Coord_3 == coord;
            9(bind_Coord_3: Coord), 10: $ bind_Coord_3;
            10, end: player = keeper;
        ",
        adds "@simpleApply 10 end [] player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        complex_1,
        include_str!("../../../../../games/rg/simpleApplyTest1.rg"),
        adds "
            @simpleApply doneB preend [dummytag] player = PlayerOrSystem(keeper);
            @simpleApplyExhaustive begin moveA [] player = PlayerOrSystem(A);
            @simpleApplyExhaustive moveA moveB [0] key = 0, player = PlayerOrSystem(B);
            @simpleApplyExhaustive moveA moveB [1] key = 1, player = PlayerOrSystem(B);
            @simpleApplyExhaustive preend end [] player = PlayerOrSystem(keeper);
        "
    );

    test_transform!(
        calculate_simple_apply,
        complex_2,
        include_str!("../../../../../games/rg/simpleApplyTest2.rg"),
        adds "
            @simpleApplyExhaustive begin moveA [] player = PlayerOrSystem(A);
            @simpleApplyExhaustive moveA moveB [0] key = 0, player = PlayerOrSystem(B);
            @simpleApplyExhaustive moveA moveB [1] key = 1, player = PlayerOrSystem(B);
            @simpleApplyExhaustive moveB preend [0, dummytag] goals[B] = Score(100), player = PlayerOrSystem(keeper);
            @simpleApplyExhaustive moveB preend [1, dummytag] goals[B] = Score(100), player = PlayerOrSystem(keeper);
            @simpleApplyExhaustive preend end [] player = PlayerOrSystem(keeper);
        "
    );

    test_transform!(
        calculate_simple_apply,
        complex_3,
        include_str!("../../../../../games/rg/simpleApplyTest3.rg"),
        adds "
            @simpleApply moveB preend [1, dummytag] goals[B] = Score(100), player = PlayerOrSystem(keeper);
            @simpleApplyExhaustive begin moveA [] player = PlayerOrSystem(A);
            @simpleApplyExhaustive moveA moveB [0] key = 0, player = PlayerOrSystem(B);
            @simpleApplyExhaustive moveA moveB [1] key = 1, player = PlayerOrSystem(B);
            @simpleApplyExhaustive preend end [] player = PlayerOrSystem(keeper);
        "
    );
}
