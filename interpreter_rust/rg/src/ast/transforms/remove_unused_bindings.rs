use std::{collections::BTreeSet, sync::Arc};

use crate::ast::{Error, Game, Node};

use super::gen_fresh_node;

type Id = Arc<str>;

impl Game<Id> {
    pub fn remove_unused_bindings(&mut self) -> Result<(), Error<Id>> {
        let mut nodes: BTreeSet<_> = self.nodes().iter().map(|n| (*n).clone()).collect();
        while let Some((node, binding)) = self.has_removable_binding() {
            let mut new_node = gen_fresh_node(format!("{}_{binding}", node.literal()), &nodes);
            nodes.insert(new_node.clone());
            for (identifier, type_) in node.bindings() {
                if identifier != &binding {
                    new_node.add_binding(identifier.clone(), type_.clone());
                }
            }
            self.rename_node(&node, &new_node);
        }

        Ok(())
    }

    fn has_removable_binding(&self) -> Option<(Node<Id>, Id)> {
        for node in self.nodes() {
            if self.is_reachability_target(node) {
                continue;
            }
            for (binding, _) in node.bindings() {
                if self.can_remove_tail(node, binding) || self.can_remove_head(node, binding) {
                    return Some((node.clone(), binding.clone()));
                }
            }
        }

        None
    }

    // This is a last node with this binding and the binding is unused
    fn can_remove_tail(&self, node: &Node<Id>, id: &Id) -> bool {
        let mut outgoing_edges = self.outgoing_edges(node);
        // If its not a last node with this binding or the binding is used in any outgoing edge
        if outgoing_edges.any(|edge| edge.rhs.has_binding(id) || edge.label.has_binding(id)) {
            return false;
        }
        // If the binding is used in any incoming edge, the `lhs` must also have the binding
        let mut incoming_edges = self.incoming_edges(node);
        incoming_edges.all(|edge| !edge.label.has_binding(id) || edge.lhs.has_binding(id))
    }

    // This is a first node with this binding and the binding is unused
    fn can_remove_head(&self, node: &Node<Id>, id: &Id) -> bool {
        let mut incoming_edges = self.incoming_edges(node);
        // If its not a first node with this binding or the binding is used in any incoming edge
        if incoming_edges.any(|edge| edge.lhs.has_binding(id) || edge.label.has_binding(id)) {
            return false;
        }
        let mut outgoing_edges = self.outgoing_edges(node);
        // If the binding is used in any outgoing edge, the `rhs` must also have the binding
        outgoing_edges.all(|edge| !edge.label.has_binding(id) || edge.rhs.has_binding(id))
    }
}
