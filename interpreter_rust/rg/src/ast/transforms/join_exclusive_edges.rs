use std::collections::BTreeSet;

use crate::ast::{Error, Game, Label};

impl<Id: PartialEq + Clone + Ord> Game<Id> {
    pub fn join_exclusive_edges(&mut self) -> Result<(), Error<Id>> {
        let mut to_skip = Vec::new();
        let mut to_remove = BTreeSet::new();
        for (idx, edge) in self.edges.iter().enumerate() {
            if to_remove.contains(edge) {
                continue;
            }
            if let Label::Comparison { lhs, rhs, negated } = &edge.label {
                for other_edge in &self.edges {
                    if let Label::Comparison {
                        lhs: other_lhs,
                        rhs: other_rhs,
                        negated: other_negated,
                    } = &other_edge.label
                    {
                        if edge.lhs == other_edge.lhs
                            && edge.rhs == other_edge.rhs
                            && lhs == other_lhs
                            && rhs == other_rhs
                            && negated != other_negated
                        {
                            to_skip.push(idx);
                            to_remove.insert(other_edge.clone());
                        }
                    }
                }
            }

            if let Label::Reachability {
                lhs, rhs, negated, ..
            } = &edge.label
            {
                for other_edge in &self.edges {
                    if let Label::Reachability {
                        lhs: other_lhs,
                        rhs: other_rhs,
                        negated: other_negated,
                        ..
                    } = &other_edge.label
                    {
                        if edge.lhs == other_edge.lhs
                            && edge.rhs == other_edge.rhs
                            && lhs == other_lhs
                            && rhs == other_rhs
                            && negated != other_negated
                        {
                            to_skip.push(idx);
                            to_remove.insert(other_edge.clone());
                        }
                    }
                }
            }
        }

        for idx in to_skip {
            self.edges[idx].skip();
        }
        self.edges.retain(|edge| !to_remove.contains(edge));

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        join_exclusive_edges,
        reachability1,
        "begin, end: ? a -> b;
        begin, end: ! a -> b;",
        "begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        reachability2,
        "begin, end: ? a -> b;
        begin, a: ! a -> b;"
    );

    test_transform!(
        join_exclusive_edges,
        comparison1,
        "begin, end: a == b;
        begin, end: a != b;",
        "begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        comparison2,
        "begin, end: a == b;
        begin, end: a == b;"
    );
}
