use crate::ast::{Error, Game, Label};

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn skip_self_comparisons(&mut self) -> Result<(), Error<Id>> {
        let mut unreachable = vec![];
        for edge in &mut self.edges {
            if let Label::Comparison { lhs, rhs, negated } = &edge.label {
                if lhs.is_equal_reference(rhs) {
                    if *negated {
                        unreachable.push(edge.clone());
                    } else {
                        edge.skip();
                    }
                }
            }
        }

        self.edges.retain(|edge| !unreachable.contains(edge));

        Ok(())
    }
}
