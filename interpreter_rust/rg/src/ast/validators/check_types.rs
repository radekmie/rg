use crate::ast::{Constant, Edge, Error, ErrorReason, Game, Label, Type, Value, Variable};
use std::{mem::take, sync::Arc};

impl<Id: Clone + PartialEq> Constant<Id> {
    fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.value.check_type(game, &self.type_)
    }
}

impl<Id: Clone + PartialEq> Edge<Id> {
    fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.label.check_type(game)
    }
}

impl<Id: Clone + PartialEq> Label<Id> {
    fn check_type(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        match self {
            Self::Assignment { lhs, rhs } => {
                let lhs = lhs.infer(game)?;
                let rhs = rhs.infer(game)?;
                if !game.is_assignable_type(&lhs, &rhs, false)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch { lhs, rhs });
                }
            }
            Self::AssignmentAny { lhs, rhs } => {
                let lhs = lhs.infer(game)?;
                if !game.is_assignable_type(&lhs, rhs, true)? {
                    return game.make_error(ErrorReason::AssignmentTypeMismatch {
                        lhs,
                        rhs: rhs.clone(),
                    });
                }
            }
            Self::Comparison { lhs, rhs, .. } => {
                let lhs = lhs.infer(game)?;
                let rhs = rhs.infer(game)?;
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
        for edge in &self.edges {
            edge.check_type(self)?;
        }

        // Constants can reference only previously defined constants.
        let mut game = self.clone();
        let variables = take(&mut game.variables);
        for constant in take(&mut game.constants) {
            constant.check_type(&game)?;
            game.constants.push(constant);
        }

        // Cariables can reference only constants.
        for variable in variables {
            variable.check_type(&game)?;
        }

        Ok(())
    }

    fn check_is_assignable_identifier(
        &self,
        lhs: &Arc<Type<Id>>,
        rhs: &Id,
    ) -> Result<(), Error<Id>> {
        if self.is_assignable_identifier(lhs, rhs)? {
            return Ok(());
        }

        self.make_error(ErrorReason::AssignmentTypeMismatch {
            lhs: lhs.clone(),
            rhs: self.infer(rhs),
        })
    }
}

impl<Id: Clone + PartialEq> Value<Id> {
    fn check_type(&self, game: &Game<Id>, type_: &Arc<Type<Id>>) -> Result<(), Error<Id>> {
        match self {
            Self::Element { identifier } => game.check_is_assignable_identifier(type_, identifier),
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
                        game.check_is_assignable_identifier(key_type, identifier)?;
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
    use crate::ast::{ErrorReason, Span, Type};
    use crate::test_validator;
    use std::sync::Arc;

    test_validator!(
        check_types,
        unresolved_type,
        "var t: T = t;",
        Err(ErrorReason::UnresolvedType {
            identifier: Arc::from("T")
        })
    );

    test_validator!(
        check_types,
        constant_using_constant,
        "type T = { 0 }; const x: T = 0; const y: T -> T = { :x };",
        Ok(())
    );

    test_validator!(
        check_types,
        constant_using_constant_reverse,
        "type T = { 0 }; const y: T -> T = { :x }; const x: T = 0;",
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T")
            }),
            rhs: Arc::from(Type::Set {
                span: Span::none(),
                identifiers: vec![Arc::from("x")]
            })
        })
    );

    test_validator!(
        check_types,
        constant_using_variable,
        "type T = { 0 }; var x: T = 0; const y: T -> T = { :x };",
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T")
            }),
            rhs: Arc::from(Type::Set {
                span: Span::none(),
                identifiers: vec![Arc::from("x")]
            })
        })
    );

    test_validator!(
        check_types,
        variable_using_constant,
        "type T = { 0 }; const x: T = 0; var y: T -> T = { :x };",
        Ok(())
    );

    test_validator!(
        check_types,
        variable_using_constant_reverse,
        "type T = { 0 }; var y: T -> T = { :x }; const x: T = 0;",
        Ok(())
    );

    test_validator!(
        check_types,
        variable_using_variable,
        "type T = { 0 }; var x: T = 0; var y: T -> T = { :x };",
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T")
            }),
            rhs: Arc::from(Type::Set {
                span: Span::none(),
                identifiers: vec![Arc::from("x")]
            })
        })
    );

    test_validator!(
        check_types,
        variable_using_variable_reverse,
        "type T = { 0 }; var y: T -> T = { :x }; var x: T = 0;",
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T")
            }),
            rhs: Arc::from(Type::Set {
                span: Span::none(),
                identifiers: vec![Arc::from("x")]
            })
        })
    );

    test_validator!(
        check_types,
        self_reference_constant,
        "type T = { 0 }; const t: T = t;",
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T")
            }),
            rhs: Arc::from(Type::Set {
                span: Span::none(),
                identifiers: vec![Arc::from("t")]
            })
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
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T")
            }),
            rhs: Arc::from(Type::Set {
                span: Span::none(),
                identifiers: vec![Arc::from("t")]
            })
        })
    );

    test_validator!(
        check_types,
        self_reference_variable_identifier,
        "type T = { t }; var t: T = t;",
        Ok(())
    );

    test_validator!(
        check_types,
        assign_any_correct,
        "type T = { 0 }; var t: T = 0; a, b: t = T(*);",
        Ok(())
    );

    test_validator!(
        check_types,
        assign_any_smaller,
        "type T0 = { 0 }; type T1 = { 0, 1 }; var t: T1 = 0; a, b: t = T0(*);",
        Ok(())
    );

    test_validator!(
        check_types,
        assign_any_bigger,
        "type T0 = { 0 }; type T1 = { 0, 1 }; var t: T0 = 0; a, b: t = T1(*);",
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T0")
            }),
            rhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T1")
            })
        })
    );

    test_validator!(
        check_types,
        assign_any_mismtach,
        "type T0 = { 0 }; type T1 = { 1 }; var t: T0 = 0; a, b: t = T1(*);",
        Err(ErrorReason::AssignmentTypeMismatch {
            lhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T0")
            }),
            rhs: Arc::from(Type::TypeReference {
                identifier: Arc::from("T1")
            })
        })
    );
}
