use crate::ast::{Error, ErrorReason, Game, Label};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn check_tag_variables(&self) -> Result<(), Error<Arc<str>>> {
        for edge in &self.edges {
            if let Label::TagVariable { identifier } = &edge.label {
                if self.variables.iter().all(|x| x.identifier != *identifier) {
                    return self.make_error(ErrorReason::UnresolvedVariable {
                        identifier: identifier.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}
