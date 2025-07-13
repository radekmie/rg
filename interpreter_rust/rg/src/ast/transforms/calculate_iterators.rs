use crate::ast::{Error, Expression, Game, Label, Pragma};
use std::collections::BTreeMap;
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;

impl Game<Id> {
    pub fn calculate_iterators(&mut self) -> Result<(), Error<Id>> {
        let single_next_edges: BTreeMap<_, _> = self
            .next_edges()
            .into_iter()
            .filter_map(|(node, mut edges)| {
                edges
                    .pop_first()
                    .filter(|_| edges.is_empty())
                    .map(|edge| (node, edge))
            })
            .collect();

        let mut pragmas = vec![];
        for edge_a_b in single_next_edges.values() {
            if let Some(variable) = edge_a_b.label.as_var_assignment() {
                if let Some(edge_b_c) = single_next_edges.get(&edge_a_b.rhs) {
                    if edge_b_c.label.is_iterator_lookup(variable) {
                        pragmas.push(Pragma::Iterator {
                            span: Span::none(),
                            node_a: edge_a_b.lhs.clone(),
                            node_b: edge_a_b.rhs.clone(),
                            node_c: edge_b_c.rhs.clone(),
                            variable: variable.clone(),
                        });
                    }
                }
            }
        }

        for pragma in pragmas {
            self.add_pragma(pragma);
        }

        Ok(())
    }
}

impl Label<Id> {
    /// Check whether it matches `expr[variable] == 1`.
    fn is_iterator_lookup(&self, variable: &Id) -> bool {
        matches!(self, Self::Comparison { lhs, rhs, negated: false }
            if rhs.uncast().is_reference_and(|x| x.as_ref() == "1") &&
                matches!(lhs.as_ref(), Expression::Access { rhs, .. }
                    if rhs.uncast().is_reference_and(|x| x == variable)))
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_iterators,
        normal,
        "a, b: x = T(*); b, c: map[x] == 1;",
        adds "@iterator a b c : x;"
    );

    test_transform!(
        calculate_iterators,
        nested,
        "a, b: x = T(*); b, c: map[y][x] == 1;",
        adds "@iterator a b c : x;"
    );

    test_transform!(
        calculate_iterators,
        nested_inversed,
        "a, b: x = T(*); b, c: map[x][y] == 1;"
    );

    test_transform!(
        calculate_iterators,
        different_comparison_1,
        "a, b: x = T(*); b, c: map[x] != 1;"
    );

    test_transform!(
        calculate_iterators,
        different_comparison_2,
        "a, b: x = T(*); b, c: map[x] == 0;"
    );

    test_transform!(
        calculate_iterators,
        different_variable,
        "a, b: x = T(*); b, c: map[y] == 1;"
    );
}
