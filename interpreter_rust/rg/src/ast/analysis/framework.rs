use super::flow::Flow;
use crate::ast::{Edge, Game, Node};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

type Id = Arc<str>;

pub trait Parameters<Domain: Clone> {
    fn bot() -> Domain;
    fn extreme(program: &Game<Id>) -> Domain;
    fn gen(input: Domain, edge: &Edge<Id>) -> Domain;
    fn join(a: Domain, b: Domain) -> Domain;
    fn kill(input: Domain, edge: &Edge<Id>) -> Domain;
    fn transfer(input: Domain, edge: &Edge<Id>) -> Domain {
        Self::gen(Self::kill(input, edge), edge)
    }
}

pub trait Instance<Domain: Clone + PartialEq, P: Parameters<Domain>> {
    fn analyse(&self, flow: &Flow, game: &Game<Id>) -> BTreeMap<Node<Id>, Domain> {
        Worker::<Domain, P>::analyse(flow, game)
    }
}

struct Worker<'a, Domain: Clone, P: Parameters<Domain>> {
    cfg: &'a Flow<'a>,
    result: BTreeMap<Node<Id>, Domain>,
    _parameters: PhantomData<P>,
}

impl<'a, Domain: Clone + PartialEq, P: Parameters<Domain>> Worker<'a, Domain, P> {
    fn new(cfg: &'a Flow<'a>, game: &Game<Id>) -> Self {
        Worker {
            cfg,
            result: BTreeMap::from([(cfg.entry(), P::extreme(game))]),
            _parameters: PhantomData,
        }
    }

    fn knowledge(&self, node: &Node<Id>) -> Domain {
        self.result.get(node).cloned().unwrap_or_else(P::bot)
    }

    fn summarize_predecessors(&self, node: &Node<Id>, old_input: &Domain) -> Domain {
        let incoming_edges = self.cfg.predecessors(node);
        if incoming_edges.is_empty() {
            return old_input.clone();
        }

        incoming_edges
            .iter()
            .map(|edge| P::transfer(self.knowledge(&edge.lhs), edge))
            .fold(P::bot(), P::join)
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
        for node in &self.cfg.nodes {
            if self.transfer(node) {
                changed = true;
            }
        }
        changed
    }

    fn run(&mut self) {
        while self.step() {}
    }

    pub fn analyse(cfg: &'a Flow<'a>, game: &Game<Id>) -> BTreeMap<Node<Id>, Domain> {
        let mut worker = Self::new(cfg, game);
        worker.run();
        worker.result
    }
}
