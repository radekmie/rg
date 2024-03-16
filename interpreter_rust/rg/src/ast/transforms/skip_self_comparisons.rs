use crate::ast::{EdgeLabel, Error, Game};
use crate::position::Span;

impl<Id: PartialEq> Game<Id> {
    pub fn skip_self_comparisons(&mut self) -> Result<(), Error<Id>> {
        for edge in &mut self.edges {
            if edge.label.is_self_comparison() {
                edge.label = EdgeLabel::Skip { span: Span::none() };
            }
        }

        Ok(())
    }
}
