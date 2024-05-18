use crate::ast::{Constant, Edge, Error, ErrorReason, Game, Label, Type, Value, Variable};
use std::sync::Arc;

impl<Id: Clone + PartialEq> Constant<Id> {
    fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.value.check_type(game, &self.type_)
    }
}

impl<Id: Clone + PartialEq> Edge<Id> {
    fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.label.check_type(game, Some(self))
    }
}

impl<Id: Clone + PartialEq> Label<Id> {
    fn check_type(&self, game: &Game<Id>, edge: Option<&Edge<Id>>) -> Result<(), Error<Id>> {
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

    fn check_is_statically_assignable(
        &self,
        lhs: &Arc<Type<Id>>,
        rhs: &Id,
    ) -> Result<(), Error<Id>> {
        if let Type::Set { identifiers, .. } = lhs.resolve(self)? {
            if identifiers.contains(rhs) {
                return Ok(());
            }
        }

        if self.resolve_constant(rhs).is_some() {
            return self.make_error(ErrorReason::UnexpectedConstant {
                identifier: rhs.clone(),
            });
        }

        if self.resolve_variable(rhs).is_some() {
            return self.make_error(ErrorReason::UnexpectedVariable {
                identifier: rhs.clone(),
            });
        }

        self.make_error(ErrorReason::AssignmentTypeMismatch {
            lhs: lhs.clone(),
            rhs: self.infer(rhs, None),
        })
    }
}

impl<Id: Clone + PartialEq> Value<Id> {
    fn check_type(&self, game: &Game<Id>, type_: &Arc<Type<Id>>) -> Result<(), Error<Id>> {
        match self {
            Self::Element { identifier } => game.check_is_statically_assignable(type_, identifier),
            Self::Map { entries, .. } => {
                let Type::Arrow {
                    lhs: key_type,
                    rhs: value_type,
                } = type_.resolve(game)?
                else {
                    return game.make_error(ErrorReason::ArrowTypeExpected { got: type_.clone() });
                };

                for entry in entries {
                    entry.value.check_type(game, value_type)?;
                    if let Some(identifier) = &entry.identifier {
                        game.check_is_statically_assignable(key_type, identifier)?;
                    }
                }

                Ok(())
            }
        }
    }
}

impl<Id: Clone + PartialEq> Variable<Id> {
    fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.default_value.check_type(game, &self.type_)
    }
}

#[cfg(test)]
mod test {
    use crate::test_validator;

    test_validator!(
        check_types,
        unresolved_type,
        "var t: T = t;",
        Err(ErrorReason::UnresolvedType {
            identifier: "T".into()
        })
    );

    test_validator!(
        check_types,
        self_reference_constant,
        "type T = { 0 }; const t: T = t;",
        Err(ErrorReason::UnexpectedConstant {
            identifier: "t".into()
        })
    );

    test_validator!(
        check_types,
        self_reference_constant_identifier,
        "type T = { t }; const t: T = t;",
        Ok(())
    );

    test_validator!(
        check_types,
        self_reference_variable,
        "type T = { 0 }; var t: T = t;",
        Err(ErrorReason::UnexpectedVariable {
            identifier: "t".into()
        })
    );

    test_validator!(
        check_types,
        self_reference_variable_identifier,
        "type T = { t }; var t: T = t;",
        Ok(())
    );
}
