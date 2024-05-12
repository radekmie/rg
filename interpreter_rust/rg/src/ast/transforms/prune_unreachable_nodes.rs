use crate::ast::analysis::ReachableNodes;
use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn prune_unreachable_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_paths = self.analyse::<ReachableNodes>();
        self.edges.retain(|edge| {
            reaching_paths
                .get(&edge.lhs)
                .is_some_and(|reachable| *reachable)
        });
        Ok(())
    }
}
