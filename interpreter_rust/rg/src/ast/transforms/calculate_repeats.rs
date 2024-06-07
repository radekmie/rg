use crate::ast::analyses::ReachingPaths;
use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_repeats(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_paths = self.analyse::<ReachingPaths>(false);
        for (node, variables) in reaching_paths {
            let has_none_repeat = variables.get(&None) == Some(&true);
            let identifiers: Vec<_> = variables
                .into_iter()
                .filter(|(_, is_repeated)| *is_repeated)
                .filter_map(|(variable, _)| variable)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            if identifiers.is_empty() && !has_none_repeat {
                continue;
            }

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

    test_transform!(
        calculate_repeats,
        small_loop,
        "begin, x: ; x, y: ; y, x: ; y, end: ;",
        "begin, x: ; x, y: ; y, x: ; y, end: ; @repeat end x y : ;"
    );

    test_transform!(
        calculate_repeats,
        hex_loop,
        "begin, end: ? 24 -> 25;
        24, 27: ;
        26, 25: ! 46 -> 47;
        27, 26: ;
        27, 32: coord = direction[coord][E];
        27, 32: coord = direction[coord][NE];
        27, 32: coord = direction[coord][NW];
        27, 32: coord = direction[coord][SE];
        27, 32: coord = direction[coord][SW];
        27, 32: coord = direction[coord][W];
        28, 26: ;
        28, 27: ;
        30, 28: board[coord] == r;
        32, 30: coord != null;
        46, 47: direction[coord][NW] != null;",
        "begin, end: ? 24 -> 25;
        24, 27: ;
        26, 25: ! 46 -> 47;
        27, 26: ;
        27, 32: coord = direction[coord][E];
        27, 32: coord = direction[coord][NE];
        27, 32: coord = direction[coord][NW];
        27, 32: coord = direction[coord][SE];
        27, 32: coord = direction[coord][SW];
        27, 32: coord = direction[coord][W];
        28, 26: ;
        28, 27: ;
        30, 28: board[coord] == r;
        32, 30: coord != null;
        46, 47: direction[coord][NW] != null;
        @repeat 25 26 27 28 30 32 : coord;"
    );

    test_transform!(
        calculate_repeats,
        repeat_test,
        file "../../../../../examples/repeatTest.rg",
        "@repeat selectDir4 : pos;"
    );

    test_transform!(
        calculate_repeats,
        repeat_test_big,
        file "../../../../../examples/repeatTestBig.rg",
        "@repeat goDown goLeft goRight goUp main : pos; @repeat setScore :;"
    );

    test_transform!(
        calculate_repeats,
        tictactoe,
        file "../../../../../examples/ticTacToe.rg",
        "@repeat end endcheckline : ; @repeat move preend turn : playerTurn;"
    );
}
