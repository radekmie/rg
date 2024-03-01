use crate::ast::{Edge, Error, Game, Pragma};
use crate::position::Span;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn calculate_tag_indexes(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges: BTreeMap<_, BTreeSet<_>> =
            self.edges
                .iter()
                .fold(BTreeMap::new(), |mut next_edges, edge| {
                    next_edges.entry(&edge.lhs).or_default().insert(edge);
                    next_edges
                });

        let mut tag_indexes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        for Edge { label, rhs, .. } in &self.edges {
            if label.is_player_assignment() {
                let mut seen = BTreeSet::new();
                let mut queue = vec![(rhs, 0)];
                while let Some((lhs, index)) = queue.pop() {
                    let maybe_edges = next_edges.get(&lhs);
                    if seen.insert((lhs, index)) {
                        if let Some(edges) = maybe_edges {
                            for edge in edges {
                                if edge.label.is_tag() {
                                    let indexes = tag_indexes.entry(lhs).or_default();
                                    match indexes.iter().max() {
                                        None => {
                                            indexes.insert(index);
                                        }
                                        Some(max) if *max < index => {
                                            indexes.insert(usize::MAX);
                                            continue;
                                        }
                                        Some(_) => {}
                                    };

                                    queue.push((&edge.rhs, index + 1));
                                } else if !edge.label.is_player_assignment() {
                                    queue.push((&edge.rhs, index));
                                }
                            }
                        }
                    }
                }
            }
        }

        self.pragmas.retain(|pragma| {
            !matches!(pragma, Pragma::TagIndex { .. } | Pragma::TagMaxIndex { .. })
        });

        let tag_indexes_by_index = tag_indexes.into_iter().fold(
            BTreeMap::new(),
            |mut groups: BTreeMap<_, Vec<_>>, (edge_name, indexes)| {
                let maybe_index = match indexes.len() {
                    0 => None,
                    1 => indexes
                        .first()
                        .copied()
                        .filter(|index| *index < usize::MAX)
                        .map(Ok),
                    _ => indexes
                        .into_iter()
                        .max()
                        .filter(|index| *index < usize::MAX)
                        .map(Err),
                };

                if let Some(index) = maybe_index {
                    let edge_names = groups.entry(index).or_default();
                    let index = edge_names.partition_point(|x| x < edge_name);
                    edge_names.insert(index, edge_name.clone());
                }

                groups
            },
        );

        for (index, edge_names) in tag_indexes_by_index {
            let pragma = match index {
                Ok(index) => Pragma::TagIndex {
                    span: Span::none(),
                    edge_names,
                    index,
                },
                Err(index) => Pragma::TagMaxIndex {
                    span: Span::none(),
                    edge_names,
                    index,
                },
            };

            let index = self.pragmas.partition_point(|x| *x < pragma);
            self.pragmas.insert(index, pragma);
        }

        Ok(())
    }
}
