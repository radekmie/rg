use super::flow::Flow;
use crate::ast::Game;
use std::{collections::BTreeMap, sync::Arc};

type Node = crate::ast::Node<Arc<str>>;
type Edge = crate::ast::Edge<Arc<str>>;

pub trait Parameters<Domain: Clone> {
    fn bot(&self) -> Domain;
    fn join(&self, a: Domain, b: Domain) -> Domain;
    fn extreme(&self, program: &Game<Arc<str>>) -> Domain;

    fn kill(&self, input: Domain, edge: &Edge) -> Domain;
    fn gen(&self, input: Domain, edge: &Edge) -> Domain;
    fn transfer(&self, input: Domain, edge: &Edge) -> Domain {
        let input = self.kill(input, edge);
        self.gen(input, edge)
    }
}

pub trait Instance<Domain: Clone + PartialEq + std::fmt::Debug> {
    fn analyse<P: Parameters<Domain>>(
        &self,
        flow: &Flow,
        p: P,
        game: &Game<Arc<str>>,
    ) -> BTreeMap<Node, Domain> {
        Worker::analyse(flow, p, game)
    }
    fn name() -> &'static str;
}

struct Worker<'a, Domain: Clone, P: Parameters<Domain>> {
    cfg: &'a Flow<'a>,
    result: BTreeMap<Node, Domain>,
    p: P,
}

impl<'a, Domain: Clone + PartialEq + std::fmt::Debug, P: Parameters<Domain>> Worker<'a, Domain, P> {
    fn new(cfg: &'a Flow<'a>, parameters: P, game: &Game<Arc<str>>) -> Self {
        let mut result = BTreeMap::new();
        let entry = cfg.entry();
        let extreme = parameters.extreme(game);
        result.insert(entry, extreme); // initial result-table
        Worker {
            cfg,
            result,
            p: parameters,
        }
    }

    fn knowledge(&self, node: &Node) -> Option<Domain> {
        self.result.get(node).cloned()
    }

    fn summarize_predecessors(&self, node: &Node, old_input: &Domain) -> Domain {
        let incoming_edges = self.cfg.predecessors(node);
        if !incoming_edges.is_empty() {
            incoming_edges
                .iter()
                .map(|edge| {
                    let pred_output = self.knowledge(&edge.lhs).unwrap_or(self.p.bot());
                    self.p.transfer(pred_output, edge)
                })
                .fold(self.p.bot(), |acc, pred_output| {
                    self.p.join(acc, pred_output)
                })
        } else {
            old_input.clone()
        }
    }

    fn transfer(&mut self, node: &Node) -> bool {
        let kw = self.knowledge(node).unwrap_or(self.p.bot());
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

    pub fn analyse(
        cfg: &'a Flow<'a>,
        parameters: P,
        game: &Game<Arc<str>>,
    ) -> BTreeMap<Node, Domain> {
        let mut worker = Self::new(cfg, parameters, game);
        worker.run();
        worker.result
    }
}
