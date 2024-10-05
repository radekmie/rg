use super::gen_fresh_node;
use crate::ast::{Error, Game, Node};
use std::collections::BTreeSet;
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn prune_unused_bindings(&mut self) -> Result<(), Error<Id>> {
        let mut nodes: BTreeSet<_> = self.nodes().iter().map(|n| (*n).clone()).collect();
        while let Some((node, binding)) = self.first_removable_binding() {
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

    fn first_removable_binding(&self) -> Option<(Node<Id>, Id)> {
        for node in self.nodes() {
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
        for edge in &self.edges {
            // Outgoing edge, the rhs and label cannot have this binding
            if &edge.lhs == node && (edge.rhs.has_binding(id) || edge.label.has_binding(id)) {
                return false;
            }
            // Incoming edge, if the label has this binding, the lhs must also have it
            if &edge.rhs == node && (edge.label.has_binding(id) && !edge.lhs.has_binding(id)) {
                return false;
            }
        }
        true
    }

    // This is a first node with this binding and the binding is unused
    fn can_remove_head(&self, node: &Node<Id>, id: &Id) -> bool {
        for edge in &self.edges {
            // Incoming edge, the lhs and label cannot have this binding
            if &edge.rhs == node && (edge.lhs.has_binding(id) || edge.label.has_binding(id)) {
                return false;
            }
            // Outgoing edge, if the label has this binding, the rhs must also have it
            if &edge.lhs == node && (edge.label.has_binding(id) && !edge.rhs.has_binding(id)) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        prune_unused_bindings,
        small1,
        "begin, a: ;
        a, a(bind_1: A): ;",
        "begin, a: ;
        a, __gen_1_a_bind_1: ;"
    );

    test_transform!(
        prune_unused_bindings,
        small2,
        "begin, a: ;
        a, a(bind_1: A): $ bind_1 ;"
    );

    test_transform!(
        prune_unused_bindings,
        small3,
        "begin, a(bind_1: A): bind_1 == 1;
        a(bind_1: A), a: ;"
    );

    test_transform!(
        prune_unused_bindings,
        small4,
        "begin, a(bind_1: A): ;
        a(bind_1: A), a: ;",
        "begin, __gen_1_a_bind_1: ;
        __gen_1_a_bind_1, a: ;"
    );

    test_transform!(
        prune_unused_bindings,
        small5,
        "begin, a(bind_1: A): ;
        a(bind_1: A), a: $ bind_1;"
    );

    test_transform!(
        prune_unused_bindings,
        small6,
        "begin, a(bind_1: A): ;
        a(bind_1: A), b(bind_1: A): $ bind_1;",
        "begin, __gen_1_a_bind_1: ;
        __gen_1_a_bind_1, b(bind_1: A): $ bind_1;"
    );

    test_transform!(
        prune_unused_bindings,
        small7,
        "begin, a(bind_1: A): ;
        a(bind_1: A), b(bind_1: A): $ bind_1;
        b(bind_1: A), c(bind_1: A): ;",
        "begin, __gen_1_a_bind_1: ;
        __gen_1_a_bind_1, b(bind_1: A): $ bind_1;
        b(bind_1: A), __gen_1_c_bind_1: ;"
    );

    test_transform!(
        prune_unused_bindings,
        rename_reachability,
        "begin, a: ;
        a, a(bind_1: A): ;
        c, d: ? a -> a(bind_1: A);",
        "begin, a: ;
        a, __gen_1_a_bind_1: ;
        c, d: ? a -> __gen_1_a_bind_1;"
    );

    test_transform!(
        prune_unused_bindings,
        rename_pragmas,
        "begin, a: ;
        a, a(bind_1: A): ;
        @disjoint a(bind_1: A): a(bind_1: A);
        @disjointExhaustive a(bind_1: A): a(bind_1: A);
        @repeat a(bind_1: A): 1;
        @simpleApply a(bind_1: A) 1 : a(bind_1: A);
        @simpleApplyExhaustive a(bind_1: A) 1 : a(bind_1: A);
        @tagIndex a(bind_1: A): 1;
        @tagMaxIndex a(bind_1: A): 1;
        @unique a(bind_1: A);",
        "@disjoint __gen_1_a_bind_1 : __gen_1_a_bind_1;
        @disjointExhaustive __gen_1_a_bind_1 : __gen_1_a_bind_1;
        @repeat __gen_1_a_bind_1 : 1;
        @simpleApply __gen_1_a_bind_1 1 : __gen_1_a_bind_1;
        @simpleApplyExhaustive __gen_1_a_bind_1 1 : __gen_1_a_bind_1;
        @tagIndex __gen_1_a_bind_1 : 1;
        @tagMaxIndex __gen_1_a_bind_1 : 1;
        @unique __gen_1_a_bind_1;
        begin, a: ;
        a, __gen_1_a_bind_1: ;"
    );
}
