use crate::ast::{Error, Game};
use std::collections::BTreeSet;

impl<Id: PartialEq + Ord + Clone> Game<Id> {
    pub fn prune_unused_constants(&mut self) -> Result<(), Error<Id>> {
        let used_constantss = self
            .variables
            .iter()
            .map(|x| x.default_value.identifiers())
            .chain(self.constants.iter().map(|x| x.value.identifiers()))
            .chain(self.edges.iter().map(|x| x.label.used_variables()));

        let mut unused_constants: BTreeSet<_> = self
            .constants
            .iter()
            .map(|x| x.identifier.clone())
            .collect();

        for used_constants in used_constantss {
            if used_constants.is_empty() {
                continue;
            }
            unused_constants.retain(|var| !used_constants.contains(var));
            if unused_constants.is_empty() {
                return Ok(());
            }
        }

        self.constants
            .retain(|constant| !unused_constants.contains(&constant.identifier));

        Ok(())
    }
}
