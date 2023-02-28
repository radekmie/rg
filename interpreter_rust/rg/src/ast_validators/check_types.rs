use crate::ast::{Edge, EdgeLabel, Error, ErrorReason, Game};

impl<Id: Clone + PartialEq> Edge<Id> {
    pub fn check_types(&self, game: &Game<Id>) -> Result<(), Error<Id>> {
        self.label.check_types(game, self)
    }
}

impl<Id: Clone + PartialEq> EdgeLabel<Id> {
    pub fn check_types(&self, game: &Game<Id>, edge: &Edge<Id>) -> Result<(), Error<Id>> {
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
        for edge in &self.edges {
            edge.check_types(self)?
        }

        Ok(())
    }
}
