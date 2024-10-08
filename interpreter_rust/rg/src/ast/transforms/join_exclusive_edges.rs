use crate::ast::{Error, Game};
use std::collections::BTreeSet;

impl<Id: PartialEq> Game<Id> {
    pub fn join_exclusive_edges(&mut self) -> Result<(), Error<Id>> {
        let mut to_ignore = BTreeSet::new();
        let mut to_remove = BTreeSet::new();

        for (i, x) in self.edges.iter().enumerate() {
            if !to_remove.contains(&i) && (x.label.is_comparison() || x.label.is_reachability()) {
                for (j, y) in self.edges.iter().enumerate() {
                    if x.lhs == y.lhs && x.rhs == y.rhs && x.label.is_negated(&y.label) {
                        to_ignore.insert(i);
                        to_remove.insert(j);
                    }
                }
            }
        }

        for index in to_ignore {
            self.edges[index].skip();
        }

        for index in to_remove.into_iter().rev() {
            self.edges.remove(index);
        }

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
