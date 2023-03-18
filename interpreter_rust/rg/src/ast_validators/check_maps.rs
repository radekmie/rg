use crate::ast::{Constant, Error, ErrorReason, Game, Value, ValueEntry, Variable};
use std::collections::BTreeSet;

impl<Id: Clone + Ord> Constant<Id> {
    pub fn check_maps(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.value.check_maps(game)
    }
}

impl<Id: Clone + Ord> Game<Id> {
    pub fn check_maps(&self) -> Result<(), Error<Id>> {
        for constant in &self.constants {
            constant.check_maps(self)?
        }

        for variable in &self.variables {
            variable.check_maps(self)?
        }

        Ok(())
    }
}

impl<Id: Clone + Ord> Value<Id> {
    pub fn check_maps(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        if let Self::Map { entries } = self {
            let mut keys = BTreeSet::new();
            for ValueEntry { identifier, value } in entries {
                value.check_maps(game)?;
                if !keys.insert(identifier) {
                    return game.make_error(ErrorReason::DuplicatedMapKey {
                        key: identifier.clone(),
                        value: self.clone(),
                    });
                }
            }

            if !keys.contains(&None) {
                return game.make_error(ErrorReason::MissingDefaultValue {
                    value: self.clone(),
                });
            }
        }

        Ok(())
    }
}

impl<Id: Clone + Ord> Variable<Id> {
    pub fn check_maps(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.default_value.check_maps(game)
    }
}
