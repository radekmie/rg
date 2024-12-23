use crate::ast::{Game, Rule, Term};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl<Id: Clone + Ord> Game<Id> {
    pub fn static_terms(&self) -> BTreeMap<Id, BTreeSet<Arc<Term<Id>>>> {
        self.0
            .iter()
            .filter_map(|rule| match rule.predicates.is_empty() {
                true => rule.term.as_atom(),
                false => None,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .filter(|x| {
                self.0.iter().all(|rule| match rule.term.as_atom() {
                    Some(y) if *x == y => rule.predicates.is_empty(),
                    _ => true,
                })
            })
            .map(|x| {
                (
                    x.clone(),
                    self.0
                        .iter()
                        .filter(move |rule| rule.term.as_atom() == Some(x))
                        .map(|rule| rule.term.clone())
                        .collect(),
                )
            })
            .collect()
    }
}

impl<Id: Clone + Ord> Rule<Id> {
    pub fn prune_static_terms(
        &mut self,
        static_terms: &BTreeMap<Id, BTreeSet<Arc<Term<Id>>>>,
    ) -> bool {
        let mut has_failed_static_term = false;
        self.predicates.retain(|predicate| {
            // As soon as any predicate fails, fail all of them.
            if has_failed_static_term {
                return false;
            }

            if !predicate.has_variable() {
                if let Some(id) = predicate.term.as_atom() {
                    if let Some(terms) = static_terms.get(id) {
                        if predicate.is_negated || !terms.contains(&predicate.term) {
                            has_failed_static_term = true;
                        }

                        return false;
                    }
                }
            }

            true
        });

        has_failed_static_term
    }
}
