use crate::ast::analyses::ReachingPaths;
use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_uniques(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_paths = self.analyse::<ReachingPaths>(false);
        let mut unique_nodes: BTreeSet<_> = reaching_paths
            .into_iter()
            .filter(|(_, variables)| !variables.values().any(|is_repeated| *is_repeated))
            .map(|(node, _)| node)
            .collect();

        self.pragmas.retain(|pragma| {
            if let Pragma::Unique { nodes, .. } = pragma {
                unique_nodes.extend(nodes.iter().cloned());
                false
            } else {
                true
            }
        });

        let pragma = Pragma::Unique {
            span: Span::none(),
            nodes: unique_nodes.into_iter().collect(),
        };

        let index = self.pragmas.partition_point(|x| *x < pragma);
        self.pragmas.insert(index, pragma);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_uniques,
        small_unique,
        "begin, x: ; x, end: ;",
        "begin, x: ; x, end: ; @unique begin end x;"
    );

    test_transform!(
        calculate_uniques,
        small_loop,
        "begin, x: ; x, y: ; y, x: ; y, end: ;",
        "begin, x: ; x, y: ; y, x: ; y, end: ; @unique begin;"
    );

    test_transform!(
        calculate_uniques,
        repeat_example,
        "begin, a: ; a, a: x = y[x]; a, end: x == z;",
        "begin, a: ; a, a: x = y[x]; a, end: x == z; @unique begin end;"
    );

    test_transform!(
        calculate_uniques,
        repeat_test,
        file "../../../../../examples/repeatTest.rg",
        "@unique begin end setScore win;"
    );

    test_transform!(
        calculate_uniques,
        repeat_test_big,
        file "../../../../../examples/repeatTestBig.rg",
        "@unique begin end win1 win2;"
    );

    test_transform!(
        calculate_uniques,
        tictactoe,
        file "../../../../../examples/ticTacToe.rg",
        "@unique begin check checkline checklineH1 checklineH2 checklineLR1 checklineLR2 checklineRL1 checklineRL2 checklineV1 checklineV2 checkwin chooseX chooseX(coordX: Coord) chooseY chooseY(coordY: Coord)                  endmove      nextturn        set      win win1 win2;"
        // TODO: Ideally everything would be `@unique`.
        // "@unique begin check checkline checklineH1 checklineH2 checklineLR1 checklineLR2 checklineRL1 checklineRL2 checklineV1 checklineV2 checkwin chooseX chooseX(coordX: Coord) chooseY chooseY(coordY: Coord) end endcheckline endmove move nextturn preend set turn win win1 win2;"
    );
}
