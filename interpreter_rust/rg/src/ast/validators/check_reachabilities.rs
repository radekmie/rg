use crate::ast::{Error, ErrorReason, Game, Node};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn check_reachabilities(&self) -> Result<(), Error<Arc<str>>> {
        let lhs = Node::new(Arc::from("begin"));
        let rhs = Node::new(Arc::from("end"));

        let is_reachable = self.make_is_reachable();
        if !is_reachable(&lhs, &rhs) {
            return self.make_error(ErrorReason::Unreachable { lhs, rhs });
        }

        Ok(())
    }
}
