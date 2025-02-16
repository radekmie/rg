use crate::ast::{Error, ErrorReason, Game, Label};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn check_assignments(&self) -> Result<(), Error<Id>> {
        for edge in &self.edges {
            edge.label.check_assignments(self)?;
        }

        Ok(())
    }
}

impl Label<Id> {
    pub fn check_assignments(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        if let Self::Assignment { lhs, .. } | Self::AssignmentAny { lhs, .. } = self {
            let identifier = lhs.access_identifier();
            if !game.variables.iter().any(|x| x.identifier == *identifier) {
                return game.make_error(ErrorReason::ConstantAssignment {
                    identifier: identifier.clone(),
                    label: self.clone(),
                });
            }
        }

        if let Self::AssignmentAny { lhs, .. } = self {
            if lhs.is_player_reference() {
                return game.make_error(ErrorReason::PlayerAnyAssignment {
                    label: self.clone(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{Expression, Label, Span, Type};
    use crate::test_validator;

    test_validator!(
        check_assignments,
        symbol,
        "begin, end: 0 = 1;",
        Err(ErrorReason::ConstantAssignment {
            identifier: Arc::from("0"),
            label: Label::Assignment {
                lhs: Arc::from(Expression::new(Arc::from("0"))),
                rhs: Arc::from(Expression::new(Arc::from("1")))
            }
        })
    );

    test_validator!(
        check_assignments,
        constant_direct,
        "const x: Bool -> Bool = { :0 }; begin, end: x = x;",
        Err(ErrorReason::ConstantAssignment {
            identifier: Arc::from("x"),
            label: Label::Assignment {
                lhs: Arc::from(Expression::new(Arc::from("x"))),
                rhs: Arc::from(Expression::new(Arc::from("x")))
            }
        })
    );

    test_validator!(
        check_assignments,
        constant_access,
        "const x: Bool -> Bool = { :0 }; begin, end: x[0] = 1;",
        Err(ErrorReason::ConstantAssignment {
            identifier: Arc::from("x"),
            label: Label::Assignment {
                lhs: Arc::from(Expression::Access {
                    span: Span::none(),
                    lhs: Arc::from(Expression::new(Arc::from("x"))),
                    rhs: Arc::from(Expression::new(Arc::from("0")))
                }),
                rhs: Arc::from(Expression::new(Arc::from("1")))
            }
        })
    );

    test_validator!(
        check_assignments,
        player_any,
        "var player: Player = x; begin, end: player = Player(*);",
        Err(ErrorReason::PlayerAnyAssignment {
            label: Label::AssignmentAny {
                lhs: Arc::from(Expression::new(Arc::from("player"))),
                rhs: Arc::from(Type::new(Arc::from("Player")))
            }
        })
    );

    test_validator!(
        check_assignments,
        variable_direct,
        "var x: Bool -> Bool = { :0 }; begin, end: x = x;",
        Ok(())
    );

    test_validator!(
        check_assignments,
        variable_any,
        "var x: Bool -> Bool = { :0 }; begin, end: x = Bool(*);",
        Ok(())
    );

    test_validator!(
        check_assignments,
        variable_access,
        "var x: Bool -> Bool = { :0 }; begin, end: x[0] = 1;",
        Ok(())
    );
}
