use crate::ast::{
    Constant, Edge, Error, ErrorReason, Game, Label, Type, Value, ValueEntry, Variable,
};
use std::sync::Arc;

impl<Id: Clone + PartialEq> Constant<Id> {
    pub fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.value.check_type(game, &self.type_)
    }
}

impl<Id: Clone + PartialEq> Edge<Id> {
    pub fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.label.check_type(game, Some(self))
    }
}

impl<Id: Clone + PartialEq> Label<Id> {
    pub fn check_type(&self, game: &Game<Id>, edge: Option<&Edge<Id>>) -> Result<(), Error<Id>> {
        match self {
            Self::Assignment { lhs, rhs } => {
                let lhs = lhs.infer(game, edge)?;
                let rhs = rhs.infer(game, edge)?;
                if !game.is_assignable_type(&lhs, &rhs, false)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch { lhs, rhs });
                }
            }
            Self::Comparison { lhs, rhs, .. } => {
                let lhs = lhs.infer(game, edge)?;
                let rhs = rhs.infer(game, edge)?;
                if !game.is_assignable_type(&lhs, &rhs, false)?
                    && !game.is_assignable_type(&rhs, &lhs, false)?
                {
                    return game.make_error(ErrorReason::ComparisonTypeMismatch { lhs, rhs });
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn check_types(&self) -> Result<(), Error<Id>> {
        for constant in &self.constants {
            constant.check_type(self)?;
        }

        for edge in &self.edges {
            edge.check_type(self)?;
        }

        for variable in &self.variables {
            variable.check_type(self)?;
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Value<Id> {
    pub fn check_type(&self, game: &Game<Id>, type_: &Arc<Type<Id>>) -> Result<(), Error<Id>> {
        match self {
            Self::Element { identifier } => {
                if !game.is_assignable_identifier(type_, identifier, false)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch {
                        lhs: type_.clone(),
                        rhs: game.infer(identifier, None),
                    });
                }
            }
            Self::Map { entries, .. } => {
                let Type::Arrow {
                    lhs: key_type,
                    rhs: value_type,
                } = type_.resolve(game)?
                else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: type_.clone() });
                };

                for ValueEntry {
                    identifier, value, ..
                } in entries
                {
                    value.check_type(game, value_type)?;
                    if let Some(identifier) = identifier {
                        if !game.is_assignable_identifier(key_type, identifier, false)? {
                            return game.make_error(ErrorReason::AssignmentTypeMismatch {
                                lhs: key_type.clone(),
                                rhs: game.infer(identifier, None),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Variable<Id> {
    pub fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.default_value.check_type(game, &self.type_)
    }
}
