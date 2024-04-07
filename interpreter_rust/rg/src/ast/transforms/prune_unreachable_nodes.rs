use crate::ast::{Error, Game, Label, Node};
use std::collections::BTreeSet;
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn prune_unreachable_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges = self.next_edges();
        let mut seen = BTreeSet::new();
        let mut queue = vec![Node::new(Arc::from("begin"))];
        while let Some(lhs) = queue.pop() {
            let maybe_edges = next_edges.get(&lhs);
            if seen.insert(lhs) {
                if let Some(edges) = maybe_edges {
                    for edge in edges {
                        if !seen.contains(&edge.rhs) {
                            queue.push(edge.rhs.clone());
                        }

                        if let Label::Reachability { lhs, .. } = &edge.label {
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
