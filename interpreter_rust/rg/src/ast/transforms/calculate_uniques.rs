use crate::ast::analyses::ReachingAssignments;
use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_uniques(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_paths = self.analyse::<ReachingAssignments>(false);
        let mut unique_nodes: BTreeSet<_> = reaching_paths
            .into_iter()
            .filter(|(_, variables)| variables.values().all(|reached| !reached.is_repeated))
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

        if !unique_nodes.is_empty() {
            let pragma = Pragma::Unique {
                span: Span::none(),
                nodes: unique_nodes.into_iter().collect(),
            };
            let index = self.pragmas.partition_point(|x| *x < pragma);
            self.pragmas.insert(index, pragma);
        }

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
        adds "@unique begin end x;"
    );

    test_transform!(
        calculate_uniques,
        small_loop,
        "begin, x: ; x, y: ; y, x: ; y, end: ;",
        adds "@unique begin;"
    );

    test_transform!(
        calculate_uniques,
        repeat_example,
        "begin, a: ; a, a: x = y[x]; a, end: x == z;",
        adds "@unique begin;"
    );

    test_transform!(
        calculate_uniques,
        repeat_test,
        include_str!("../../../../../examples/repeatTest.rg"),
        adds "@unique begin end setScore win;"
    );

    test_transform!(
        calculate_uniques,
        repeat_test_big,
        include_str!("../../../../../examples/repeatTestBig.rg"),
        adds "@unique begin end;"
    );

    test_transform!(
        calculate_uniques,
        breakthrough,
        include_str!("../../../../../examples/breakthrough.rg"),
        adds "@unique begin checkOwn continue directionForward directionLeft directionLeftChecked directionOK directionRight directionRightChecked done end finish forward lose move moved score selectDirection selectPos selectedPos(position: Position) setFinished setPos(position: Position) turn win wincheck;"
    );

    test_transform!(
        calculate_uniques,
        tictactoe,
        include_str!("../../../../../examples/ticTacToe.rg"),
        adds "@unique begin check checkline checklineH1 checklineH2 checklineLR1 checklineLR2 checklineRL1 checklineRL2 checklineV1 checklineV2 checkwin chooseX chooseX(coordX: Coord) chooseY chooseY(coordY: Coord) end endmove move nextturn preend set turn win win1 win2;"
    );

    test_transform!(
        calculate_uniques,
        simple_apply_test_1,
        include_str!("../../../../../examples/simpleApplyTest1.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );

    test_transform!(
        calculate_uniques,
        simple_apply_test_2,
        include_str!("../../../../../examples/simpleApplyTest2.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );

    test_transform!(
        calculate_uniques,
        simple_apply_test_3,
        include_str!("../../../../../examples/simpleApplyTest3.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );
}
