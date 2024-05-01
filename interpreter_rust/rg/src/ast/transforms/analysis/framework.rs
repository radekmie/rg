use crate::ast::Game;

use super::flow::Flow;
use std::{collections::BTreeMap, sync::Arc};

type Id = Arc<str>;
type Node = crate::ast::Node<Id>;
type Edge = crate::ast::Edge<Id>;

pub trait Parameters<Domain: Clone> {
    fn bot() -> Domain;
    fn join(a: Domain, b: Domain) -> Domain;
    fn extreme(program: &Game<Id>) -> Domain;

    fn kill(input: Domain, edge: &Edge) -> Domain;
    fn gen(input: Domain, edge: &Edge) -> Domain;
    fn transfer(input: Domain, edge: &Edge) -> Domain {
        let input = Self::kill(input, edge);
        Self::gen(input, edge)
    }
}

pub trait Instance<Domain: Clone + PartialEq + std::fmt::Debug> {
    fn analyse<P: Parameters<Domain>>(
        &self,
        flow: &Flow,
        p: P,
        game: &Game<Id>,
    ) -> BTreeMap<Node, Domain> {
        Worker::analyse(flow, p, game)
    }
    fn name() -> &'static str;
}

struct Worker<'a, Domain: Clone, P: Parameters<Domain>> {
    cfg: &'a Flow<'a>,
    result: BTreeMap<Node, Domain>,
    parameters: P,
}

impl<'a, Domain: Clone + PartialEq + std::fmt::Debug, P: Parameters<Domain>> Worker<'a, Domain, P> {
    fn new(cfg: &'a Flow<'a>, parameters: P, game: &Game<Id>) -> Self {
        let mut result = BTreeMap::new();
        let entry = cfg.entry();
        let extreme = P::extreme(game);
        result.insert(entry, extreme); // initial result-table
        Worker {
            cfg,
            result,
            parameters,
        }
    }

    fn knowledge(&self, node: &Node) -> Option<Domain> {
        self.result.get(node).cloned()
    }

    fn summarize_predecessors(&self, node: &Node, old_input: &Domain) -> Domain {
        if *node != self.cfg.entry() {
            let incoming_edges = self.cfg.predecessors(node).unwrap();
            let preds_kw: Vec<_> = incoming_edges
                .iter()
                .map(|edge| {
                    let pred_output = self.knowledge(&edge.lhs).unwrap_or(P::bot());
                    P::transfer(pred_output, edge)
                })
                .collect();
            preds_kw
                .into_iter()
                .fold(P::bot(), |acc, pred_output| P::join(acc, pred_output))
        } else {
            old_input.clone()
        }
    }

    fn transfer(&mut self, node: &Node) -> bool {
        let kw = self.knowledge(node).unwrap_or(P::bot());
        let new_kw = self.summarize_predecessors(node, &kw);
        let changed = kw != new_kw;
        self.result.insert((*node).clone(), new_kw);
        changed
    }

    fn step(&mut self) -> bool {
        let nodes = self.cfg.nodes();
        let mut changed = false;
        for node in nodes.iter() {
            if self.transfer(node) {
                changed = true;
            }
        }
        changed
    }

    fn run(&mut self) {
        while self.step() {}
    }

    pub fn analyse(cfg: &'a Flow<'a>, parameters: P, game: &Game<Id>) -> BTreeMap<Node, Domain> {
        let mut worker = Self::new(cfg, parameters, game);
        worker.run();
        worker.result
    }
}
