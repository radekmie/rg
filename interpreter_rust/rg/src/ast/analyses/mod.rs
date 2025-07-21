mod constants_analysis;
mod reachable_nodes;
mod reaching_assignments;
mod reaching_definitions;
mod value_sets;

use crate::ast::{Edge, Game, Label, Node};
pub use constants_analysis::ConstantsAnalysis;
pub use reachable_nodes::ReachableNodes;
pub use reaching_assignments::ReachingAssignments;
pub use reaching_definitions::{Definition, ReachingDefinitions};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

pub trait Analysis {
    type Domain: Clone + PartialEq;

    fn bot(&self) -> Self::Domain;

    fn extreme(&self, program: &Game<Id>) -> Self::Domain;

    fn gen(&self, input: Self::Domain, _edge: &Arc<Edge<Id>>) -> Self::Domain {
        input
    }

    fn join(&self, a: Self::Domain, b: Self::Domain) -> Self::Domain;

    fn kill(&self, input: Self::Domain, _edge: &Arc<Edge<Id>>) -> Self::Domain {
        input
    }

    fn transfer(&self, input: Self::Domain, edge: &Arc<Edge<Id>>) -> Self::Domain {
        self.gen(self.kill(input, edge), edge)
    }

    fn with_reachability(&self) -> bool {
        false
    }
}

impl Game<Id> {
    pub fn analyse<A: Analysis>(&self, analysis: &A) -> BTreeMap<Node<Id>, A::Domain> {
        let flow = Flow::new(self, analysis.with_reachability());
        let mut worker = Worker::new(self, analysis, &flow);
        worker.run();
        worker.result
    }
}

struct Flow<'a> {
    next_nodes: BTreeMap<&'a Node<Id>, BTreeSet<&'a Node<Id>>>,
    nodes: BTreeSet<&'a Node<Id>>,
    prev_edges: BTreeMap<&'a Node<Id>, BTreeSet<&'a Arc<Edge<Id>>>>,
    #[expect(clippy::type_complexity)]
    reachability_edges: Option<BTreeMap<&'a Node<Id>, BTreeSet<Arc<Edge<Id>>>>>,
}

impl<'a> Flow<'a> {
    fn entry() -> Node<Id> {
        Node::new(Id::from("begin"))
    }

    fn new(game: &'a Game<Arc<str>>, with_reachability: bool) -> Self {
        let mut next_nodes = game.next_nodes();

        let reachability_edges = with_reachability.then(|| {
            let mut reachability_edges: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
            for edge in &game.edges {
                if let Label::Reachability { lhs, .. } = &edge.label {
                    // Create a `fake` edge simulating start of reachability check
                    // node0, node1: ? start -> target;
                    // node0 is predecessor of start
                    let skip_edge = Edge::new_skip(edge.lhs.clone(), lhs.clone());
                    reachability_edges
                        .entry(lhs)
                        .or_default()
                        .insert(Arc::from(skip_edge));
                    next_nodes.entry(&edge.lhs).or_default().insert(lhs);
                }
            }

            reachability_edges
        });

        Self {
            next_nodes,
            nodes: game.nodes(),
            prev_edges: game.prev_edges(),
            reachability_edges,
        }
    }

    fn predecessors(&self, node: &Node<Id>) -> BTreeSet<&Arc<Edge<Id>>> {
        let mut result = BTreeSet::new();
        result.extend(self.prev_edges.get(node).into_iter().flatten());
        if let Some(reachability_edges) = &self.reachability_edges {
            result.extend(reachability_edges.get(node).into_iter().flatten());
        }
        result
    }
}

struct Worker<'a, A: Analysis> {
    analysis: &'a A,
    flow: &'a Flow<'a>,
    result: BTreeMap<Node<Id>, A::Domain>,
}

impl<'a, A: Analysis> Worker<'a, A> {
    fn knowledge(&self, node: &Node<Id>) -> A::Domain {
        self.result
            .get(node)
            .cloned()
            .unwrap_or_else(|| self.analysis.bot())
    }

    fn new(game: &'a Game<Arc<str>>, analysis: &'a A, flow: &'a Flow<'a>) -> Self {
        let result = BTreeMap::from([(Flow::entry(), analysis.extreme(game))]);
        Worker {
            analysis,
            flow,
            result,
        }
    }

    fn run(&mut self) {
        let mut worklist = self.flow.nodes.clone();
        while let Some(node) = worklist.pop_first() {
            if self.transfer(node) {
                worklist.extend(self.flow.next_nodes.get(node).into_iter().flatten());
            }
        }
    }

    fn summarize_predecessors(&self, node: &Node<Id>, old_input: &A::Domain) -> A::Domain {
        self.flow
            .predecessors(node)
            .iter()
            .map(|edge| self.analysis.transfer(self.knowledge(&edge.lhs), edge))
            .reduce(|a, b| self.analysis.join(a, b))
            .unwrap_or_else(|| old_input.clone())
    }

    fn transfer(&mut self, node: &Node<Id>) -> bool {
        let old_kw = self.knowledge(node);
        let new_kw = self.summarize_predecessors(node, &old_kw);
        let changed = old_kw != new_kw;
        self.result.insert((*node).clone(), new_kw);
        changed
    }
}

#[cfg(test)]
impl Game<Id> {
    #[allow(clippy::type_complexity)]
    pub fn test_analysis<A: Analysis>(
        source: &str,
        expect: &str,
        analysis: Box<dyn FnOnce(&Self) -> A>,
        formatter: Box<dyn FnOnce(BTreeMap<Node<Id>, A::Domain>) -> String>,
    ) {
        let game = Self::test_parse_or_fail(source);
        let actual = formatter(game.analyse(&analysis(&game)));
        let actual = actual.trim();
        let expect = expect.trim();
        assert!(
            actual == expect,
            "\n\n>>> Actual: <<<\n        {actual}\n>>> Expect: <<<\n        {expect}\n"
        );
    }
}
