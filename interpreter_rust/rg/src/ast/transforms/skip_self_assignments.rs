use crate::ast::{Error, Game, Label};
use crate::position::Span;

impl<Id: PartialEq> Game<Id> {
    pub fn skip_self_assignments(&mut self) -> Result<(), Error<Id>> {
        for edge in &mut self.edges {
            if edge.label.is_self_assignment() {
                edge.label = Label::Skip { span: Span::none() };
            }
        }

        Ok(())
    }
}
