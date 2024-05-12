use crate::ast::{Edge, Error, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

/**
 * Return a subautomaton of [edges] that:
 * 1. contains [start] and [target]
 * 2. for any node except [target] contains all outgoing nodes
 * 3. contains no edges from [target]
 * 4. for any initial environment, at most one path can reach [target] from [start]
 *    - limited analysis, may reject some valid results here
 * 4.1. and none of them change the environment (currently: no assignments allowed)
 *
 * @param {ast.EdgeDeclaration[]} edges - considered automaton
 * @param {ast.EdgeName} start - origin of the search
 * @param {ast.EdgeName} target - target of the search
 * @returns {Result<ast.EdgeDeclaration[], string>} Subautomaton of 'edges' or an error
 */
impl Game<Id> {
    pub fn inline_reachability(&mut self) -> Result<(), Error<Id>> {
        for edge in self.edges.clone() {
            if let Label::Reachability { lhs, rhs, .. } = &edge.label {
                if let Some(subgraph) = self.find_acceptable_paths(lhs, rhs) {
                    dbg!(&edge);
                    self.substitute_reachability(edge.clone(), subgraph);
                }
            }
        }
        Ok(())
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
            self.remove_edge(&edge);
            let new_start = gen_fresh_node(format!("rechability_{start}_{target}"), self.nodes());

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
                    let lhs = gen_fresh_node(edge.lhs.to_string(), self.nodes());
                    mapping.insert(edge.lhs.clone(), lhs.clone());
                    edge.lhs = lhs;
                }

                if let Some(rhs) = mapping.get(&edge.rhs) {
                    edge.rhs = rhs.clone();
                } else {
                    let rhs = gen_fresh_node(edge.rhs.to_string(), self.nodes());
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
fn gen_fresh_node(node: String, nodes: BTreeSet<&Node<Id>>) -> Node<Id> {
    for x in 0..nodes.len() {
        let fresh_node: Node<Id> = Node::new(Id::from(format!("__gen_{node}_{x}")));
        if !nodes.contains(&fresh_node) {
            return fresh_node;
        }
    }
    let name = format!("__gen_{node}_{}", nodes.len());
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

    no_changes!(
        reachability_with_generator,
        "type T = { null };
        begin, end: ? a -> c;
        a, b(t: T): t == null;
        b(t: T), c: t == null;"
    );
}
