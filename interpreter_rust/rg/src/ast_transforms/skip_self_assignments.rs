use crate::ast::{EdgeDeclaration, EdgeLabel, Error, GameDeclaration};
use std::rc::Rc;

impl<Id: PartialEq> GameDeclaration<Id> {
    pub fn skip_self_assignments(mut self) -> Result<Self, Error<Id>> {
        for edge in &mut self.edges {
            if edge.label.is_self_assignment() {
                *edge = Rc::new(EdgeDeclaration {
                    label: Rc::new(EdgeLabel::Skip),
                    lhs: edge.lhs.clone(),
                    rhs: edge.rhs.clone(),
                });
            }
        }

        Ok(self)
    }
}
