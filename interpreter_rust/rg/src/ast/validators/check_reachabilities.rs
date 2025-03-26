use crate::ast::{Error, ErrorReason, Game, Node};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn check_reachabilities(&self) -> Result<(), Error<Arc<str>>> {
        let lhs = Node::new(Arc::from("begin"));
        let rhs = Node::new(Arc::from("end"));

        let check_reachability = self.make_check_reachability(false);
        if !check_reachability(&lhs, &rhs).is_reachable() {
            return self.make_error(ErrorReason::Unreachable { lhs, rhs });
        }

        Ok(())
    }
}
