mod constants_analysis;
mod reachable_nodes;
mod reaching_assignments;
mod reaching_definitions;

use crate::ast::{Edge, Game, Label, Node};
pub use constants_analysis::ConstantsAnalysis;
pub use reachable_nodes::ReachableNodes;
pub use reaching_assignments::ReachingAssignments;
pub use reaching_definitions::ReachingDefinitions;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

pub trait Analysis {
    type Domain: Clone + PartialEq;
    type Context: PartialEq + Default;

    fn bot() -> Self::Domain;

    fn extreme(program: &Game<Id>, ctx: &Self::Context) -> Self::Domain;

    fn gen(input: Self::Domain, _edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        input
    }

    fn get_context(_program: &Game<Id>) -> Self::Context {
        Self::Context::default()
    }

    fn join(a: Self::Domain, b: Self::Domain) -> Self::Domain;

    fn kill(input: Self::Domain, _edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        input
    }

    fn transfer(input: Self::Domain, edge: &Edge<Id>, ctx: &Self::Context) -> Self::Domain {
        Self::gen(Self::kill(input, edge, ctx), edge, ctx)
    }
}

impl Game<Id> {
    pub fn analyse<A: Analysis>(&self, with_reachability: bool) -> BTreeMap<Node<Id>, A::Domain> {
        let flow = Flow::new(self, with_reachability);
        let mut worker = Worker::<A>::new(self, &flow);
        worker.run();
        worker.result
    }

    pub fn analyse_with_context<A: Analysis>(
        &self,
        with_reachability: bool,
    ) -> (BTreeMap<Node<Id>, A::Domain>, A::Context) {
        let flow = Flow::new(self, with_reachability);
        let mut worker = Worker::<A>::new(self, &flow);
        worker.run();
        (worker.result, worker.ctx)
    }
}

struct Flow<'a> {
    next_nodes: BTreeMap<&'a Node<Id>, BTreeSet<&'a Node<Id>>>,
    nodes: BTreeSet<&'a Node<Id>>,
    prev_edges: BTreeMap<&'a Node<Id>, BTreeSet<&'a Edge<Id>>>,
    reachability_edges: Option<BTreeMap<&'a Node<Id>, BTreeSet<Edge<Id>>>>,
}

impl<'a> Flow<'a> {
    fn entry(&self) -> Node<Id> {
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
                    reachability_edges.entry(lhs).or_default().insert(skip_edge);
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

    fn predecessors(&self, node: &Node<Id>) -> BTreeSet<&Edge<Id>> {
        let mut result = BTreeSet::new();
        result.extend(self.prev_edges.get(node).into_iter().flatten());
        if let Some(reachability_edges) = &self.reachability_edges {
            result.extend(reachability_edges.get(node).into_iter().flatten());
        }
        result
    }
}

struct Worker<'a, I: Analysis + ?Sized> {
    flow: &'a Flow<'a>,
    result: BTreeMap<Node<Id>, I::Domain>,
    worklist: BTreeSet<&'a Node<Id>>,
    ctx: I::Context,
}

impl<'a, I: Analysis + ?Sized> Worker<'a, I> {
    fn knowledge(&self, node: &Node<Id>) -> I::Domain {
        self.result.get(node).cloned().unwrap_or_else(I::bot)
    }

    fn new(game: &'a Game<Arc<str>>, flow: &'a Flow<'a>) -> Self {
        let ctx = I::get_context(game);
        Worker {
            flow,
            result: BTreeMap::from([(flow.entry(), I::extreme(game, &ctx))]),
            worklist: flow.nodes.clone(),
            ctx,
        }
    }

    fn run(&mut self) {
        while let Some(node) = self.worklist.pop_first() {
            if self.transfer(node) {
                if let Some(next_nodes) = self.flow.next_nodes.get(node) {
                    self.worklist.extend(next_nodes.iter());
                }
            }
        }
    }

    fn summarize_predecessors(&self, node: &Node<Id>, old_input: &I::Domain) -> I::Domain {
        let incoming_edges = self.flow.predecessors(node);

        incoming_edges
            .iter()
            .map(|edge| I::transfer(self.knowledge(&edge.lhs), edge, &self.ctx))
            .reduce(I::join)
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
