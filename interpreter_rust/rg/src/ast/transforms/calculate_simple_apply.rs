use crate::ast::{EdgeLabel, Error, Game, Pragma};
use crate::position::Span;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn calculate_simple_apply(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges: BTreeMap<_, BTreeSet<_>> =
            self.edges
                .iter()
                .fold(BTreeMap::new(), |mut next_edges, edge| {
                    next_edges.entry(&edge.lhs).or_default().insert(edge);
                    next_edges
                });

        let edge_names: BTreeSet<_> = self
            .edges
            .iter()
            .flat_map(|edge| [&edge.lhs, &edge.rhs])
            .cloned()
            .collect();

        let mut simple_apply_edge_names = BTreeSet::new();
        for edge_name in edge_names {
            let mut seen = BTreeSet::new();
            let mut paths_to_players: BTreeMap<_, BTreeSet<(_, Vec<_>)>> = BTreeMap::new();
            let mut paths_to_tags: BTreeMap<_, BTreeSet<(_, Vec<_>)>> = BTreeMap::new();
            let mut queue = vec![(edge_name.clone(), vec![])];
            while let Some((lhs, assignments)) = queue.pop() {
                let maybe_edges = next_edges.get(&lhs);
                if seen.insert(lhs) {
                    if let Some(edges) = maybe_edges {
                        for edge in edges {
                            let mut assignments = assignments.clone();
                            if let EdgeLabel::Assignment { lhs, rhs } = &edge.label {
                                assignments.push(edge.label.clone());
                                if lhs.is_player_reference() {
                                    paths_to_players
                                        .entry(rhs.clone())
                                        .or_default()
                                        .insert((edge.rhs.clone(), assignments));
                                } else {
                                    queue.push((edge.rhs.clone(), assignments));
                                }
                            } else if let EdgeLabel::Tag { symbol } = &edge.label {
                                paths_to_tags
                                    .entry(symbol.clone())
                                    .or_default()
                                    .insert((edge.rhs.clone(), assignments));
                            } else {
                                queue.push((edge.rhs.clone(), assignments));
                            }
                        }
                    }
                }
            }

            if paths_to_players.len() <= 1
                && paths_to_tags.into_values().all(|paths| paths.len() == 1)
            {
                simple_apply_edge_names.insert(edge_name.clone());
            }
        }

        self.pragmas.retain(|pragma| {
            if let Pragma::SimpleApply { edge_names, .. } = pragma {
                simple_apply_edge_names.extend(edge_names.iter().cloned());
                false
            } else {
                true
            }
        });

        if !simple_apply_edge_names.is_empty() {
            let pragma = Pragma::SimpleApply {
                span: Span::none(),
                edge_names: simple_apply_edge_names.into_iter().collect(),
            };

            let index = self.pragmas.partition_point(|x| *x < pragma);
            self.pragmas.insert(index, pragma);
        }

        Ok(())
    }
}
