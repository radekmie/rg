use crate::ast::analyses::ReachableNodes;
use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn skip_unused_tags(&mut self) -> Result<(), Error<Arc<str>>> {
        let reachable_nodes = self.analyse::<ReachableNodes>(false);
        for edge in &mut self.edges {
            if edge.label.is_tag()
                && !reachable_nodes
                    .get(&edge.lhs)
                    .is_some_and(|reachable| *reachable)
            {
                Arc::make_mut(edge).skip();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        skip_unused_tags,
        small,
        "begin, end: ;
        t1, t2: $ 1;
        t2, t3: $ 2;",
        "begin, end: ;
        t1, t2: ;
        t2, t3: ;"
    );

    test_transform!(
        skip_unused_tags,
        reachability,
        "begin, end: ? t1 -> t2;
        t1, t2: $ 1;
        t2, t3: $ 2;",
        "begin, end: ? t1 -> t2;
        t1, t2: ;
        t2, t3: ;"
    );

    test_transform!(
        skip_unused_tags,
        used_tag,
        "begin, t2: ? t1 -> t2;
        t1, t2: $ 1;
        t2, t3: $ 2;
        t3, end: ;",
        "begin, t2: ? t1 -> t2;
        t1, t2: ;
        t2, t3: $ 2;
        t3, end: ;"
    );
}
