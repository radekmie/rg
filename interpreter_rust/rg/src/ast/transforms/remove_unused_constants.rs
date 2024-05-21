use std::collections::BTreeSet;

use crate::ast::{Error, Game};

impl<Id: PartialEq + Ord + Clone> Game<Id> {
    pub fn remove_unused_constants(&mut self) -> Result<(), Error<Id>> {
        let used_constantss = self
            .variables
            .iter()
            .map(|x| x.default_value.used_variables())
            .chain(self.constants.iter().map(|x| x.value.used_variables()))
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
