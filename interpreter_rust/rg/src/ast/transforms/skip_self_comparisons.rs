use crate::ast::{Error, Game, Label};
use std::sync::Arc;

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn skip_self_comparisons(&mut self) -> Result<(), Error<Id>> {
        for edge in &mut self.edges {
            if let Label::Comparison { lhs, rhs, negated } = &edge.label {
                if lhs.is_equal_reference(rhs) && !*negated {
                    Arc::make_mut(edge).skip();
                }
            }
        }

        Ok(())
    }
}
