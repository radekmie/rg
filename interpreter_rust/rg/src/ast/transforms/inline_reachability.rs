use super::gen_fresh_node;
use crate::ast::{Edge, Error, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn inline_reachability(&mut self) -> Result<(), Error<Id>> {
        for edge in self.edges.clone() {
            if let Label::Reachability {
                lhs, rhs, negated, ..
            } = &edge.label
            {
                if let Some(subgraph) = self.find_rechability_paths(lhs, rhs, *negated) {
                    self.substitute_reachability(edge.clone(), subgraph);
                }
            }
        }
        Ok(())
    }

    /// Return a subautomaton of [edges] that:
    /// 1. contains [start] and [target]
    /// 2. for any node except [target] contains all outgoing nodes
    /// 3. contains no edges from [target]
    /// 4. for any initial environment, at most one path can reach [target] from [start]
    ///    - limited analysis, may reject some valid results here
    /// 4.1. and none of them change the environment (currently: no assignments allowed)
    /// 5. If the reachability is negated, the path consists of one edge
    fn find_rechability_paths(
        &self,
        start: &Node<Id>,
        target: &Node<Id>,
        negated: bool,
    ) -> Option<BTreeSet<Edge<Id>>> {
        if negated {
            let mut direct_path = self
                .edges
                .iter()
                .filter(|e| e.lhs == *start && e.rhs == *target && !e.label.is_assignment());
            let edge = direct_path
                .next()
                .filter(|_| direct_path.next().is_none())?;
            Some(BTreeSet::from_iter(iter::once(edge.clone())))
        } else {
            self.find_acceptable_paths(start, target)
        }
    }

    fn find_acceptable_paths(
        &self,
        start: &Node<Id>,
        target: &Node<Id>,
    ) -> Option<BTreeSet<Edge<Id>>> {
        let next_edges = self.next_edges();
        let mut queue = vec![(start, BTreeSet::new())];
        let mut result = BTreeSet::new();
        while let Some((lhs, mut previous)) = queue.pop() {
            previous.insert(lhs);
            if let Some(edges) = next_edges.get(&lhs) {
                if !are_edges_exclusive(edges) {
                    return None; // multiple paths found
                }
                for edge in edges {
                    if edge.has_bindings()
                        || previous.contains(&edge.rhs)
                        || edge.label.is_assignment()
                    {
                        return None;
                    }
                    result.insert((*edge).clone());
                    if edge.rhs != *target {
                        queue.push((&edge.rhs, previous.clone()));
                    }
                }
            }
        }
        Some(result)
    }

    fn substitute_reachability(&mut self, mut edge: Edge<Id>, subgraph: BTreeSet<Edge<Id>>) {
        if let Label::Reachability {
            lhs: start,
            rhs: target,
            negated,
            ..
        } = edge.label.clone()
        {
            let mut nodes: BTreeSet<_> = self.nodes().iter().map(|n| (*n).clone()).collect();

            self.remove_edge(&edge);
            let new_start = gen_fresh_node(format!("reachability_{start}_{target}"), &nodes);
            nodes.insert(new_start.clone());

            let mut mapping = BTreeMap::new();
            mapping.insert(start, new_start.clone());
            mapping.insert(target.clone(), edge.rhs);

            edge.rhs = new_start;
            edge.skip();
            self.add_edge(edge);

            for mut edge in subgraph {
                if let Some(lhs) = mapping.get(&edge.lhs) {
                    edge.lhs = lhs.clone();
                } else {
                    let lhs = gen_fresh_node(edge.lhs.to_string(), &nodes);
                    nodes.insert(lhs.clone());
                    mapping.insert(edge.lhs.clone(), lhs.clone());
                    edge.lhs = lhs;
                }

                if let Some(rhs) = mapping.get(&edge.rhs) {
                    edge.rhs = rhs.clone();
                } else {
                    let rhs = gen_fresh_node(edge.rhs.to_string(), &nodes);
                    nodes.insert(rhs.clone());
                    mapping.insert(edge.rhs.clone(), rhs.clone());
                    edge.rhs = rhs;
                }
                if negated {
                    edge.label.negate();
                }
                self.add_edge(edge);
            }
        }
    }
}

