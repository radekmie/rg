use crate::ast::analyses::ReachingPaths;
use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_repeats(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_paths = self.analyse::<ReachingPaths>(true);
        for (node, paths) in reaching_paths {
            if !paths.iter().any(|path| path.has_duplicate) {
                continue;
            }

            let identifiers: Vec<_> = paths
                .into_iter()
                .flat_map(|path| path.variables)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            if let Some(Pragma::Repeat { nodes, .. }) = self.pragmas.iter_mut().find(
                |x| matches!(x, Pragma::Repeat { identifiers: ids, .. } if *ids == identifiers),
            ) {
                if let Err(index) = nodes.binary_search(&node) {
                    nodes.insert(index, node);
                }
            } else {
                let pragma = Pragma::Repeat {
                    span: Span::none(),
                    nodes: vec![node],
                    identifiers,
                };

                let index = self.pragmas.partition_point(|x| *x < pragma);
                self.pragmas.insert(index, pragma);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_repeats,
        repeat_example,
        "begin, a: ; a, a: x = y[x]; a, end: x == z;",
        "begin, a: ; a, a: x = y[x]; a, end: x == z; @repeat a : x;"
    );
}
