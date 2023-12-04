use crate::ast::{EdgeLabel, EdgeName, Error, ErrorReason, Game};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn check_reachabilities(&self) -> Result<(), Error<Arc<str>>> {
        let next_edges: BTreeMap<&EdgeName<Arc<str>>, BTreeSet<&EdgeName<Arc<str>>>> = self
            .edges
            .iter()
            .fold(BTreeMap::default(), |mut next_edges, edge| {
                next_edges.entry(&edge.lhs).or_default().insert(&edge.rhs);
                next_edges
            });

        let is_reachable = |a: &EdgeName<Arc<str>>, b: &EdgeName<Arc<str>>| -> bool {
            let mut seen = BTreeSet::default();
            let mut queue = vec![a];
            while let Some(lhs) = queue.pop() {
                if let Some(rhss) = next_edges.get(lhs) {
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

        let begin = EdgeName::new(Arc::from("begin"));
        let end = EdgeName::new(Arc::from("end"));
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
