use crate::ast::{Error, Game, Label, Pragma, Span};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn calculate_simple_apply(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges = self.next_edges();
        let nodes: BTreeSet<_> = self
            .edges
            .iter()
            .filter_map(|edge| {
                if edge.label.is_player_assignment() || edge.label.is_tag() {
                    Some(&edge.rhs)
                } else if let Label::Reachability { lhs, .. } = &edge.label {
                    Some(lhs)
                } else if edge.lhs.is_begin() {
                    Some(&edge.lhs)
                } else {
                    None
                }
            })
            .cloned()
            .collect();

        let mut pragmas = vec![];
        for node in nodes {
            let mut paths_to_edges: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
            let mut paths_to_players: BTreeMap<_, _> = BTreeMap::new();
            let mut paths_to_tags: BTreeMap<_, BTreeSet<(_, _, _)>> = BTreeMap::new();
            let mut queue = vec![(node.clone(), vec![], vec![])];
            while let Some((lhs, path, assignments)) = queue.pop() {
                let path_to_edge = paths_to_edges.entry(lhs.clone()).or_default();

                // Add only if it's not saturated yet.
                if path_to_edge.len() <= 1 {
                    path_to_edge.insert(assignments.clone());
                }

                for edge in next_edges.get(&lhs).into_iter().flatten() {
                    let mut path = path.clone();
                    path.push(edge.rhs.clone());

                    let mut assignments = assignments.clone();
                    match &edge.label {
                        Label::Assignment { lhs, rhs } => {
                            assignments.push(edge.label.clone());
                            if lhs.uncast().is_player_reference() {
                                paths_to_players.entry(rhs.clone()).or_insert(path);
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

            if paths_to_players.len() <= 1 {
                let is_exhaustive = paths_to_edges
                    .values()
                    .all(|assignments| assignments.len() == 1);

                if paths_to_players.len() == 1 {
                    let path = paths_to_players.pop_first().unwrap().1;
                    pragmas.push((node.clone(), vec![], path, true, is_exhaustive));
                }

                for (tag, mut paths) in paths_to_tags {
                    if paths.len() == 1 {
                        let path = paths.pop_first().unwrap().1;
                        pragmas.push((node.clone(), vec![tag], path, false, is_exhaustive));
                    }
                }
            }
        }

        // Merge all pairs of
        //   @simpleApply x ...xtags : ...xs;
        //   @simpleApply y ...ytags : ...ys x;
        // Into
        //   @simpleApply y ...ytags ...xtags : ...ys x ...xs;
        // If there's exactly one `@simpleApply x : ...;` and there's no
        // `player` assignment merged on the resulting path (except the end).
        for index_x in (0..pragmas.len()).rev() {
            let (prev, next) = pragmas.split_at_mut(index_x);
            let ((x, xtags, xs, xplayer, _), next) = next.split_first_mut().unwrap();
            if prev
                .iter()
                .chain(next.iter())
                .any(|(node, _, _, _, _)| node == x)
            {
                continue;
            }

            let mut any_matched = false;
            for (_, ytags, ys, yplayer, _) in prev.iter_mut().chain(next) {
                if ys.last() == Some(x) && !*yplayer {
                    ytags.extend_from_slice(xtags);
                    ys.extend_from_slice(xs);
                    *yplayer |= *xplayer;
                    any_matched = true;
                }
            }

            if any_matched {
                pragmas.swap_remove(index_x);
            }
        }

        for (node, tags, nodes, _, is_exhaustive) in pragmas {
            let pragma = if is_exhaustive {
                Pragma::SimpleApplyExhaustive {
                    span: Span::none(),
                    node,
                    tags,
                    nodes,
                }
            } else {
                Pragma::SimpleApply {
                    span: Span::none(),
                    node,
                    tags,
                    nodes,
                }
            };

            if let Err(index) = self.pragmas.binary_search(&pragma) {
                self.pragmas.insert(index, pragma);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_simple_apply,
        complex_1,
        include_str!("../../../../../examples/simpleApplyTest1.rg"),
        adds "
            @simpleApplyExhaustive doneB dummytag : extraB preend;
            @simpleApplyExhaustive moveA 0 : tagA0 doneA moveB;
            @simpleApplyExhaustive moveA 1 : tagA1 doneA moveB;
            @simpleApplyExhaustive preend : end;
        "
    );

    test_transform!(
        calculate_simple_apply,
        complex_2,
        include_str!("../../../../../examples/simpleApplyTest2.rg"),
        adds "
            @simpleApplyExhaustive moveA 0 : tagA0 doneA moveB;
            @simpleApplyExhaustive moveA 1 : tagA1 doneA moveB;
            @simpleApplyExhaustive moveB 0 dummytag : tagB0same tagB0 doneB extraB preend;
            @simpleApplyExhaustive moveB 1 dummytag : tagB1same tagB1 doneB extraB preend;
            @simpleApplyExhaustive preend : end;
        "
    );

    test_transform!(
        calculate_simple_apply,
        complex_3,
        include_str!("../../../../../examples/simpleApplyTest3.rg"),
        adds "
            @simpleApplyExhaustive moveA 0 : tagA0 doneA moveB;
            @simpleApplyExhaustive moveA 1 : tagA1 doneA moveB;
            @simpleApply moveB 1 dummytag : tagB1same tagB1 doneB extraB preend;
            @simpleApplyExhaustive preend : end;
        "
    );
}
