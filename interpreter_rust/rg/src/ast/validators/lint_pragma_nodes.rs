use crate::ast::{Error, ErrorReason, Game, Pragma};

impl<Id: Clone + Ord + std::fmt::Display> Game<Id> {
    pub fn lint_pragma_nodes(&self) -> impl Iterator<Item = Error<Id>> + '_ {
        let nodes = self.nodes();
        self.pragmas
            .iter()
            .filter_map(Pragma::nodes)
            .flatten()
            .filter_map(move |node| {
                if !nodes.contains(node) {
                    return Some(Error {
                        game: None,
                        reason: ErrorReason::UnresolvedNode { node: node.clone() },
                    });
                }

                None
            })
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{ErrorReason, Node};
    use crate::test_linter;
    use std::sync::Arc;

    macro_rules! unresolved {
        ($symbol:expr) => {
            ErrorReason::UnresolvedNode {
                node: Node::new(Arc::from($symbol)),
            }
        };
    }

    test_linter!(
        lint_pragma_nodes,
        disjoint,
        "@disjoint x : y;",
        &[unresolved!("x"), unresolved!("y"),]
    );

    test_linter!(
        lint_pragma_nodes,
        disjoint_exhaustive,
        "@disjointExhaustive x : y;",
        &[unresolved!("x"), unresolved!("y"),]
    );

    test_linter!(
        lint_pragma_nodes,
        iterator,
        "@iterator x y z : w;",
        &[unresolved!("x"), unresolved!("y"), unresolved!("z"),]
    );

    test_linter!(
        lint_pragma_nodes,
        repeat,
        "@repeat x :;",
        &[unresolved!("x")]
    );

    test_linter!(
        lint_pragma_nodes,
        simple_apply,
        "@simpleApply x y [];",
        &[unresolved!("x"), unresolved!("y"),]
    );

    test_linter!(
        lint_pragma_nodes,
        simple_apply_exhaustive,
        "@simpleApplyExhaustive x y [];",
        &[unresolved!("x"), unresolved!("y"),]
    );

    test_linter!(
        lint_pragma_nodes,
        tag_index,
        "@tagIndex x y : 0;",
        &[unresolved!("x"), unresolved!("y"),]
    );

    test_linter!(
        lint_pragma_nodes,
        tag_max_index,
        "@tagMaxIndex x y : 0;",
        &[unresolved!("x"), unresolved!("y"),]
    );

    test_linter!(lint_pragma_nodes, unique, "@unique x;", &[unresolved!("x")]);
}
