use crate::ast::{Error, Game};
use std::collections::BTreeSet;
use std::sync::Arc;

const IMPORTANT_VARIABLES: [&str; 3] = ["player", "goals", "visible"];

impl Game<Arc<str>> {
    pub fn prune_unused_variables(&mut self) -> Result<(), Error<Arc<str>>> {
        let mut unused_variables: BTreeSet<_> = self
            .variables
            .iter()
            .filter(|id| !IMPORTANT_VARIABLES.contains(&id.identifier.as_ref()))
            .map(|x| x.identifier.clone())
            .collect();

        for edge in &self.edges {
            let mut used_variables = edge.label.used_variables();
            if let Some((identifier, _)) = edge.label.as_var_assignment() {
                used_variables.retain(|var| *var != identifier);
            }
            if used_variables.is_empty() {
                continue;
            }
            unused_variables.retain(|var| !used_variables.contains(var));
            if unused_variables.is_empty() {
                return Ok(());
            }
        }

        for edge in &mut self.edges {
            if let Some((identifier, _)) = edge.label.as_var_assignment() {
                if unused_variables.contains(identifier) {
                    Arc::make_mut(edge).skip();
                }
            }
        }

        self.variables
            .retain(|var| !unused_variables.contains(&var.identifier));

        Ok(())
    }
}
