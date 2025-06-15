use crate::ast::{Error, ErrorReason, Game};

impl<Id: Clone + PartialEq> Game<Id> {
    pub fn check_tag_loops(&self) -> Result<(), Error<Id>> {
        for edge in &self.edges {
            if edge.lhs == edge.rhs && (edge.label.is_tag() || edge.label.is_tag_variable()) {
                return self.make_error(ErrorReason::TagLoop { edge: edge.clone() });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{Edge, Label, Node};
    use crate::test_validator;

    test_validator!(check_tag_loops, no_loop, "x, y: $v;", Ok(()));
    test_validator!(check_tag_loops, no_loop_variable, "x, y: $$v;", Ok(()));

    test_validator!(
        check_tag_loops,
        loop_,
        "x, x: $v;",
        Err(ErrorReason::TagLoop {
            edge: Arc::from(Edge::new(
                Node::new(Arc::from("x")),
                Node::new(Arc::from("x")),
                Label::Tag {
                    symbol: Arc::from("v")
                }
            ))
        })
    );

    test_validator!(
        check_tag_loops,
        loop_variable,
        "x, x: $$v;",
        Err(ErrorReason::TagLoop {
            edge: Arc::from(Edge::new(
                Node::new(Arc::from("x")),
                Node::new(Arc::from("x")),
                Label::TagVariable {
                    identifier: Arc::from("v")
                }
            ))
        })
    );
}
