use crate::ast::{Error, ErrorReason, Game};
use std::collections::BTreeSet;

impl<Id: Clone + Ord> Game<Id> {
    pub fn check_duplicated_names(&self) -> Result<(), Error<Id>> {
        macro_rules! check {
            ($list:ident, $error:path) => {
                let mut identifiers = BTreeSet::new();
                for x in &self.$list {
                    if !identifiers.insert(&x.identifier) {
                        return self.make_error({
                            $error {
                                identifier: x.identifier.clone(),
                            }
                        });
                    }
                }
            };
        }

        check!(constants, ErrorReason::DuplicatedConstant);
        check!(typedefs, ErrorReason::DuplicatedTypedef);
        check!(variables, ErrorReason::DuplicatedVariable);

        Ok(())
    }
}
