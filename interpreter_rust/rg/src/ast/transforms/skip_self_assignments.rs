use crate::ast::{Error, Game};

impl<Id: PartialEq> Game<Id> {
    pub fn skip_self_assignments(&mut self) -> Result<(), Error<Id>> {
        for edge in &mut self.edges {
            if edge.label.is_self_assignment() {
                edge.skip();
            }
        }

        Ok(())
    }
}
