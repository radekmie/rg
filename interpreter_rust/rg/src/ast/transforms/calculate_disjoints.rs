use crate::ast::{Edge, Error, Expression, Game, Label, Node, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_disjoints(&mut self) -> Result<(), Error<Arc<str>>> {
        let game = Self {
            constants: self.constants.clone(),
            typedefs: self.typedefs.clone(),
            variables: self.variables.clone(),
            ..Self::default()
        };

        let mut pragmas = vec![];
        for (node, edges) in self.next_edges() {
            if edges.len() == 1 {
                continue;
            }

            let node_bindings: BTreeSet<_> = node.bindings().collect();
            if edges.iter().any(|edge| edge.bindings() != node_bindings) {
                continue;
            }

            if let Some((is_exhaustive, mut nodes)) = game.get_disjoint(edges) {
                nodes.sort_unstable();
                let pragma = if is_exhaustive {
                    Pragma::DisjointExhaustive {
                        span: Span::none(),
                        node: node.clone(),
                        nodes,
                    }
                } else {
                    Pragma::Disjoint {
                        span: Span::none(),
                        node: node.clone(),
                        nodes,
                    }
                };

                pragmas.push(pragma);
            }
        }

        for pragma in pragmas {
            let index = self.pragmas.partition_point(|x| *x < pragma);
            if self.pragmas.get(index) != Some(&pragma) {
                self.pragmas.insert(index, pragma);
            }
        }

        Ok(())
    }

    fn get_disjoint(
        &self,
        mut edges: BTreeSet<&Arc<Edge<Arc<str>>>>,
    ) -> Option<(bool, Vec<Node<Arc<str>>>)> {
        // Take first comparison (if there is any).
        let e1 = match edges.iter().find(|edge| edge.label.is_comparison()) {
            None => edges.pop_first().unwrap(),
            Some(&edge) => edges.take(&edge.clone()).unwrap(),
        };

        // If-else.
        if edges.len() == 1 {
            if let Some(e2) = edges.first() {
                if e1.rhs != e2.rhs && e1.label.is_negated(&e2.label) {
                    return Some((true, vec![e1.rhs.clone(), e2.rhs.clone()]));
                }
            }
        }

        // Switch.
        if let Label::Comparison {
            lhs,
            rhs,
            negated: false,
        } = &e1.label
        {
            if let Expression::Reference { identifier } = rhs.uncast() {
                let lhs1 = lhs.uncast();
                let mut nodes = vec![e1.rhs.clone()];
                let mut symbols = BTreeSet::from([identifier]);
                for edge in edges {
                    if let Label::Comparison {
                        lhs: lhs2,
                        rhs,
                        negated: false,
                    } = &edge.label
                    {
                        if lhs1 == lhs2.uncast() {
                            if let Expression::Reference { identifier } = rhs.uncast() {
                                if symbols.insert(identifier) {
                                    nodes.push(edge.rhs.clone());
                                    continue;
                                }
                            }
                        }
                    }
                }

                if nodes.len() == 1 {
                    return None;
                }

                let is_exhaustive = lhs1
                    .infer(self, None)
                    .and_then(|type_| type_.values(self))
                    .is_ok_and(|values| values.len() == nodes.len());

                return Some((is_exhaustive, nodes));
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_disjoints,
        if_else_comparison,
        "begin, a: x == y; begin, b: x != y; a, end: ; b, end: ;",
        adds "@disjointExhaustive begin : a b;"
    );

    test_transform!(
        calculate_disjoints,
        if_else,
        "begin, a: ? x -> y; begin, b: ! x -> y; a, end: ; b, end: ; x, y: ;",
        adds "@disjointExhaustive begin : a b;"
    );

    test_transform!(
        calculate_disjoints,
        if_if,
        "begin, a: ? x -> y; begin, b: ? x -> y; a, end: ; b, end: ; x, y: ;"
    );

    test_transform!(
        calculate_disjoints,
        if_not_equal_paths_lhs,
        "begin, a: ? z -> y; begin, b: ! x -> y; a, end: ; b, end: ; x, y: ; z, y: ;"
    );

    test_transform!(
        calculate_disjoints,
        if_not_equal_paths_rhs,
        "begin, a: ? x -> y; begin, b: ! x -> z; a, end: ; b, end: ; x, y: ; x, z: ;"
    );

    test_transform!(calculate_disjoints, switch_1, "begin, a: x == 0; a, end: ;");

    test_transform!(
        calculate_disjoints,
        switch_2,
        "begin, a: x == 0; begin, b: x == 1; a, end: ; b, end: ;",
        adds "@disjoint begin : a b;"
    );

    test_transform!(
        calculate_disjoints,
        switch_3,
        "begin, a: x == 0; begin, b: x == 1; begin, c: x == 2; a, end: ; b, end: ; c, end: ;",
        adds "@disjoint begin : a b c;"
    );

    test_transform!(
        calculate_disjoints,
        switch_different_variable,
        "begin, a: x == 0; begin, b: y == 1; a, end: ; b, end: ;"
    );

    test_transform!(
        calculate_disjoints,
        switch_not_disjoint,
        "begin, a: x == 0; begin, b: x == 0; a, end: ; b, end: ;"
    );

    test_transform!(
        calculate_disjoints,
        switch_negated_1,
        "begin, a: x != 0; begin, b: x == 1; a, end: ; b, end: ;"
    );

    test_transform!(
        calculate_disjoints,
        switch_negated_2,
        "begin, a: x != 0; begin, b: x != 1; a, end: ; b, end: ;"
    );

    test_transform!(
        calculate_disjoints,
        if_else_comparison_binding_nested,
        "begin, entry: ; entry, a(b: Bool): b == 0; entry, b(b: Bool): b != 0; a(b: Bool), end: ; b(b: Bool), end: ;"
    );

    test_transform!(
        calculate_disjoints,
        if_else_comparison_binding_outer,
        "begin, entry(b: Bool): ; entry(b: Bool), a(b: Bool): b == 0; entry(b: Bool), b(b: Bool): b != 0; a(b: Bool), end: ; b(b: Bool), end: ;",
        adds "@disjointExhaustive entry(b: Bool) : a(b: Bool) b(b: Bool);"
    );

    test_transform!(
        calculate_disjoints,
        breakthrough,
        include_str!("../../../../../games/rg/breakthrough.rg"),
        adds "@disjointExhaustive wincheck : continue win; @disjointExhaustive turn : lose move;"
    );

    test_transform!(
        calculate_disjoints,
        tictactoe,
        include_str!("../../../../../games/rg/ticTacToe.rg"),
        adds "@disjointExhaustive checkwin : nextturn win; @disjointExhaustive turn : move preend;"
    );

    test_transform!(
        calculate_disjoints,
        simple_apply_test_5,
        include_str!("../../../../../games/rg/simpleApplyTest5.rg"),
        adds "@disjointExhaustive readKey : readOne readZero;"
    );

    test_transform!(
        calculate_disjoints,
        simple_apply_test_6,
        include_str!("../../../../../games/rg/simpleApplyTest6.rg"),
        adds "@disjoint readKey : readOne readZero; @disjointExhaustive readHidden : draw readDone win;"
    );
}
