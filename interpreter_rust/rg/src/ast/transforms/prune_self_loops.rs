use crate::ast::{Error, Game, Label};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn prune_self_loops(&mut self) -> Result<(), Error<Arc<str>>> {
        self.edges.retain(|edge| {
            edge.lhs != edge.rhs
                || !matches!(
                    &edge.label,
                    Label::Comparison { .. } | Label::Reachability { .. } | Label::Skip { .. }
                )
        });

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(prune_self_loops, edge_assignment, "x, x: v = 0;");
    test_transform!(prune_self_loops, edge_assignment_any, "x, x: v = V(*);");
    test_transform!(prune_self_loops, edge_comparison, "x, x: v == 0;", "");
    test_transform!(prune_self_loops, edge_reachability, "x, x: ? a -> b;", "");
    test_transform!(
        prune_self_loops,
        edge_reachability_negated,
        "x, x: ! a -> b;",
        ""
    );
    test_transform!(prune_self_loops, edge_skip, "x, x:;", "");
    test_transform!(prune_self_loops, edge_tag, "x, x: $v;");
    test_transform!(prune_self_loops, edge_tag_variable, "x, x: $$v;");
}
