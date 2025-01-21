mod constants_analysis;
mod reachable_nodes;
mod reaching_assignments;
mod reaching_binding_assignments;
mod reaching_definitions;

use crate::ast::{Edge, Game, Label, Node};
pub use constants_analysis::ConstantsAnalysis;
pub use reachable_nodes::ReachableNodes;
pub use reaching_assignments::ReachingAssignments;
pub use reaching_binding_assignments::ReachingBindingAssignments;
pub use reaching_definitions::ReachingDefinitions;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

pub trait Analysis {
    type Context;
    type Domain: Clone + PartialEq;

    fn bot() -> Self::Domain;

    fn extreme(program: &Game<Id>, ctx: &Self::Context) -> Self::Domain;

    fn gen(input: Self::Domain, _edge: &Arc<Edge<Id>>, _ctx: &Self::Context) -> Self::Domain {
        input
    }

    fn get_context(_program: &Game<Id>) -> Self::Context;

    fn join(a: Self::Domain, b: Self::Domain, ctx: &Self::Context) -> Self::Domain;

    fn kill(input: Self::Domain, _edge: &Arc<Edge<Id>>, _ctx: &Self::Context) -> Self::Domain {
        input
    }

    fn transfer(input: Self::Domain, edge: &Arc<Edge<Id>>, ctx: &Self::Context) -> Self::Domain {
        Self::gen(Self::kill(input, edge, ctx), edge, ctx)
    }
}

impl Game<Id> {
    pub fn analyse<A: Analysis>(&self, with_reachability: bool) -> BTreeMap<Node<Id>, A::Domain> {
        self.analyse_with_context::<A>(with_reachability).0
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

struct Worker<'a, A: Analysis + ?Sized> {
    ctx: A::Context,
    flow: &'a Flow<'a>,
    result: BTreeMap<Node<Id>, A::Domain>,
}

impl<'a, A: Analysis + ?Sized> Worker<'a, A> {
    fn knowledge(&self, node: &Node<Id>) -> A::Domain {
        self.result.get(node).cloned().unwrap_or_else(A::bot)
    }

    fn new(game: &'a Game<Arc<str>>, flow: &'a Flow<'a>) -> Self {
        let ctx = A::get_context(game);
        let result = BTreeMap::from([(Flow::entry(), A::extreme(game, &ctx))]);
        Worker { ctx, flow, result }
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
            .map(|edge| A::transfer(self.knowledge(&edge.lhs), edge, &self.ctx))
            .reduce(|a, b| A::join(a, b, &self.ctx))
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
