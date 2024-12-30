use crate::ast::{Error, ErrorReason, Game, Label};

impl<Id: Clone + Ord> Game<Id> {
    pub fn lint_reachabilities(&self) -> impl Iterator<Item = Error<Id>> + '_ {
        let is_reachable = self.make_is_reachable();
        self.edges.iter().filter_map(move |edge| {
            if let Label::Reachability { lhs, rhs, .. } = &edge.label {
                if !is_reachable(lhs, rhs) {
                    return Some(Error {
                        game: None,
                        reason: ErrorReason::Unreachable {
                            lhs: lhs.clone(),
                            rhs: rhs.clone(),
                        },
                    });
                }
            }

            None
        })
    }
}
