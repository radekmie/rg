use super::flow::Flow;
use crate::ast::{Edge, Game, Node};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

type Id = Arc<str>;

pub trait Analysis {
    type Domain: Clone + PartialEq;

    fn analyse(&self, flow: &Flow, game: &Game<Id>) -> BTreeMap<Node<Id>, Self::Domain> {
        let mut worker = Worker::<Self>::new(flow, game);
        worker.run();
        worker.result
    }

    fn bot() -> Self::Domain;

    fn extreme(program: &Game<Id>) -> Self::Domain;

    fn gen(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain;

    fn join(a: Self::Domain, b: Self::Domain) -> Self::Domain;

    fn kill(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain;

    fn transfer(input: Self::Domain, edge: &Edge<Id>) -> Self::Domain {
        Self::gen(Self::kill(input, edge), edge)
    }
}

struct Worker<'a, I: Analysis + ?Sized> {
    flow: &'a Flow<'a>,
    result: BTreeMap<Node<Id>, I::Domain>,
    _parameters: PhantomData<I>,
}

impl<'a, I: Analysis + ?Sized> Worker<'a, I> {
    fn new(flow: &'a Flow<'a>, game: &Game<Id>) -> Self {
        Worker {
            flow,
            result: BTreeMap::from([(flow.entry(), I::extreme(game))]),
            _parameters: PhantomData,
        }
    }

    fn knowledge(&self, node: &Node<Id>) -> I::Domain {
        self.result.get(node).cloned().unwrap_or_else(I::bot)
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

    fn step(&mut self) -> bool {
        let mut changed = false;
        for node in &self.flow.nodes {
            if self.transfer(node) {
                changed = true;
            }
        }
        changed
    }

    fn run(&mut self) {
        while self.step() {}
    }
}
