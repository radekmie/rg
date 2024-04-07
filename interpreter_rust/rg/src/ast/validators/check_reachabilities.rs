use crate::ast::{EdgeLabel, Error, ErrorReason, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn check_reachabilities(&self) -> Result<(), Error<Arc<str>>> {
        let next_nodes: BTreeMap<_, BTreeSet<_>> =
            self.edges
                .iter()
                .fold(BTreeMap::new(), |mut next_nodes, edge| {
                    next_nodes.entry(&edge.lhs).or_default().insert(&edge.rhs);
                    next_nodes
                });

        let is_reachable = |a: &Node<_>, b: &Node<_>| -> bool {
            let mut seen = BTreeSet::new();
            let mut queue = vec![a];
            while let Some(lhs) = queue.pop() {
                if let Some(rhss) = next_nodes.get(lhs) {
                    for rhs in rhss {
                        if !seen.contains(rhs) {
                            if rhs == &b {
                                return true;
                            }

                            seen.insert(rhs);
                            queue.push(rhs);
                        }
                    }
                }
            }

            false
        };

        let begin = Node::new(Arc::from("begin"));
        let end = Node::new(Arc::from("end"));
        if !is_reachable(&begin, &end) {
            return self.make_error(ErrorReason::Unreachable {
                lhs: begin,
                rhs: end,
            });
        }

        for edge in &self.edges {
            if let EdgeLabel::Reachability { lhs, rhs, .. } = &edge.label {
                if !is_reachable(lhs, rhs) {
                    return self.make_error(ErrorReason::Unreachable {
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}
