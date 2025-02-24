use crate::ast::{Edge, Error, Expression, Game, Label};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn expand_assignment_any(&mut self) -> Result<(), Error<Id>> {
        for index in (0..self.edges.len()).rev() {
            let edge = &self.edges[index];
            if let Label::AssignmentAny { lhs, rhs } = &edge.label {
                let new_edges: Vec<_> = rhs
                    .values(self)?
                    .into_iter()
                    .map(|value| {
                        let label = Label::Assignment {
                            lhs: lhs.clone(),
                            rhs: Arc::from(Expression::new(value)),
                        };
                        Arc::from(Edge::new(edge.lhs.clone(), edge.rhs.clone(), label))
                    })
                    .collect();
                self.edges.splice(index..index + 1, new_edges);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        expand_assignment_any,
        edge,
        "type T = { a, b }; x, y: a = T(*);",
        "type T = { a, b }; x, y: a = a; x, y: a = b;"
    );
}
