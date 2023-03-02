use crate::ast::{Error, Game};
use std::rc::Rc;

impl Game<Rc<str>> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Rc<str>>> {
        for index in (0..self.edges.len()).rev() {
            let mappings = self.create_mappings(self.edges[index].bindings())?;
            if !mappings.is_empty() {
                for mapping in &mappings {
                    self.edges
                        .push(self.edges[index].substitute_bindings(mapping));
                }

                self.edges.remove(index);
            }
        }

        Ok(())
    }
}
