use crate::ast::{EdgeLabel, EdgeName, Error, Game};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn prune_unreachable_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges: BTreeMap<_, BTreeSet<_>> = self
            .edges
            .iter()
            .fold(BTreeMap::new(), |mut next_edges, edge| {
                next_edges.entry(&edge.lhs).or_default().insert(edge);
                next_edges
            });

        let mut seen = BTreeSet::new();
        let mut queue = vec![EdgeName::new(Arc::from("begin"))];
        while let Some(lhs) = queue.pop() {
            let maybe_edges = next_edges.get(&lhs);
            if seen.insert(lhs) {
                if let Some(edges) = maybe_edges {
                    for edge in edges {
                        if !seen.contains(&edge.rhs) {
                            queue.push(edge.rhs.clone());
                        }

                        if let EdgeLabel::Reachability { lhs, .. } = &edge.label {
                            if !seen.contains(lhs) {
                                queue.push(lhs.clone());
                            }
                        }
                    }
                }
            }
        }

        self.edges
            .retain(|edge| seen.contains(&edge.lhs) && seen.contains(&edge.rhs));

        Ok(())
    }
}
