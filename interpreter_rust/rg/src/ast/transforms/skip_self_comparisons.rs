use crate::ast::{Error, Game, Label};

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn skip_self_comparisons(&mut self) -> Result<(), Error<Id>> {
        let mut unreachable = Vec::new();
        for edge in &mut self.edges {
            match &edge.label {
                Label::Comparison { lhs, rhs, negated } if lhs.is_equal_reference(rhs) => {
                    if *negated {
                        unreachable.push(edge.clone());
                    } else {
                        edge.skip();
                    }
                }
                _ => (),
            }
        }

        self.edges.retain(|edge| !unreachable.contains(edge));

        Ok(())
    }
}
