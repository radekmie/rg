use super::flow::Flow;
use crate::ast::{Edge, Game, Node};
use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

type Id = Arc<str>;

pub trait Parameters<Domain: Clone> {
    fn bot() -> Domain;
    fn join(a: Domain, b: Domain) -> Domain;
    fn extreme(program: &Game<Arc<str>>) -> Domain;

    fn kill(input: Domain, edge: &Edge<Id>) -> Domain;
    fn gen(input: Domain, edge: &Edge<Id>) -> Domain;
    fn transfer(input: Domain, edge: &Edge<Id>) -> Domain {
        let input = Self::kill(input, edge);
        Self::gen(input, edge)
    }
}

pub trait Instance<Domain: Clone + PartialEq, P: Parameters<Domain>> {
    fn analyse(&self, flow: &Flow, game: &Game<Arc<str>>) -> BTreeMap<Node<Id>, Domain> {
        Worker::<Domain, P>::analyse(flow, game)
    }
}

struct Worker<'a, Domain: Clone, P: Parameters<Domain>> {
    cfg: &'a Flow<'a>,
    result: BTreeMap<Node<Id>, Domain>,
    _parameters: PhantomData<P>,
}

impl<'a, Domain: Clone + PartialEq, P: Parameters<Domain>> Worker<'a, Domain, P> {
    fn new(cfg: &'a Flow<'a>, game: &Game<Arc<str>>) -> Self {
        let mut result = BTreeMap::new();
        let entry = cfg.entry();
        let extreme = P::extreme(game);
        result.insert(entry, extreme); // initial result-table
        Worker {
            cfg,
            result,
            _parameters: PhantomData,
        }
    }

    fn knowledge(&self, node: &Node<Id>) -> Domain {
        self.result.get(node).cloned().unwrap_or_else(P::bot)
    }

    fn summarize_predecessors(&self, node: &Node<Id>, old_input: &Domain) -> Domain {
        let incoming_edges = self.cfg.predecessors(node);
        if !incoming_edges.is_empty() {
            incoming_edges
                .iter()
                .map(|edge| {
                    let pred_output = self.knowledge(&edge.lhs);
                    P::transfer(pred_output, edge)
                })
                .fold(P::bot(), |acc, pred_output| P::join(acc, pred_output))
        } else {
            old_input.clone()
        }
    }

    fn transfer(&mut self, node: &Node<Id>) -> bool {
        let kw = self.knowledge(node);
        let new_kw = self.summarize_predecessors(node, &kw);
        let changed = kw != new_kw;
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

    pub fn analyse(cfg: &'a Flow<'a>, game: &Game<Arc<str>>) -> BTreeMap<Node<Id>, Domain> {
        let mut worker = Self::new(cfg, game);
        worker.run();
        worker.result
    }
}
