use super::{gen_fresh_node, max_node_id};
use crate::ast::{Edge, Error, Expression, Game, Label};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn expand_tag_variable(&mut self) -> Result<(), Error<Id>> {
        let mut max_id = max_node_id(&self.nodes());
        for index in (0..self.edges.len()).rev() {
            if let Label::TagVariable { identifier } = &self.edges[index].label {
                let edge = &self.edges[index];

                let variable = self.resolve_variable_or_fail(identifier)?;
                let values = variable.type_.values(self)?;
                let new_edges: Vec<_> = values
                    .into_iter()
                    .flat_map(|symbol| {
                        let new_node = gen_fresh_node(&mut max_id);
                        let first = Edge::new(
                            edge.lhs.clone(),
                            new_node.clone(),
                            Label::Comparison {
                                lhs: Arc::from(Expression::new(variable.identifier.clone())),
                                rhs: Arc::from(Expression::new_cast(
                                    variable.type_.clone(),
                                    Arc::from(Expression::new(symbol.clone())),
                                )),
                                negated: false,
                            },
                        );
                        let second = Edge::new(new_node, edge.rhs.clone(), Label::Tag { symbol });
                        vec![Arc::from(first), Arc::from(second)]
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
        expand_tag_variable,
        edge,
        "type T = { a, b, c }; 
        var t: T = a; 
        x, y: $$ t;",
        "type T = { a, b, c };
        var t: T = a;
        x, 1: t == T(a);
        1, y: $ a;
        x, 2: t == T(b);
        2, y: $ b;
        x, 3: t == T(c);
        3, y: $ c;"
    );
}
