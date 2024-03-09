use crate::ast::{EdgeLabel, Error, Game, Pragma};
use crate::position::Span;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn calculate_uniques(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges: BTreeMap<_, BTreeSet<_>> =
            self.edges
                .iter()
                .fold(BTreeMap::new(), |mut next_edges, edge| {
                    next_edges.entry(&edge.lhs).or_default().insert(edge);
                    next_edges
                });

        let mut unique_edge_names: BTreeSet<_> = self
            .edges
            .iter()
            .flat_map(|edge| [&edge.lhs, &edge.rhs])
            .cloned()
            .collect();

        let edge_names: BTreeSet<_> = self
            .edges
            .iter()
            .filter_map(|edge| {
                if edge.label.is_player_assignment() || edge.label.is_tag() {
                    Some(&edge.rhs)
                } else if let EdgeLabel::Reachability { lhs, .. } = &edge.label {
                    Some(lhs)
                } else if edge.lhs.is_begin() {
                    Some(&edge.lhs)
                } else {
                    None
                }
            })
            .cloned()
            .collect();

        for edge_name in edge_names {
            let mut seen = BTreeSet::new();
            let mut queue = vec![edge_name];
            while let Some(lhs) = queue.pop() {
                let maybe_edges = next_edges.get(&lhs);
                if seen.insert(lhs) {
                    if let Some(edges) = maybe_edges {
                        for edge in edges {
                            if seen.contains(&edge.rhs) {
                                unique_edge_names.remove(&edge.rhs);
                            } else if !edge.label.is_player_assignment() && !edge.label.is_tag() {
                                queue.push(edge.rhs.clone());
                            }
                        }
                    }
                }
            }
        }

        self.pragmas.retain(|pragma| {
            if let Pragma::Unique { edge_names, .. } = pragma {
                unique_edge_names.extend(edge_names.iter().cloned());
                false
            } else {
                true
            }
        });

        let pragma = Pragma::Unique {
            span: Span::none(),
            edge_names: unique_edge_names.into_iter().collect(),
        };

        let index = self.pragmas.partition_point(|x| *x < pragma);
        self.pragmas.insert(index, pragma);

        Ok(())
    }
}
