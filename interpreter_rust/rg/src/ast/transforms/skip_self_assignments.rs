use crate::ast::{Error, Game};
use std::sync::Arc;

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn skip_self_assignments(&mut self) -> Result<(), Error<Id>> {
        for edge in &mut self.edges {
            if edge.label.is_self_assignment() {
                Arc::make_mut(edge).skip();
            }
        }

        Ok(())
    }
}
