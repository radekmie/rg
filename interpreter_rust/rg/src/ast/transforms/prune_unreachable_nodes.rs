use crate::ast::analyses::ReachableNodes;
use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn prune_unreachable_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        let reachable_nodes = self.analyse::<ReachableNodes>(true);
        self.edges.retain(|edge| {
            reachable_nodes
                .get(&edge.lhs)
                .is_some_and(|reachable| *reachable)
        });

        let next_edges = self.next_edges();
        self.edges
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, edge)| {
                !edge.rhs.is_end()
                    && !next_edges.contains_key(&edge.rhs)
                    && !self.is_reachability_target(&edge.rhs)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|index| {
                self.edges.remove(index);
            });

        Ok(())
    }
}
