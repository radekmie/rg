use crate::ast::{Edge, Error, Game, Label};

impl<Id: Clone + Ord> Game<Id> {
    pub fn skip_generator_comparisons(&mut self) -> Result<(), Error<Id>> {
        let mut edges_to_skip = vec![];
        for node in self.nodes() {
            let Some(incoming_edge) = self.incoming_edge(node) else {
                continue;
            };
            for (bind, _) in node.bindings() {
                if incoming_edge.lhs.has_binding(bind)
                    || !is_generator_comparison(bind, &incoming_edge.label)
                    || self
                        .outgoing_edges(node)
                        .any(|edge| edge_uses_bind(edge, bind))
                {
                    continue;
                }
                edges_to_skip.push(incoming_edge.clone());
            }
        }

        for edge in &mut self.edges {
            if edges_to_skip.contains(edge) {
                edge.skip();
            }
        }

        Ok(())
    }
}

fn edge_uses_bind<Id: PartialEq>(edge: &Edge<Id>, bind: &Id) -> bool {
    edge.label.has_binding(bind) || edge.rhs.has_binding(bind)
}

fn is_generator_comparison<Id: PartialEq>(bind: &Id, label: &Label<Id>) -> bool {
    match label {
        Label::Comparison { lhs, rhs, negated } if !negated => {
            lhs.uncast().is_reference_and(|e| e == bind)
                || rhs.uncast().is_reference_and(|e| e == bind)
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
        "x, y(t: T): ;
        y(t: T), z: ;"
    );

    test_transform!(
        skip_generator_comparisons,
        large,
        "x, y(t: T): t == null;
        y(t: T), z: $ 1;
        y(t: T), a: $ 2;",
        "x, y(t: T): ;
        y(t: T), z: $ 1;
        y(t: T), a: $ 2;"
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
        "x, y(t: T): ;
        y(t: T), z: ;"
    );

    test_transform!(
        skip_generator_comparisons,
        binding_in_cast,
        "x, y(t: T): null == T(t);
        y(t: T), z: ;",
        "x, y(t: T): ;
        y(t: T), z: ;"
    );

    test_transform!(
        skip_generator_comparisons,
        another_binding_used,
        "x, y(t: T)(a: A): null == T(t);
        y(t: T)(a: A), z: coord = a;",
        "x, y(t: T)(a: A): ;
        y(t: T)(a: A), z: coord = a;"
    );

    test_transform!(
        skip_generator_comparisons,
        second_binding,
        "20_118_115(bind_18: Coord), 20_118_118: ;
        20_118_111(bind_19: Coord), 20_118_115(bind_18: Coord): Coord(bind_18) == coord;
        20_118_110, 20_118_111(bind_19: Coord): Coord(bind_19) == coord;",
        "20_118_115(bind_18: Coord), 20_118_118: ;
        20_118_111(bind_19: Coord), 20_118_115(bind_18: Coord): ;
        20_118_110, 20_118_111(bind_19: Coord): ;"
    );
}
