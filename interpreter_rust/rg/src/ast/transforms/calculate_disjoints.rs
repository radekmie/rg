use crate::ast::{Edge, Error, Game, Label, Node, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;

impl Game<Id> {
    pub fn calculate_disjoints(&mut self) -> Result<(), Error<Id>> {
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

            let edges: Vec<_> = edges.into_iter().collect();
            if let Some((is_exhaustive, mut nodes)) = game.get_disjoint(&edges) {
                nodes.sort_unstable();
                let node = node.clone();
                let span = Span::none();
                pragmas.push(if is_exhaustive {
                    Pragma::DisjointExhaustive { span, node, nodes }
                } else {
                    Pragma::Disjoint { span, node, nodes }
                });
            }
        }

        for pragma in pragmas {
            self.add_pragma(pragma);
        }

        Ok(())
    }

    fn get_disjoint(&self, edges: &[&Arc<Edge<Id>>]) -> Option<(bool, Vec<Node<Id>>)> {
        self.get_disjoint_switch(edges)
            .or_else(|| self.get_disjoint_if_else(edges))
    }

    fn get_disjoint_if_else(&self, edges: &[&Arc<Edge<Id>>]) -> Option<(bool, Vec<Node<Id>>)> {
        // Fast-path for two edges.
        if let [x, y] = edges {
            let is_disjoint = x.rhs != y.rhs && x.label.is_negated(&y.label);
            return is_disjoint.then(|| (true, vec![x.rhs.clone(), y.rhs.clone()]));
        }

        let labels_by_rhs: BTreeMap<_, Vec<_>> =
            edges
                .iter()
                .fold(BTreeMap::new(), |mut edges_by_rhs, edge| {
                    edges_by_rhs.entry(&edge.rhs).or_default().push(&edge.label);
                    edges_by_rhs
                });

        for (a, xs) in &labels_by_rhs {
            for (b, ys) in &labels_by_rhs {
                if a != b
                    && xs.iter().all(|x| ys.iter().any(|y| x.is_negated(y)))
                    && ys.iter().all(|x| xs.iter().any(|y| x.is_negated(y)))
                {
                    return Some((true, vec![(*a).clone(), (*b).clone()]));
                }
            }
        }

        None
    }

    fn get_disjoint_switch(&self, edges: &[&Arc<Edge<Id>>]) -> Option<(bool, Vec<Node<Id>>)> {
        let mut disjoints = vec![];
        for index1 in (0..edges.len()).rev() {
            let Label::Comparison {
                lhs,
                rhs,
                negated: false,
            } = &edges[index1].label
            else {
                continue;
            };

            let Some(identifier) = rhs.uncast().as_reference() else {
                continue;
            };

            let lhs1 = lhs.uncast();
            let mut nodes = vec![edges[index1].rhs.clone()];
            let mut symbols = BTreeSet::from([identifier]);
            for index2 in (0..edges.len()).rev() {
                if index1 == index2 {
                    continue;
                }

                if let Label::Comparison {
                    lhs: lhs2,
                    rhs,
                    negated: false,
                } = &edges[index2].label
                {
                    if lhs1 == lhs2.uncast() {
                        if let Some(identifier) = rhs.uncast().as_reference() {
                            if symbols.insert(identifier) {
                                nodes.push(edges[index2].rhs.clone());
                                continue;
                            }
                        }
                    }
                }
            }

            if nodes.len() == 1 {
                continue;
            }

            let all_values = lhs1
                .infer(self)
                .and_then(|type_| type_.values(self))
                .is_ok_and(|values| values.len() == nodes.len());

            disjoints.push((all_values, nodes));
        }

        // Select the best one.
        disjoints
            .into_iter()
            .max_by_key(|(is_exhaustive, nodes)| (nodes.len(), *is_exhaustive))
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
        if_else_parallel,
        "begin, a: ? x1 -> y1; begin, b: ! x1 -> y1; begin, b: ? x2 -> y2; a, end: ; b, end: ; x1, y1: ; x2, y2: ;"
    );

    test_transform!(
        calculate_disjoints,
        if_else_parallel_double,
        "begin, a: ? x1 -> y1; begin, a: ! x2 -> y2; begin, b: ! x1 -> y1; begin, b: ? x2 -> y2; a, end: ; b, end: ; x1, y1: ; x2, y2: ;",
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
        simple_apply_test_1,
        include_str!("../../../../../games/rg/simpleApplyTest1.rg"),
        adds "@disjointExhaustive moveB : tagB0same tagB1same;"
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
