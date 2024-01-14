use crate::ast::{Edge, Error, ErrorReason, Game};
use std::collections::BTreeSet;

impl<Id: Clone + Ord> Game<Id> {
    pub fn check_multiple_edges(&self) -> Result<(), Error<Id>> {
        let mut pairs = BTreeSet::new();
        for Edge { lhs, rhs, .. } in &self.edges {
            if !pairs.insert((lhs, rhs)) {
                return self.make_error(ErrorReason::MultipleEdges {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                });
            }
        }

        Ok(())
    }
}
