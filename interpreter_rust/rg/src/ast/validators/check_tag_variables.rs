use crate::ast::{Error, ErrorReason, Game, Label};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn check_tag_variables(&self) -> Result<(), Error<Arc<str>>> {
        for edge in &self.edges {
            if let Label::TagVariable { identifier } = &edge.label {
                if let Some(variable) = self.variables.iter().find(|x| x.identifier == *identifier)
                {
                    let type_ = variable.type_.resolve(self)?;
                    if !type_.is_set() {
                        return self.make_error(ErrorReason::SetTypeExpected {
                            got: Arc::from(type_.clone()),
                        });
                    }
                } else {
                    return self.make_error(ErrorReason::UnresolvedVariable {
                        identifier: identifier.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{ErrorReason, Type};
    use crate::test_validator;
    use std::sync::Arc;
    use utils::position::Span;

    test_validator!(
        check_tag_variables,
        correct,
        "type T = { 0 }; var v: T = 0; x, y: $$v;",
        Ok(())
    );

    test_validator!(
        check_tag_variables,
        unresolved_variable,
        "x, y: $$v;",
        Err(ErrorReason::UnresolvedVariable {
            identifier: Arc::from("v")
        })
    );

    test_validator!(
        check_tag_variables,
        arrow_type,
        "type T = { 0 } -> { 0 }; var v: T = { :0 }; x, y: $$v;",
        Err(ErrorReason::SetTypeExpected {
            got: Arc::from(Type::Arrow {
                lhs: Arc::from(Type::Set {
                    span: Span::none(),
                    identifiers: vec![Arc::from("0")]
                }),
                rhs: Arc::from(Type::Set {
                    span: Span::none(),
                    identifiers: vec![Arc::from("0")]
                })
            })
        })
    );
}
