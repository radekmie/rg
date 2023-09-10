use crate::ast::{EdgeLabel, EdgeName, Error, ErrorReason, Game};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

impl Game<Rc<str>> {
    pub fn check_reachabilities(&self) -> Result<(), Error<Rc<str>>> {
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

        let is_reachable = |a: &EdgeName<Rc<str>>, b: &EdgeName<Rc<str>>| -> bool {
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

        let begin = EdgeName::from(Rc::from("begin"));
        let end = EdgeName::from(Rc::from("end"));
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
