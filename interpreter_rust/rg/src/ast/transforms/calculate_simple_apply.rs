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
            let mut paths_to_players: BTreeMap<_, BTreeSet<(_, _, _)>> = BTreeMap::new();
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
                                        paths_to_players.entry(rhs.clone()).or_default().insert((
                                            edge.rhs.clone(),
                                            path.clone(),
                                            assignments,
                                        ));
                                    } else {
                                        queue.push((edge.rhs.clone(), path.clone(), assignments));
                                    }
                                }
                                Label::Tag { symbol } => {
                                    paths_to_tags.entry(symbol.clone()).or_default().insert((
                                        edge.rhs.clone(),
                                        path.clone(),
                                        assignments,
                                    ));
                                }
                                _ => {
                                    queue.push((edge.rhs.clone(), path.clone(), assignments));
                                }
                            }
                        }
                    }
                }
            }

            if paths_to_players.len() <= 1 {
                if paths_to_players.len() == 1 {
                    pragmas.push(Pragma::SimpleApply {
                        span: Span::none(),
                        node: node.clone(),
                        tags: vec![],
                        nodes: paths_to_players
                            .pop_first()
                            .unwrap()
                            .1
                            .pop_first()
                            .unwrap()
                            .1,
                    });
                }

                for (tag, mut paths) in paths_to_tags.into_iter() {
                    if paths.len() == 1 {
                        pragmas.push(Pragma::SimpleApply {
                            span: Span::none(),
                            node: node.clone(),
                            tags: vec![tag],
                            nodes: paths.pop_first().unwrap().1,
                        });
                    }
                }
            }
        }

        for pragma in pragmas {
            if let Err(index) = self.pragmas.binary_search(&pragma) {
                self.pragmas.insert(index, pragma);
            }
        }

        Ok(())
    }
}
