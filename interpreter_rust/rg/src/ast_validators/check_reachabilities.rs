use crate::ast::{EdgeLabel, EdgeName, Error, ErrorReason, Game};
use std::collections::{BTreeMap, BTreeSet};

impl Game<String> {
    pub fn check_reachabilities(&self) -> Result<(), Error<String>> {
        let next_edges = self
            .edges
            .iter()
            .fold(BTreeMap::default(), |mut next_edges, edge| {
                next_edges
                    .entry(&edge.lhs)
                    .or_insert_with(BTreeSet::default)
                    .insert(&edge.rhs);
                next_edges
            });

        let is_reachable = |a: &EdgeName<String>, b: &EdgeName<String>| -> bool {
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

        let begin = EdgeName::from("begin".to_string());
        let end = EdgeName::from("end".to_string());
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
