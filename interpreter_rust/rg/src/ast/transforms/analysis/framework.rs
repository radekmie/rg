use super::flow_graph::FlowGraph;
use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};

type Id = Arc<str>;

pub trait Parameters<L, G, Domain: Clone> {
    fn bot() -> Domain;
    fn join(a: Domain, b: Domain) -> Domain;
    fn extreme(program: &G) -> Domain;
    // fn transfer(input: &Domain, label: &L) -> Domain;

    fn kill(input: Domain, label: &L) -> Domain;
    fn gen(input: Domain, label: &L) -> Domain;
    fn transfer(input: Domain, label: &L) -> Domain {
        let input = Self::kill(input, label);
        Self::gen(input, label)
    }
}

pub trait Instance<L: Ord + Clone, Domain: Clone + PartialEq> {
    // type Result = BTreeMap<L, (Domain, Domain)>;

    fn analyse<F: FlowGraph<L>, G, P: Parameters<L, G, Domain>>(
        &self,
        flow: &F,
        p: P,
        game: &G,
    ) -> BTreeMap<L, (Domain, Domain)> {
        Worker::analyse(flow, p, game)
    }
    fn name() -> &'static str;
}


struct Worker<'a, L, G, Domain: Clone, F: FlowGraph<L>, P: Parameters<L, G, Domain>> {
    cfg: &'a F,
    result: BTreeMap<L, (Domain, Domain)>,
    parameters: P,
    _g: PhantomData<G>,
}

impl<
        'a,
        L: Ord + Clone,
        G,
        Domain: Clone + PartialEq,
        F: FlowGraph<L>,
        P: Parameters<L, G, Domain>,
    > Worker<'a, L, G, Domain, F, P>
{
    fn new(cfg: &'a F, parameters: P, game: &G) -> Self {
        let mut result = BTreeMap::new();
        let entry = cfg.entry();
        let extreme = P::extreme(game);
        result.insert(entry, (extreme.clone(), extreme)); // initial result-table
        Worker {
            cfg,
            result,
            parameters,
            _g: PhantomData,
        }
    }

    fn output(&self, label: &L) -> &Domain {
        let kw = self.result.get(label).unwrap();
        &kw.1
    }

    fn summarize_predecessors(&self, label: &L, old_input: &Domain) -> Domain {
        if *label != self.cfg.entry() {
            let preds = self.cfg.predecessors(label).unwrap();
            let preds_outputs: Vec<_> = preds
                .iter()
                .map(|&pred| self.output(pred).clone())
                .collect();
            preds_outputs
                .into_iter()
                .fold(P::bot(), |acc, pred_output| P::join(acc, pred_output))
        } else {
            old_input.clone()
        }
    }

    fn transfer(&mut self, label: &L) -> bool {
        dbg!("transfer");
        let (input, old_output) = self.result.get(label).unwrap().clone();
        let input = self.summarize_predecessors(label, &input);
        let output = if self.cfg.is_reversed() {
            P::transfer(input.clone(), label)
        } else {
            input.clone()
        };
        self.result
            .insert((*label).clone(), (input, output.clone()));
        output != old_output
    }

    fn step(&mut self) -> bool {
        let labels = self.cfg.labels();
        let mut changed = false;
        for label in labels.iter() {
            if self.transfer(label) {
                changed = true;
            }
        }
        changed
    }

    fn run(&mut self) {
        while self.step() {}
    }

    pub fn analyse(cfg: &'a F, parameters: P, game: &G) -> BTreeMap<L, (Domain, Domain)> {
        let mut worker = Self::new(cfg, parameters, game);
        worker.run();
        worker.result
    }
}