fn are_edges_exclusive(edges: &BTreeSet<&Edge<Id>>) -> bool {
    for edge in edges {
        for other in edges {
            if edge != other && !edge.is_exclusive_with(other) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        inline_reachability,
        basic1,
        "a, b: ? x -> y;
        x, y: 1 == 1;",
        "x, y: 1 == 1;
        a, __gen_1_reachability_x_y: ;
        __gen_1_reachability_x_y, b: 1 == 1;"
    );

    test_transform!(
        inline_reachability,
        basic2,
        "a, b: ? x -> z;
        x, y: 1 == 1;
        y, z: 2 == 2;",
        "x, y: 1 == 1;
        y, z: 2 == 2;
        a, __gen_1_reachability_x_z: ;
        __gen_1_reachability_x_z, __gen_1_y: 1 == 1;
        __gen_1_y, b: 2 == 2;"
    );

    test_transform!(
        inline_reachability,
        basic3,
        "a, b: ? x -> z;
        x, y: ;
        y, z: 2 == 2;",
        "x, y: ;
        y, z: 2 == 2;
        a, __gen_1_reachability_x_z: ;
        __gen_1_y, b: 2 == 2;
        __gen_1_reachability_x_z, __gen_1_y: ;"
    );

    test_transform!(
        inline_reachability,
        basic4,
        "a, b: ? x -> z;
        x, y: 1 == 1;
        y, z: ;",
        "x, y: 1 == 1;
        y, z: ;
        a, __gen_1_reachability_x_z: ;
        __gen_1_reachability_x_z, __gen_1_y: 1 == 1;
        __gen_1_y, b: ;"
    );

    test_transform!(
        inline_reachability,
        exclusive_comparison,
        "x, y: ? a -> d;
        a, b: 1 == 1;
        a, c: 1 != 1;
        b, d: ;
        c, d: ;",
        "a, b: 1 == 1;
        a, c: 1 != 1;
        b, d: ;
        c, d: ;
        x, __gen_1_reachability_a_d: ;
        __gen_1_reachability_a_d, __gen_1_b: 1 == 1;
        __gen_1_reachability_a_d, __gen_1_c: 1 != 1;
        __gen_1_b, y: ;
        __gen_1_c, y: ;"
    );

    test_transform!(
        inline_reachability,
        non_exclusive_comparison,
        "type T = {1, 2};
        var v: T = 1;
        x, y: ? a -> d;
        a, b: v == 1;
        a, c: v != 2;
        b, d: ;
        c, d: ;"
    );

    test_transform!(
        inline_reachability,
        exclusive_reachability_step,
        "x, y: ? a -> d;
        a, b: ? e -> f;
        a, c: ! e -> f;
        b, d: ;
        c, d: ;
        e, f: ;",
        "b, d: ;
        c, d: ;
        e, f: ;
        x, __gen_1_reachability_a_d: ;
        __gen_1_reachability_a_d, __gen_1_b: ? e -> f;
        __gen_1_reachability_a_d, __gen_1_c: ! e -> f;
        __gen_1_b, y: ;
        __gen_1_c, y: ;
        a, __gen_1_reachability_e_f: ;
        __gen_1_reachability_e_f, b: ;
        a, __gen_2_reachability_e_f: ;
        __gen_2_reachability_e_f, c: ;"
    );

    test_transform!(
        inline_reachability,
        exclusive_reachability_step2,
        "b, d: ;
        c, d: ;
        e, f: ;
        x, __gen_1_reachability_a_d: ;
        __gen_1_reachability_a_d, __gen_1_b: ? e -> f;
        __gen_1_reachability_a_d, __gen_1_c: ! e -> f;
        __gen_1_b, y: ;
        __gen_1_c, y: ;
        a, __gen_1_reachability_e_f: ;
        __gen_1_reachability_e_f, b: ;
        a, __gen_2_reachability_e_f: ;
        __gen_2_reachability_e_f, c: ;",
        "b, d: ;
        c, d: ;
        e, f: ;
        x, __gen_1_reachability_a_d: ;
        __gen_1_b, y: ;
        __gen_1_c, y: ;
        a, __gen_1_reachability_e_f: ;
        __gen_1_reachability_e_f, b: ;
        a, __gen_2_reachability_e_f: ;
        __gen_2_reachability_e_f, c: ;
        __gen_1_reachability_a_d, __gen_3_reachability_e_f: ;
        __gen_3_reachability_e_f, __gen_1_b: ;
        __gen_1_reachability_a_d, __gen_4_reachability_e_f: ;
        __gen_4_reachability_e_f, __gen_1_c: ;"
    );

    test_transform!(
        inline_reachability,
        non_exclusive_reachability,
        "x, y: ? a -> d;
        a, b: ? e -> f;
        a, c: ! e -> g;
        b, d: ;
        c, d: ;
        e, f: ;
        e, g: ;",
        "x, y: ? a -> d;
        a, b: ? e -> f;
        b, d: ;
        c, d: ;
        e, f: ;
        e, g: ;
        a, __gen_1_reachability_e_g: ;
        __gen_1_reachability_e_g, c: ;"
    );

    test_transform!(
        inline_reachability,
        dont_copy_trailing_edges,
        "x, y: ? a -> c;
        a, b: 0 == 0;
        b, c: 1 == 1;
        c, d: 2 == 2;",
        "a, b: 0 == 0;
        b, c: 1 == 1;
        c, d: 2 == 2;
        x, __gen_1_reachability_a_c: ;
        __gen_1_reachability_a_c, __gen_1_b: 0 == 0;
        __gen_1_b, y: 1 == 1;"
    );

    test_transform!(
        inline_reachability,
        reachability_with_generator,
        "type T = { null };
        begin, end: ? a -> c;
        a, b(t: T): t == null;
        b(t: T), c: t == null;"
    );

    test_transform!(
        inline_reachability,
        negated,
        "a, b: ! x -> y;
        x, y: 1 == 1;",
        "x, y: 1 == 1;
        a, __gen_1_reachability_x_y: ;
        __gen_1_reachability_x_y, b: 1 != 1;"
    );

    test_transform!(
        inline_reachability,
        long_negated,
        "a, b: ! x -> y;
        x, x1: 1 == 1;
        x1, y: ;"
    );
}
