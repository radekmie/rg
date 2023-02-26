use crate::ast::{EdgeLabel, Error, GameDeclaration};

impl<Id: PartialEq> GameDeclaration<Id> {
    pub fn skip_self_assignments(mut self) -> Result<Self, Error<Id>> {
        for edge in &mut self.edges {
            if edge.label.is_self_assignment() {
                edge.label = EdgeLabel::Skip;
            }
        }

        Ok(self)
    }
}
