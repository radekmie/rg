use crate::ast::{EdgeLabel, Error, ErrorReason, GameDeclaration};

impl<Id: Clone + PartialEq> GameDeclaration<Id> {
    pub fn check_types(&self) -> Result<(), Error<Id>> {
        for edge in &self.edges {
            match &*edge.label {
                EdgeLabel::Assignment { lhs, rhs } => {
                    let lhs = self.infer_expression(edge, lhs)?;
                    let rhs = self.infer_expression(edge, rhs)?;
                    if !self.is_assignable_type(&lhs, &rhs, false)? {
                        return self.make_error(ErrorReason::AssignmentTypeMismatch { lhs, rhs });
                    }
                }
                EdgeLabel::Comparison { lhs, rhs, .. } => {
                    let lhs = self.infer_expression(edge, lhs)?;
                    let rhs = self.infer_expression(edge, rhs)?;
                    if !self.is_assignable_type(&lhs, &rhs, false)?
                        && !self.is_assignable_type(&rhs, &lhs, false)?
                    {
                        return self.make_error(ErrorReason::ComparisonTypeMismatch { lhs, rhs });
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
