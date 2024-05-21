use crate::ast::{Error, ErrorReason, Game, Label, Node};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn check_reachabilities(&self) -> Result<(), Error<Arc<str>>> {
        let is_reachable = self.make_is_reachable();

        let begin = Node::new(Arc::from("begin"));
        let end = Node::new(Arc::from("end"));
        if !is_reachable(&begin, &end) {
            return self.make_error(ErrorReason::Unreachable {
                lhs: begin,
                rhs: end,
            });
        }

        for edge in &self.edges {
            if let Label::Reachability { lhs, rhs, .. } = &edge.label {
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
