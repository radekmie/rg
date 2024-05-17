use crate::ast::{Edge, Error, Game, Label};

impl<Id: PartialEq + Clone> Game<Id> {
    pub fn skip_generator_comparisons(&mut self) -> Result<(), Error<Id>> {
        let mut edges_to_skip = vec![];
        for edge in &self.edges {
            if self.incoming_edge(&edge.rhs).is_none() || edge.lhs.has_bindings() {
                continue;
            }
            let Some(bind) = edge
                .rhs
                .binding()
                .filter(|bind| is_generator_comparison(bind.0, &edge.label))
            else {
                continue;
            };
            if self
                .outgoing_edges(&edge.rhs)
                .any(|edge| edge_uses_bind(edge, bind.0))
            {
                continue;
            }
            edges_to_skip.push((*edge).clone());
        }

        for to_skip in &edges_to_skip {
            for edge in &mut self.edges {
                if edge.lhs == to_skip.rhs {
                    edge.lhs = to_skip.lhs.clone();
                }
            }
        }

        self.edges.retain(|edge| !edges_to_skip.contains(edge));

        Ok(())
    }
}

fn edge_uses_bind<Id: PartialEq>(edge: &Edge<Id>, bind: &Id) -> bool {
    edge.label.has_variable(bind)
        || edge.rhs.has_binding(bind)
        || edge.label.is_tag_and(|tag| tag == bind)
}

fn is_generator_comparison<Id: PartialEq>(bind: &Id, label: &Label<Id>) -> bool {
    match label {
        Label::Comparison { lhs, rhs, negated } if !negated => {
            lhs.is_reference_and(|e| e == bind) || rhs.is_reference_and(|e| e == bind)
        }
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        skip_generator_comparisons,
        small,
        "x, y(t: T): t == null;
        y(t: T), z: ;",
        "x, z: ;"
    );

    test_transform!(
        skip_generator_comparisons,
        large,
        "x, y(t: T): t == null;
        y(t: T), z: $ 1;
        y(t: T), a: $ 2;",
        "x, z: $1;
        x, a: $2;"
    );

    test_transform!(
        skip_generator_comparisons,
        old_binding,
        "x(t: T), y(t: T): t == null;
        y(t: T), z: ;"
    );

    test_transform!(
        skip_generator_comparisons,
        negated,
        "x, y(t: T): t != null;
        y(t: T), z: ;"
    );

    test_transform!(
        skip_generator_comparisons,
        binding_used,
        "x, y(t: T): t == null;
        y(t: T), z: t != 1;"
    );

    test_transform!(
        skip_generator_comparisons,
        binding_used_in_tag,
        "x, y(t: T): t == null;
        y(t: T), z: $ t;"
    );

    test_transform!(
        skip_generator_comparisons,
        binding_in_lhs,
        "x, y(t: T): null == t;
        y(t: T), z: ;",
        "x, z: ;"
    );
}
