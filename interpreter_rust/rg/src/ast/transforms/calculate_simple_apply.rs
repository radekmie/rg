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
                if edge.lhs.is_begin() {
                    Some(&edge.lhs)
                } else if edge.label.is_player_assignment() || edge.label.is_tag() {
                    Some(&edge.rhs)
                } else if let Label::Reachability { lhs, .. } = &edge.label {
                    Some(lhs)
                } else {
                    None
                }
            })
            .cloned()
            .collect();

        let mut pragmas = vec![];
        'outer: for node in nodes {
            let mut paths_to_edges = BTreeMap::new();
            let mut paths_to_players: BTreeMap<_, _> = BTreeMap::new();
            let mut paths_to_tags: BTreeMap<_, BTreeSet<(_, _, _)>> = BTreeMap::new();
            let mut queue = vec![(node.clone(), vec![], vec![])];
            while let Some((lhs, path, assignments)) = queue.pop() {
                let maybe_edges = next_edges.get(&lhs);

                let mut seen = None;
                paths_to_edges
                    .entry(lhs.clone())
                    .and_modify(|existing| seen = Some(*existing == assignments))
                    .or_insert_with(|| assignments.clone());

                match seen {
                    Some(true) => {}
                    Some(false) => continue 'outer,
                    None => {
                        let Some(edges) = maybe_edges else { continue };

                        for edge in edges {
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
                }
            }

            if paths_to_players.len() <= 1 {
                if paths_to_players.len() == 1 {
                    pragmas.push((node.clone(), paths_to_players.pop_first().unwrap().1, None));
                }

                for (tag, mut paths) in paths_to_tags.into_iter() {
                    if paths.len() == 1 {
                        pragmas.push((node.clone(), paths.pop_first().unwrap().1, Some(tag)));
                    }
                }
            }
        }

        // Merge all pairs of
        //   @simpleApply x : ...xs;
        //   @simpleApply y ...ts : ...ys x;
        // Into
        //   @simpleApply y ...ts : ...ys x ...xs;
        // If there's exactly one `@simpleApply x : ...;`.
        for index_x in (1..pragmas.len()).rev() {
            let (prev, next) = pragmas.split_at_mut(index_x);
            let ((x, xs, tag), next) = next.split_first_mut().unwrap();
            if tag.is_some() || prev.iter().chain(next.iter()).any(|(node, _, _)| node == x) {
                continue;
            }

            let mut any_matched = false;
            for (_, ys, _) in prev.iter_mut().chain(next) {
                if ys.last() == Some(x) {
                    ys.extend_from_slice(xs);
                    any_matched = true;
                }
            }

            if any_matched {
                pragmas.swap_remove(index_x);
            }
        }

        for (node, nodes, tag) in pragmas {
            let pragma = Pragma::SimpleApply {
                span: Span::none(),
                node,
                tags: tag.into_iter().collect(),
                nodes,
            };
            if let Err(index) = self.pragmas.binary_search(&pragma) {
                self.pragmas.insert(index, pragma);
            }
        }

        Ok(())
    }
}
