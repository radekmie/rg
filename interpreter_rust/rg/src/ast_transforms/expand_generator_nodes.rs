use crate::ast::{Error, GameDeclaration};
use std::rc::Rc;

impl GameDeclaration<String> {
    pub fn expand_generator_nodes(mut self) -> Result<Self, Error<String>> {
        self.edges = self
            .edges
            .iter()
            .map(|edge| {
                self.create_mappings(edge.bindings()).map(|mappings| {
                    mappings
                        .into_iter()
                        .map(|mapping| Rc::new(edge.substitute_bindings(&mapping)))
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(self)
    }
}
