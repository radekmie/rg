use crate::ast::{
    Edge, Error, Expression, Game, Label, Node, Pragma, PragmaAssignment, PragmaTag, Span,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

macro_rules! assign_any_stub {
    () => {
        Arc::from(Expression::new(Id::from("")))
    };
}

type Id = Arc<str>;

impl Game<Id> {
    pub fn calculate_simple_apply(&mut self) -> Result<(), Error<Id>> {
        let mut simple_paths = self.calculate_simple_paths()?;
        SimplePath::merge_all(&mut simple_paths);
        SimplePath::remove_any_stubs(&mut simple_paths);
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
    fn calculate_simple_paths(&self) -> Result<Vec<SimplePath>, Error<Id>> {
        let next_edges = self.next_edges();
        let mut expose_index = 0;
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
                        Label::AssignmentAny { lhs, .. } => {
                            assignments.push(PragmaAssignment {
                                lhs: lhs.clone(),
                                rhs: assign_any_stub!(),
                            });

                            queue.push((edge.rhs.clone(), path, assignments));
                        }
                        Label::Tag { symbol } => {
                            paths_to_tags
                                .entry(PragmaTag::Symbol {
                                    symbol: symbol.clone(),
                                })
                                .or_default()
                                .insert((edge.rhs.clone(), path, assignments));
                        }
                        Label::TagVariable { identifier } => {
                            paths_to_tags
                                .entry(PragmaTag::Variable {
                                    identifier: identifier.clone(),
                                    type_: self.infer(identifier),
                                })
                                .or_default()
                                .insert((edge.rhs.clone(), path, assignments));
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
            for (mut tag, mut paths) in paths_to_tags {
                let (_, mut path, mut assignments) = paths.pop_first().unwrap();

                // If there's exactly one path to a tag, it's trivially simple.
                if paths.is_empty() {
                    if let PragmaTag::Variable { identifier, type_ } = &mut tag {
                        expose_index += 1;
                        *identifier = Id::from(format!("{identifier}_{expose_index}"));
                        let mut unrelated_assign_any_found = false;
                        for assignment in &mut assignments {
                            if assignment.rhs == assign_any_stub!() {
                                unrelated_assign_any_found |= assignment.lhs.infer(self)? != *type_;
                                assignment.rhs = Arc::from(Expression::new_cast(
                                    type_.clone(),
                                    Arc::from(Expression::new(identifier.clone())),
                                ));
                            }
                        }

                        if unrelated_assign_any_found {
                            continue;
                        }
                    };

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

                //   2. All paths have the assignments. There's one exception:
                //      if the last edge is "exposing a variable".
                if path.len() <= 1 {
                    continue;
                }

                let PragmaTag::Variable {
                    mut identifier,
                    type_,
                } = tag
                else {
                    continue;
                };

                //   3. Remove all trailing assignments to the exposed variable.
                let exposed_variable = Arc::from(Expression::new(identifier.clone()));
                macro_rules! expose_variable {
                    ($assignments:expr) => {
                        while $assignments
                            .last()
                            .is_some_and(|x| x.lhs.uncast() == exposed_variable.as_ref())
                        {
                            $assignments.pop();
                        }
                    };
                }

                expose_variable!(assignments);
                paths = paths
                    .into_iter()
                    .map(|(node, nodes, mut assignments)| {
                        expose_variable!(assignments);
                        (node, nodes, assignments)
                    })
                    .collect();

                //   4. All paths must have the same assignments.
                if paths.iter().any(|x| x.2 != assignments) {
                    continue;
                }

                expose_index += 1;
                identifier = Id::from(format!("{identifier}_{expose_index}"));
                assignments.push(PragmaAssignment {
                    lhs: exposed_variable,
                    rhs: Arc::from(Expression::new_cast(
                        type_.clone(),
                        Arc::from(Expression::new(identifier.clone())),
                    )),
                });

                simple_paths_to_tags.push(SimplePath {
                    assignments,
                    is_direct_from_player,
                    is_exhaustive: true,
                    node: node.clone(),
                    path: vec![path.remove(0), path.pop().unwrap()],
                    tags: vec![PragmaTag::Variable { identifier, type_ }],
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

        Ok(simple_paths)
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
            } else if edge.label.is_tag() || edge.label.is_tag_variable() {
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
    tags: Vec<PragmaTag<Id>>,
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

    fn remove_any_stubs(simple_paths: &mut Vec<Self>) {
        simple_paths.retain(|simple_path| {
            simple_path
                .assignments
                .iter()
                .all(|assignment| assignment.rhs != assign_any_stub!())
        });
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

                let mut tagsets: Vec<_> = indexes
                    .iter()
                    .map(|index| &simple_paths[*index].tags)
                    .collect();

                let mut ambiguous = false;
                while let Some(x) = tagsets.pop() {
                    ambiguous |= tagsets.iter().any(|y| {
                        let mut ambiguous_prefix = false;
                        for index in 0..(x.len().min(y.len())) {
                            if x[index].is_variable() && y[index].is_variable() {
                                ambiguous_prefix = true;
                                break;
                            }

                            if x[index] != y[index] {
                                break;
                            }
                        }

                        if !ambiguous_prefix {
                            return false;
                        }

                        let mut ambiguous_suffix = false;
                        for index in (0..(x.len().min(y.len()))).rev() {
                            if x[index].is_variable() && y[index].is_variable() {
                                ambiguous_suffix = true;
                                break;
                            }

                            if x[index] != y[index] {
                                break;
                            }
                        }

                        ambiguous_suffix
                    });
                }

                if !ambiguous {
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
        assign_any,
        "
            type Position = {0, 1, 2};
            var position: Position = 0;
            const next: Position -> Position = {:0};
            begin, x: position = Position(*);
            x, end: player = keeper;
        "
    );

    test_transform!(
        calculate_simple_apply,
        assign_any_assignment_chain,
        "
            type Position = {0, 1, 2};
            var position: Position = 0;
            const next: Position -> Position = {:0};
            begin, x: position = Position(*);
            x, y: position = next[position];
            y, end: player = keeper;
        "
    );

    test_transform!(
        calculate_simple_apply,
        assign_any_assignment_expose_chain_1,
        "
            type Position = {0, 1, 2};
            var position: Position = 0;
            const next: Position -> Position = {:0};
            begin, x: position = Position(*);
            x, y: position = next[position];
            y, z: $$ position;
            z, end: player = keeper;
        ",
        adds "@simpleApplyExhaustive begin end [position_1: Position] position = Position(position_1), position = next[position], player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        assign_any_assignment_expose_chain_2,
        "
            type Position = {0, 1, 2};
            var position: Position = 0;
            const next: Position -> Position = {:0};
            begin, x: position = next[position];
            x, y: position = Position(*);
            y, z: $$ position;
            z, end: player = keeper;
        ",
        adds "@simpleApplyExhaustive begin end [position_1: Position] position = next[position], position = Position(position_1), player = keeper;"
    );

    test_transform!(
        calculate_simple_apply,
        multiple_paths_with_expose,
        "
            type Position = {0, 1, 2};
            var position: Position = 0;
            begin, x: position = north[position];
            begin, y: position = south[position];
            x, show: ;
            y, show: ;
            show, shown: $$ position;
            shown, end: player = keeper;
        ",
        adds "@simpleApplyExhaustive begin end [position_1: Position] position = Position(position_1), player = keeper;"
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

    test_transform!(
        calculate_simple_apply,
        chess_hrg_king_standard_move,
        "
            type AllDir = { up, down, left, right, upleft, upright, downleft, downright };
            type Position = { null, V__0_0, V__0_1, V__0_2, V__0_3, V__0_4, V__0_5, V__0_6, V__0_7, V__1_0, V__1_1, V__1_2, V__1_3, V__1_4, V__1_5, V__1_6, V__1_7, V__2_0, V__2_1, V__2_2, V__2_3, V__2_4, V__2_5, V__2_6, V__2_7, V__3_0, V__3_1, V__3_2, V__3_3, V__3_4, V__3_5, V__3_6, V__3_7, V__4_0, V__4_1, V__4_2, V__4_3, V__4_4, V__4_5, V__4_6, V__4_7, V__5_0, V__5_1, V__5_2, V__5_3, V__5_4, V__5_5, V__5_6, V__5_7, V__6_0, V__6_1, V__6_2, V__6_3, V__6_4, V__6_5, V__6_6, V__6_7, V__7_0, V__7_1, V__7_2, V__7_3, V__7_4, V__7_5, V__7_6, V__7_7 };
            const leftdir: Position -> Position = { :null, V__1_0: V__0_0, V__1_1: V__0_1, V__1_2: V__0_2, V__1_3: V__0_3, V__1_4: V__0_4, V__1_5: V__0_5, V__1_6: V__0_6, V__1_7: V__0_7, V__2_0: V__1_0, V__2_1: V__1_1, V__2_2: V__1_2, V__2_3: V__1_3, V__2_4: V__1_4, V__2_5: V__1_5, V__2_6: V__1_6, V__2_7: V__1_7, V__3_0: V__2_0, V__3_1: V__2_1, V__3_2: V__2_2, V__3_3: V__2_3, V__3_4: V__2_4, V__3_5: V__2_5, V__3_6: V__2_6, V__3_7: V__2_7, V__4_0: V__3_0, V__4_1: V__3_1, V__4_2: V__3_2, V__4_3: V__3_3, V__4_4: V__3_4, V__4_5: V__3_5, V__4_6: V__3_6, V__4_7: V__3_7, V__5_0: V__4_0, V__5_1: V__4_1, V__5_2: V__4_2, V__5_3: V__4_3, V__5_4: V__4_4, V__5_5: V__4_5, V__5_6: V__4_6, V__5_7: V__4_7, V__6_0: V__5_0, V__6_1: V__5_1, V__6_2: V__5_2, V__6_3: V__5_3, V__6_4: V__5_4, V__6_5: V__5_5, V__6_6: V__5_6, V__6_7: V__5_7, V__7_0: V__6_0, V__7_1: V__6_1, V__7_2: V__6_2, V__7_3: V__6_3, V__7_4: V__6_4, V__7_5: V__6_5, V__7_6: V__6_6, V__7_7: V__6_7 };
            const rightdir: Position -> Position = { :null, V__0_0: V__1_0, V__0_1: V__1_1, V__0_2: V__1_2, V__0_3: V__1_3, V__0_4: V__1_4, V__0_5: V__1_5, V__0_6: V__1_6, V__0_7: V__1_7, V__1_0: V__2_0, V__1_1: V__2_1, V__1_2: V__2_2, V__1_3: V__2_3, V__1_4: V__2_4, V__1_5: V__2_5, V__1_6: V__2_6, V__1_7: V__2_7, V__2_0: V__3_0, V__2_1: V__3_1, V__2_2: V__3_2, V__2_3: V__3_3, V__2_4: V__3_4, V__2_5: V__3_5, V__2_6: V__3_6, V__2_7: V__3_7, V__3_0: V__4_0, V__3_1: V__4_1, V__3_2: V__4_2, V__3_3: V__4_3, V__3_4: V__4_4, V__3_5: V__4_5, V__3_6: V__4_6, V__3_7: V__4_7, V__4_0: V__5_0, V__4_1: V__5_1, V__4_2: V__5_2, V__4_3: V__5_3, V__4_4: V__5_4, V__4_5: V__5_5, V__4_6: V__5_6, V__4_7: V__5_7, V__5_0: V__6_0, V__5_1: V__6_1, V__5_2: V__6_2, V__5_3: V__6_3, V__5_4: V__6_4, V__5_5: V__6_5, V__5_6: V__6_6, V__5_7: V__6_7, V__6_0: V__7_0, V__6_1: V__7_1, V__6_2: V__7_2, V__6_3: V__7_3, V__6_4: V__7_4, V__6_5: V__7_5, V__6_6: V__7_6, V__6_7: V__7_7 };
            const updir: Position -> Position = { :null, V__0_1: V__0_0, V__0_2: V__0_1, V__0_3: V__0_2, V__0_4: V__0_3, V__0_5: V__0_4, V__0_6: V__0_5, V__0_7: V__0_6, V__1_1: V__1_0, V__1_2: V__1_1, V__1_3: V__1_2, V__1_4: V__1_3, V__1_5: V__1_4, V__1_6: V__1_5, V__1_7: V__1_6, V__2_1: V__2_0, V__2_2: V__2_1, V__2_3: V__2_2, V__2_4: V__2_3, V__2_5: V__2_4, V__2_6: V__2_5, V__2_7: V__2_6, V__3_1: V__3_0, V__3_2: V__3_1, V__3_3: V__3_2, V__3_4: V__3_3, V__3_5: V__3_4, V__3_6: V__3_5, V__3_7: V__3_6, V__4_1: V__4_0, V__4_2: V__4_1, V__4_3: V__4_2, V__4_4: V__4_3, V__4_5: V__4_4, V__4_6: V__4_5, V__4_7: V__4_6, V__5_1: V__5_0, V__5_2: V__5_1, V__5_3: V__5_2, V__5_4: V__5_3, V__5_5: V__5_4, V__5_6: V__5_5, V__5_7: V__5_6, V__6_1: V__6_0, V__6_2: V__6_1, V__6_3: V__6_2, V__6_4: V__6_3, V__6_5: V__6_4, V__6_6: V__6_5, V__6_7: V__6_6, V__7_1: V__7_0, V__7_2: V__7_1, V__7_3: V__7_2, V__7_4: V__7_3, V__7_5: V__7_4, V__7_6: V__7_5, V__7_7: V__7_6 };
            const downdir: Position -> Position = { :null, V__0_0: V__0_1, V__0_1: V__0_2, V__0_2: V__0_3, V__0_3: V__0_4, V__0_4: V__0_5, V__0_5: V__0_6, V__0_6: V__0_7, V__1_0: V__1_1, V__1_1: V__1_2, V__1_2: V__1_3, V__1_3: V__1_4, V__1_4: V__1_5, V__1_5: V__1_6, V__1_6: V__1_7, V__2_0: V__2_1, V__2_1: V__2_2, V__2_2: V__2_3, V__2_3: V__2_4, V__2_4: V__2_5, V__2_5: V__2_6, V__2_6: V__2_7, V__3_0: V__3_1, V__3_1: V__3_2, V__3_2: V__3_3, V__3_3: V__3_4, V__3_4: V__3_5, V__3_5: V__3_6, V__3_6: V__3_7, V__4_0: V__4_1, V__4_1: V__4_2, V__4_2: V__4_3, V__4_3: V__4_4, V__4_4: V__4_5, V__4_5: V__4_6, V__4_6: V__4_7, V__5_0: V__5_1, V__5_1: V__5_2, V__5_2: V__5_3, V__5_3: V__5_4, V__5_4: V__5_5, V__5_5: V__5_6, V__5_6: V__5_7, V__6_0: V__6_1, V__6_1: V__6_2, V__6_2: V__6_3, V__6_3: V__6_4, V__6_4: V__6_5, V__6_5: V__6_6, V__6_6: V__6_7, V__7_0: V__7_1, V__7_1: V__7_2, V__7_2: V__7_3, V__7_3: V__7_4, V__7_4: V__7_5, V__7_5: V__7_6, V__7_6: V__7_7 };
            const upleftdir: Position -> Position = { :null, V__1_1: V__0_0, V__1_2: V__0_1, V__1_3: V__0_2, V__1_4: V__0_3, V__1_5: V__0_4, V__1_6: V__0_5, V__1_7: V__0_6, V__2_1: V__1_0, V__2_2: V__1_1, V__2_3: V__1_2, V__2_4: V__1_3, V__2_5: V__1_4, V__2_6: V__1_5, V__2_7: V__1_6, V__3_1: V__2_0, V__3_2: V__2_1, V__3_3: V__2_2, V__3_4: V__2_3, V__3_5: V__2_4, V__3_6: V__2_5, V__3_7: V__2_6, V__4_1: V__3_0, V__4_2: V__3_1, V__4_3: V__3_2, V__4_4: V__3_3, V__4_5: V__3_4, V__4_6: V__3_5, V__4_7: V__3_6, V__5_1: V__4_0, V__5_2: V__4_1, V__5_3: V__4_2, V__5_4: V__4_3, V__5_5: V__4_4, V__5_6: V__4_5, V__5_7: V__4_6, V__6_1: V__5_0, V__6_2: V__5_1, V__6_3: V__5_2, V__6_4: V__5_3, V__6_5: V__5_4, V__6_6: V__5_5, V__6_7: V__5_6, V__7_1: V__6_0, V__7_2: V__6_1, V__7_3: V__6_2, V__7_4: V__6_3, V__7_5: V__6_4, V__7_6: V__6_5, V__7_7: V__6_6 };
            const uprightdir: Position -> Position = { :null, V__0_1: V__1_0, V__0_2: V__1_1, V__0_3: V__1_2, V__0_4: V__1_3, V__0_5: V__1_4, V__0_6: V__1_5, V__0_7: V__1_6, V__1_1: V__2_0, V__1_2: V__2_1, V__1_3: V__2_2, V__1_4: V__2_3, V__1_5: V__2_4, V__1_6: V__2_5, V__1_7: V__2_6, V__2_1: V__3_0, V__2_2: V__3_1, V__2_3: V__3_2, V__2_4: V__3_3, V__2_5: V__3_4, V__2_6: V__3_5, V__2_7: V__3_6, V__3_1: V__4_0, V__3_2: V__4_1, V__3_3: V__4_2, V__3_4: V__4_3, V__3_5: V__4_4, V__3_6: V__4_5, V__3_7: V__4_6, V__4_1: V__5_0, V__4_2: V__5_1, V__4_3: V__5_2, V__4_4: V__5_3, V__4_5: V__5_4, V__4_6: V__5_5, V__4_7: V__5_6, V__5_1: V__6_0, V__5_2: V__6_1, V__5_3: V__6_2, V__5_4: V__6_3, V__5_5: V__6_4, V__5_6: V__6_5, V__5_7: V__6_6, V__6_1: V__7_0, V__6_2: V__7_1, V__6_3: V__7_2, V__6_4: V__7_3, V__6_5: V__7_4, V__6_6: V__7_5, V__6_7: V__7_6 };
            const downleftdir: Position -> Position = { :null, V__1_0: V__0_1, V__1_1: V__0_2, V__1_2: V__0_3, V__1_3: V__0_4, V__1_4: V__0_5, V__1_5: V__0_6, V__1_6: V__0_7, V__2_0: V__1_1, V__2_1: V__1_2, V__2_2: V__1_3, V__2_3: V__1_4, V__2_4: V__1_5, V__2_5: V__1_6, V__2_6: V__1_7, V__3_0: V__2_1, V__3_1: V__2_2, V__3_2: V__2_3, V__3_3: V__2_4, V__3_4: V__2_5, V__3_5: V__2_6, V__3_6: V__2_7, V__4_0: V__3_1, V__4_1: V__3_2, V__4_2: V__3_3, V__4_3: V__3_4, V__4_4: V__3_5, V__4_5: V__3_6, V__4_6: V__3_7, V__5_0: V__4_1, V__5_1: V__4_2, V__5_2: V__4_3, V__5_3: V__4_4, V__5_4: V__4_5, V__5_5: V__4_6, V__5_6: V__4_7, V__6_0: V__5_1, V__6_1: V__5_2, V__6_2: V__5_3, V__6_3: V__5_4, V__6_4: V__5_5, V__6_5: V__5_6, V__6_6: V__5_7, V__7_0: V__6_1, V__7_1: V__6_2, V__7_2: V__6_3, V__7_3: V__6_4, V__7_4: V__6_5, V__7_5: V__6_6, V__7_6: V__6_7 };
            const downrightdir: Position -> Position = { :null, V__0_0: V__1_1, V__0_1: V__1_2, V__0_2: V__1_3, V__0_3: V__1_4, V__0_4: V__1_5, V__0_5: V__1_6, V__0_6: V__1_7, V__1_0: V__2_1, V__1_1: V__2_2, V__1_2: V__2_3, V__1_3: V__2_4, V__1_4: V__2_5, V__1_5: V__2_6, V__1_6: V__2_7, V__2_0: V__3_1, V__2_1: V__3_2, V__2_2: V__3_3, V__2_3: V__3_4, V__2_4: V__3_5, V__2_5: V__3_6, V__2_6: V__3_7, V__3_0: V__4_1, V__3_1: V__4_2, V__3_2: V__4_3, V__3_3: V__4_4, V__3_4: V__4_5, V__3_5: V__4_6, V__3_6: V__4_7, V__4_0: V__5_1, V__4_1: V__5_2, V__4_2: V__5_3, V__4_3: V__5_4, V__4_4: V__5_5, V__4_5: V__5_6, V__4_6: V__5_7, V__5_0: V__6_1, V__5_1: V__6_2, V__5_2: V__6_3, V__5_3: V__6_4, V__5_4: V__6_5, V__5_5: V__6_6, V__5_6: V__6_7, V__6_0: V__7_1, V__6_1: V__7_2, V__6_2: V__7_3, V__6_3: V__7_4, V__6_4: V__7_5, V__6_5: V__7_6, V__6_6: V__7_7 };
            const all_direction: AllDir -> Position -> Position = { :uprightdir, up: updir, down: downdir, left: leftdir, right: rightdir, upleft: upleftdir, downleft: downleftdir, downright: downrightdir };
            var dir: AllDir = up;
            var pos: Position = null;
            1, 2: $ K;
            2, 3: dir = AllDir(*);
            3, 4: pos = all_direction[dir][pos];
            4, 5: $$ pos;
        "
    );
}
