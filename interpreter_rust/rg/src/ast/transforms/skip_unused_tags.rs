use crate::ast::analyses::ReachableNodes;
use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn skip_unused_tags(&mut self) -> Result<(), Error<Arc<str>>> {
        let artificial_tags = self.artificial_tags();
        let reachable_nodes = self.analyse::<ReachableNodes>(false);
        for edge in &mut self.edges {
            if (edge.label.is_tag_and(|tag| !artificial_tags.contains(tag))
                || edge.label.is_tag_variable())
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
        reachability_artificial,
        "begin, end: ? t1 -> t2;
        t1, t2: $ 1;
        t2, t3: $ 2;
        @artificialTag 1 2;"
    );

    test_transform!(
        skip_unused_tags,
        reachability_tag_variable,
        "begin, end: ? t1 -> t2;
        t1, t2: $$ 1;
        t2, t3: $$ 2;",
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

    test_transform!(
        skip_unused_tags,
        used_tag_variable,
        "begin, t2: ? t1 -> t2;
        t1, t2: $$ 1;
        t2, t3: $$ 2;
        t3, end: ;",
        "begin, t2: ? t1 -> t2;
        t1, t2: ;
        t2, t3: $$ 2;
        t3, end: ;"
    );
}
