use crate::ast::analyses::ReachingAssignments;
use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_uniques(&mut self) -> Result<(), Error<Arc<str>>> {
        let has_next_edges: BTreeSet<_> = self.edges.iter().map(|edge| edge.lhs.clone()).collect();
        let reaching_paths = self.analyse::<ReachingAssignments>(false);
        let mut unique_nodes: BTreeSet<_> = reaching_paths
            .into_iter()
            .filter(|(node, variables)| {
                !has_next_edges.contains(node)
                    || variables.values().all(|reached| !reached.is_repeated)
            })
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
            self.add_pragma(Pragma::Unique {
                span: Span::none(),
                nodes: unique_nodes.into_iter().collect(),
            });
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
        adds "@unique begin end y;"
    );

    test_transform!(
        calculate_uniques,
        repeat_example,
        "begin, a: ; a, a: x = y[x]; a, end: x == z;",
        adds "@unique begin end;"
    );

    test_transform!(
        calculate_uniques,
        repeat_multiple,
        "
            begin, choice: x = 0;
            begin, choice: x = 1;
            choice, joined: ;
            joined, end: ;
        ",
        adds "@unique begin end joined;"
    );

    test_transform!(
        calculate_uniques,
        tictactoe_hrg_condition,
        "
            begin, end: ? win_call_1 -> win_end;
            win_call_1, win_2: position != next_d1[position];
            win_2, win_3: board[position] == board[next_d1[position]];
            win_3, win_end: board[position] == board[__gen_next_d1_next_d1[position]];
            win_call_1, win_5: position != next_d2[position];
            win_5, win_6: board[position] == board[next_d2[position]];
            win_6, win_end: board[position] == board[__gen_next_d2_next_d2[position]];
            win_call_1, win_8: board[position] == board[next_h[position]];
            win_8, win_end: board[position] == board[__gen_next_h_next_h[position]];
            win_call_1, win_10: board[position] == board[next_v[position]];
            win_10, win_end: board[position] == board[__gen_next_v_next_v[position]];
        ",
        adds "@unique begin end win_10 win_2 win_3 win_5 win_6 win_8 win_call_1 win_end;"
    );

    test_transform!(
        calculate_uniques,
        repeat_test,
        include_str!("../../../../../games/rg/repeatTest.rg"),
        adds "@unique begin end setScore win;"
    );

    test_transform!(
        calculate_uniques,
        repeat_test_big,
        include_str!("../../../../../games/rg/repeatTestBig.rg"),
        adds "@unique begin end setScore win1 win1Tag win2 win2Tag;"
    );

    test_transform!(
        calculate_uniques,
        breakthrough,
        include_str!("../../../../../games/rg/breakthrough.rg"),
        adds "@unique begin checkOwn continue directionForward directionLeft directionLeftChecked directionOK directionRight directionRightChecked done end finish forward lose move moved score selectDirection selectPos selectedPos(position: Position) setFinished setPos(position: Position) turn win wincheck;"
    );

    test_transform!(
        calculate_uniques,
        tictactoe,
        include_str!("../../../../../games/rg/ticTacToe.rg"),
        adds "@unique begin check checkline checklineH1 checklineH2 checklineLR1 checklineLR2 checklineRL1 checklineRL2 checklineV1 checklineV2 checkwin chooseX chooseX(coordX: Coord) chooseY chooseY(coordY: Coord) end endcheckline endmove move nextturn preend set turn win win1 win2;"
    );

    test_transform!(
        calculate_uniques,
        simple_apply_test_1,
        include_str!("../../../../../games/rg/simpleApplyTest1.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );

    test_transform!(
        calculate_uniques,
        simple_apply_test_2,
        include_str!("../../../../../games/rg/simpleApplyTest2.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );

    test_transform!(
        calculate_uniques,
        simple_apply_test_3,
        include_str!("../../../../../games/rg/simpleApplyTest3.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );
}
