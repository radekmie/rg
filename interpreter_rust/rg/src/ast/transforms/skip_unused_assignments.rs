use crate::ast::{Edge, Error, Expression, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use utils::position::Positioned;

impl<Id: PartialEq + Ord + Clone> Game<Id> {
    pub fn skip_unused_assignments(&mut self) -> Result<(), Error<Id>> {
        let next_edges = self.next_edges();
        let mut unused = BTreeSet::new();
        for edge in &self.edges {
            if let Label::Assignment { lhs, .. } = &edge.label {
                if let Expression::Reference { identifier } = lhs.as_ref() {
                    if self.check_unused_assignment(&next_edges, &edge.rhs, identifier) {
                        unused.insert((*edge).clone());
                    }
                }
            }
        }

        for edge in &mut self.edges {
            if unused.contains(edge) {
                edge.label = Label::Skip { span: edge.span() }
            }
        }

        Ok(())
    }

    fn check_unused_assignment(
        &self,
        next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Edge<Id>>>,
        start: &Node<Id>,
        id: &Id,
    ) -> bool {
        let mut queue = vec![start];
        let mut seen = BTreeSet::new();
        while let Some(lhs) = queue.pop() {
            let maybe_edges = next_edges.get(&lhs);
            if seen.insert(lhs) {
                if let Some(edges) = maybe_edges {
                    for edge in edges {
                        if !is_reassigned(&edge.label, id) {
                            if !seen.contains(&edge.rhs) {
                                queue.push(&edge.rhs);
                            }
                            if let Label::Reachability { lhs, .. } = &edge.label {
                                if !seen.contains(lhs) {
                                    queue.push(lhs);
                                }
                            }
                            if is_used_in_label(&edge.label, id) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
        true
    }
}

fn is_reassigned<Id: PartialEq>(label: &Label<Id>, id: &Id) -> bool {
    match label {
        Label::Assignment { lhs, rhs } => {
            if let Expression::Reference { identifier } = lhs.as_ref() {
                identifier == id && !is_used_in_expression(rhs, id)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_used_in_label<Id: PartialEq>(label: &Label<Id>, id: &Id) -> bool {
    match label {
        Label::Assignment { lhs, rhs } | Label::Comparison { lhs, rhs, .. } => {
            is_used_in_expression(lhs, id) || is_used_in_expression(rhs, id)
        }
        Label::Tag { symbol } => symbol == id,
        _ => false,
    }
}

fn is_used_in_expression<Id: PartialEq>(expression: &Expression<Id>, id: &Id) -> bool {
    match expression {
        Expression::Access { lhs, rhs, .. } => {
            is_used_in_expression(lhs, id) || is_used_in_expression(rhs, id)
        }
        Expression::Cast { rhs, .. } => is_used_in_expression(rhs, id),
        Expression::Reference { identifier } => identifier == id,
    }
}
