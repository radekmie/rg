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
            for entry in entries {
                match entry {
                    ValueEntry::DefaultEntry { value } => {
                        value.check_maps(game)?;
                        if !keys.insert(None) {
                            return game.make_error(ErrorReason::DuplicatedMapKey {
                                key: None,
                                value: self.clone(),
                            });
                        }
                    }
                    ValueEntry::NamedEntry { identifier, value } => {
                        value.check_maps(game)?;
                        if !keys.insert(Some(identifier)) {
                            return game.make_error(ErrorReason::DuplicatedMapKey {
                                key: Some(identifier.clone()),
                                value: self.clone(),
                            });
                        }
                    }
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
