use crate::ast::{Error, ErrorReason, Game, Label, ReachabilityCheckResult};

impl<Id: Clone + Ord + std::fmt::Display> Game<Id> {
    pub fn lint_reachabilities(&self) -> impl Iterator<Item = Error<Id>> + '_ {
        let check_reachability = self.make_check_reachability(true);
        self.edges.iter().filter_map(move |edge| {
            if let Label::Reachability { lhs, rhs, .. } = &edge.label {
                let reason = match check_reachability(lhs, rhs) {
                    ReachabilityCheckResult::Loop => Some(ErrorReason::ReachabilityLoop {
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                    }),
                    ReachabilityCheckResult::Reachable => None,
                    ReachabilityCheckResult::Unreachable => Some(ErrorReason::Unreachable {
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                    }),
                };

                if let Some(reason) = reason {
                    return Some(Error { game: None, reason });
                }
            }

            None
        })
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Node;
    use crate::test_linter;

    test_linter!(
        lint_reachabilities,
        loop_1,
        "a, b: ? a -> b;",
        vec![ErrorReason::ReachabilityLoop {
            lhs: Node::new(Arc::from("a")),
            rhs: Node::new(Arc::from("b")),
        }]
    );

    test_linter!(
        lint_reachabilities,
        loop_2,
        "a, b: ? a -> c; b, c: ;",
        vec![ErrorReason::ReachabilityLoop {
            lhs: Node::new(Arc::from("a")),
            rhs: Node::new(Arc::from("c")),
        }]
    );

    test_linter!(
        lint_reachabilities,
        loop_3,
        "a, b: ; b, c: ? a -> c;",
        vec![ErrorReason::ReachabilityLoop {
            lhs: Node::new(Arc::from("a")),
            rhs: Node::new(Arc::from("c")),
        }]
    );

    test_linter!(
        lint_reachabilities,
        unreachable_1,
        "begin, end: ? x -> y;",
        vec![ErrorReason::Unreachable {
            lhs: Node::new(Arc::from("x")),
            rhs: Node::new(Arc::from("y")),
        }]
    );

    test_linter!(
        lint_reachabilities,
        unreachable_2,
        "begin, end: ? x -> y; x, z: ;",
        vec![ErrorReason::Unreachable {
            lhs: Node::new(Arc::from("x")),
            rhs: Node::new(Arc::from("y")),
        }]
    );
}
