use crate::ast::analyses::ReachingAssignments;
use crate::ast::{Error, Game, Pragma};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_repeats(&mut self) -> Result<(), Error<Arc<str>>> {
        let has_next_edges: BTreeSet<_> = self.edges.iter().map(|edge| edge.lhs.clone()).collect();
        let reaching_paths = self.analyse::<ReachingAssignments>(false);
        for (node, variables) in reaching_paths {
            if !has_next_edges.contains(&node) {
                continue;
            }

            let has_none_repeat = variables
                .get(&None)
                .is_some_and(|reached| reached.is_repeated);
            let identifiers: Vec<_> = variables
                .into_iter()
                .filter(|(_, reached)| has_none_repeat || reached.is_repeated)
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
                self.add_pragma(Pragma::Repeat {
                    span: Span::none(),
                    nodes: vec![node],
                    identifiers,
                });
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
        adds "@repeat a : x;"
    );

    test_transform!(
        calculate_repeats,
        repeat_example_rbg_board,
        "begin, a: ; a, a: board[coord] = b; a, end: board[coord] == b;",
        adds "@repeat a : board;"
    );

    test_transform!(
        calculate_repeats,
        repeat_example_rbg_board_with_pragma,
        "begin, a: ; a, a: board[coord] = b; a, end: board[coord] == b; @translatedFromRbg;",
        adds "@repeat a :;"
    );

    test_transform!(
        calculate_repeats,
        repeat_example_rbg_coord,
        "begin, a: ; a, a: coord = rx0y0; a, end: coord == rx0y0;",
        adds "@repeat a : coord;"
    );

    test_transform!(
        calculate_repeats,
        repeat_example_rbg_coord_with_pragma,
        "begin, a: ; a, a: coord = rx0y0; a, end: coord == rx0y0; @translatedFromRbg;",
        adds "@repeat a : coord;"
    );

    test_transform!(
        calculate_repeats,
        small_loop,
        "begin, x: ; x, y: ; y, x: ; y, end: ;",
        adds "@repeat x :;"
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
        adds "@repeat 26 27 32 : coord;"
    );

    test_transform!(
        calculate_repeats,
        overlapping_variable_setters,
        "
            var x: Bool = 0;
            var y: Bool = 0;
            var z: Bool = 0;
            begin, a: ;
            a, b: x = 1;
            b, c1: y == 0;
            b, c2: y == 1;
            c1, d1: z = 0;
            c2, d2: z = 1;
            d1, e: ;
            d2, e: ;
            e, end: ;
        ",
        adds "@repeat e : x z;"
    );

    test_transform!(
        calculate_repeats,
        base_on_disjoint_pragma,
        "
            @disjoint b : c1 c2;
            var x: Bool = 0;
            var y: Bool = 0;
            var z: Bool = 0;
            begin, a: ;
            a, b: x = 1;
            b, c1: y == 0;
            b, c2: y == 1;
            c1, d1: z = 0;
            c2, d2: z = 1;
            d1, e: ;
            d2, e: ;
            e, end: ;
        "
    );

    test_transform!(
        calculate_repeats,
        repeat_multiple,
        "
            begin, choice: x = 0;
            begin, choice: x = 1;
            choice, joined: ;
            joined, end: ;
        ",
        adds "@repeat choice : x;"
    );

    test_transform!(
        calculate_repeats,
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
        "
    );

    test_transform!(
        calculate_repeats,
        repeat_test,
        include_str!("../../../../../games/rg/repeatTest.rg"),
        adds "@repeat selectDir4 : pos;"
    );

    test_transform!(
        calculate_repeats,
        repeat_test_big,
        include_str!("../../../../../games/rg/repeatTestBig.rg"),
        adds "@repeat goDown goLeft goRight goUp main : pos; @repeat setScore :;"
    );

    test_transform!(
        calculate_repeats,
        tictactoe,
        include_str!("../../../../../games/rg/ticTacToe.rg")
    );
}
