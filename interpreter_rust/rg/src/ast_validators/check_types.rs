use crate::ast::{
    Constant, Edge, EdgeLabel, Error, ErrorReason, Game, Type, Value, ValueEntry, Variable,
};
use std::rc::Rc;

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

impl<Id: Clone + PartialEq> EdgeLabel<Id> {
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
            constant.check_type(self)?
        }

        for edge in &self.edges {
            edge.check_type(self)?
        }

        for variable in &self.variables {
            variable.check_type(self)?
        }

        Ok(())
    }
}

impl<Id: Clone + PartialEq> Value<Id> {
    pub fn check_type(&self, game: &Game<Id>, type_: &Rc<Type<Id>>) -> Result<(), Error<Id>> {
        match self {
            Self::Element { identifier } => {
                let rhs = game.infer(identifier, None);
                if !game.is_assignable_type(type_, &rhs, false)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch {
                        lhs: type_.clone(),
                        rhs,
                    });
                }
            }
            Self::Map { entries } => {
                if let Type::Arrow {
                    lhs: key_type,
                    rhs: value_type,
                } = type_.resolve(game)?
                {
                    for entry in entries {
                        match entry {
                            ValueEntry::DefaultEntry { value } => {
                                value.check_type(game, value_type)?;
                            }
                            ValueEntry::NamedEntry { identifier, value } => {
                                let rhs = game.infer(identifier, None);
                                if !game.is_assignable_type(key_type, &rhs, false)? {
                                    return game.make_error(ErrorReason::AssignmentTypeMismatch {
                                        lhs: key_type.clone(),
                                        rhs,
                                    });
                                }
                                value.check_type(game, value_type)?;
                            }
                        }
                    }
                } else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: type_.clone() });
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
