use crate::ast::{Game, Rule};

impl<Id: Ord> Game<Id> {
    pub fn canonicalize(mut self) -> Self {
        self.0.iter_mut().for_each(Rule::canonicalize);
        self.0.sort_unstable();
        self.0.dedup();
        self
    }
}

impl<Id: Ord> Rule<Id> {
    fn canonicalize(&mut self) {
        self.predicates.sort_unstable();
    }
}
