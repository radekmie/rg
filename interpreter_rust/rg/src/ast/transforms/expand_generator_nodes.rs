use crate::ast::{Edge, Error, Expression, Game, Label};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Id>> {
        for index in (0..self.edges.len()).rev() {
            if let Label::AssignmentAny { lhs, rhs } = &self.edges[index].label {
                let edge = &self.edges[index];
                let values = rhs.values(self)?;
                // TODO: Maybe we need a new node for each value?
                let new_edges: Vec<_> = values.into_iter().map(|value| {
                    let label = Label::Assignment {
                        lhs: lhs.clone(),
                        rhs: Arc::from(Expression::new(value)),
                    };
                    Arc::from(Edge::new(edge.lhs.clone(), edge.rhs.clone(), label))
                }).collect();
                self.edges.splice(
                    index..index + 1,
                    new_edges
                );

            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        expand_generator_nodes,
        edge,
        "type T = { a, b }; x, y: a = T(*);",
        "type T = { a, b }; type T = { a, b }; x, y: a = a; x, y: a = b;"
    );
}
