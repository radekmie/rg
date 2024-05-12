use crate::ast::{Edge, Error, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn inline_reachability(&mut self) -> Result<(), Error<Id>> {
        for edge in self.edges.clone() {
            if let Label::Reachability { lhs, rhs, .. } = &edge.label {
                if let Some(subgraph) = self.find_acceptable_paths(lhs, rhs) {
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
            let mut nodes = self
                .nodes()
                .iter()
                .map(|n| (*n).clone())
                .collect::<BTreeSet<_>>();

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

#[allow(clippy::needless_pass_by_value)]
fn gen_fresh_node(node: String, nodes: &BTreeSet<Node<Id>>) -> Node<Id> {
    for x in 1..nodes.len() {
        let fresh_node: Node<Id> = Node::new(Id::from(format!("__gen_{x}_{node}")));
        if !nodes.contains(&fresh_node) {
            return fresh_node;
        }
    }
    let name = format!("__gen_{}_{node}", nodes.len());
    Node::new(Id::from(name))
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
    use crate::ast::transforms::inline_reachability::Id;
    use crate::ast::Game;
    use crate::parsing::parser::parse_with_errors;
    use map_id::MapId;
    use std::sync::Arc;

    fn parse(input: &str) -> Game<Arc<str>> {
        let (game, errors) = parse_with_errors(input);
        assert!(errors.is_empty(), "Parse errors: {errors:?}");
        game.map_id(&mut |id| Id::from(id.identifier.as_str()))
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual);
                let expect = parse($expect);
                actual.inline_reachability().unwrap();

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    macro_rules! no_changes {
        ($name:ident, $actual:expr) => {
            test!($name, $actual, $actual);
        };
    }

    test!(
        basic1,
        "a, b: ? x -> y;
        x, y: 1 == 1;",
        "x, y: 1 == 1;
        a, __gen_1_reachability_x_y: ;
        __gen_1_reachability_x_y, b: 1 == 1;"
    );

    test!(
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

    test!(
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

    test!(
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

    test!(
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

    no_changes!(
        non_exclusive_comparison,
        "type T = {1, 2};
        var v: T = 1;
        x, y: ? a -> d;
        a, b: v == 1;
        a, c: v != 2;
        b, d: ;
        c, d: ;"
    );

    test!(
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

    test!(
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

    no_changes!(
        non_exclusive_reachability,
        "x, y: ? a -> d;
        a, b: ? e -> f;
        a, c: ! e -> g;
        b, d: ;
        c, d: ;
        e, f: ;
        e, g: ;"
    );

    test!(
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

    no_changes!(
        reachability_with_generator,
        "type T = { null };
        begin, end: ? a -> c;
        a, b(t: T): t == null;
        b(t: T), c: t == null;"
    );
}
