mod reachable_nodes;
mod reaching_definitions;
mod reaching_paths;

use crate::ast::{Edge, Game, Label, Node};
pub use reachable_nodes::ReachableNodes;
pub use reaching_definitions::ReachingDefinitions;
pub use reaching_paths::ReachingPaths;
use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::Arc;

type Id = Arc<str>;

pub trait Analysis {
    type Domain: Clone + PartialEq;

    fn bot() -> Self::Domain;

    fn extreme(program: &Game<Id>) -> Self::Domain;

    fn gen(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain;

    fn join(a: Self::Domain, b: Self::Domain) -> Self::Domain;

    fn kill(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain;

    fn transfer(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        Self::gen(Self::kill(input, edge), edge)
    }
}

impl Game<Id> {
    pub fn analyse<A: Analysis>(&self, with_reachability: bool) -> BTreeMap<Node<Id>, A::Domain> {
        let flow = Flow::new(self, with_reachability);
        let mut worker = Worker::<A>::new(self, &flow);
        worker.run();
        worker.result
    }
}

enum Flow<'a> {
    Forward {
        nodes: BTreeSet<&'a Node<Id>>,
        prev_edges: BTreeMap<&'a Node<Id>, BTreeSet<&'a Edge<Id>>>,
    },
    ForwardReachability {
        nodes: BTreeSet<&'a Node<Id>>,
        prev_edges: BTreeMap<&'a Node<Id>, BTreeSet<&'a Edge<Id>>>,
        reachability_edges: BTreeMap<&'a Node<Id>, BTreeSet<Edge<Id>>>,
    },
}

impl<'a> Flow<'a> {
    fn entry(&self) -> Node<Id> {
        Node::new(Id::from("begin"))
    }

    fn new(game: &'a Game<Arc<str>>, with_reachability: bool) -> Self {
        if with_reachability {
            let mut reachability_edges = BTreeMap::new();
            for edge in &game.edges {
                if let Label::Reachability { lhs, .. } = &edge.label {
                    // Create a `fake` edge simulating start of reachability check
                    // node0, node1: ? start -> target;
                    // node0 is predecessor of start
                    let skip_edge = Edge::new_skip(edge.lhs.clone(), lhs.clone());
                    reachability_edges
                        .entry(lhs)
                        .or_insert_with(BTreeSet::new)
                        .insert(skip_edge);
                }
            }
            Self::ForwardReachability {
                nodes: game.nodes(),
                prev_edges: game.prev_edges(),
                reachability_edges,
            }
        } else {
            Self::Forward {
                nodes: game.nodes(),
                prev_edges: game.prev_edges(),
            }
        }
    }

    fn predecessors(&self, node: &Node<Id>) -> BTreeSet<&Edge<Id>> {
        match self {
            Flow::Forward { prev_edges, .. } => prev_edges
                .get(node)
                .map_or(BTreeSet::new(), |edges| (*edges).clone()),
            Flow::ForwardReachability {
                prev_edges,
                reachability_edges,
                ..
            } => {
                let mut result = BTreeSet::new();
                result.extend(prev_edges.get(node).into_iter().flatten());
                result.extend(reachability_edges.get(node).into_iter().flatten());
                result
            }
        }
    }

    fn nodes(&self) -> &BTreeSet<&Node<Id>> {
        match self {
            Flow::Forward { nodes, .. } => nodes,
            Flow::ForwardReachability { nodes, .. } => nodes,
        }
    }
}

struct Worker<'a, A: Analysis + ?Sized> {
    flow: &'a Flow<'a>,
    result: BTreeMap<Node<Id>, A::Domain>,
    _parameters: PhantomData<A>,
}

impl<'a, I: Analysis + ?Sized> Worker<'a, I> {
    fn knowledge(&self, node: &Node<Id>) -> I::Domain {
        self.result.get(node).cloned().unwrap_or_else(I::bot)
    }

    fn new(game: &'a Game<Arc<str>>, flow: &'a Flow<'a>) -> Self {
        Worker {
            flow,
            result: BTreeMap::from([(flow.entry(), I::extreme(game))]),
            _parameters: PhantomData,
        }
    }

    fn run(&mut self) {
        while self.step() {}
    }

    fn step(&mut self) -> bool {
        let mut changed = false;
        for node in self.flow.nodes() {
            if self.transfer(node) {
                changed = true;
            }
        }
        changed
    }

    fn summarize_predecessors(&self, node: &Node<Id>, old_input: &I::Domain) -> I::Domain {
        let incoming_edges = self.flow.predecessors(node);
        if incoming_edges.is_empty() {
            return old_input.clone();
        }

        incoming_edges
            .iter()
            .map(|edge| I::transfer(self.knowledge(&edge.lhs), edge))
            .fold(I::bot(), I::join)
    }

    fn transfer(&mut self, node: &Node<Id>) -> bool {
        let old_kw = self.knowledge(node);
        let new_kw = self.summarize_predecessors(node, &old_kw);
        let changed = old_kw != new_kw;
        self.result.insert((*node).clone(), new_kw);
        changed
    }
}
