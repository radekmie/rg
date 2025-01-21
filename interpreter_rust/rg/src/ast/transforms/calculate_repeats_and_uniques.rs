use crate::ast::analyses::{ReachableNodes, ReachingAssignments};
use crate::ast::{Error, Game, Node, Pragma};
use std::collections::BTreeSet;
use std::mem::swap;
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;

impl Game<Id> {
    pub fn calculate_repeats_and_uniques(&mut self) -> Result<(), Error<Id>> {
        let has_next_edges: BTreeSet<_> = self.edges.iter().map(|edge| edge.lhs.clone()).collect();
        let reachable_nodes = self.analyse::<ReachableNodes>(false);
        let reaching_assignments = self.analyse::<ReachingAssignments>(false);

        // Temporary clone for `is_reachable`.
        let mut clone = Self::default();
        swap(&mut self.edges, &mut clone.edges);
        let is_reachable = clone.make_is_reachable();

        // Sort existing `@repeat`s.
        for pragma in &mut self.pragmas {
            if let Pragma::Repeat {
                nodes, identifiers, ..
            } = pragma
            {
                identifiers.sort_unstable();
                nodes.sort_unstable();
            }
        }

        // Collect existing `@unique`s.
        let mut unique_nodes = BTreeSet::new();
        self.pragmas.retain(|pragma| {
            if let Pragma::Unique { nodes, .. } = pragma {
                unique_nodes.extend(nodes.iter().cloned());
                false
            } else {
                true
            }
        });

        for (node, variables) in reaching_assignments {
            // If it was marked as unique, trust it.
            if unique_nodes.contains(&node) {
                continue;
            }

            // If there are no next edges, consider it unique.
            if !has_next_edges.contains(&node) {
                unique_nodes.insert(node);
                continue;
            }

            let has_empty_repeat = variables
                .get(&None)
                .is_some_and(|assignment| assignment.is_repeated);
            let identifiers: Vec<_> = variables
                .into_iter()
                .filter(|(_, assignment)| has_empty_repeat || assignment.is_repeated)
                .filter_map(|(variable, _)| variable)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            // If there's nothing to repeat, consider it unique.
            if !has_empty_repeat && identifiers.is_empty() {
                unique_nodes.insert(node);
                continue;
            }

            // Entire subautomatas are unique as long as they're not on cycles.
            if reachable_nodes
                .get(&node)
                .is_none_or(|reachable| !*reachable)
                && !is_reachable(&node, &node)
            {
                unique_nodes.insert(node);
                continue;
            }

            // Add `@repeat`.
            self.add_repeat(node, identifiers);
        }

        // Add `@unique`.
        if !unique_nodes.is_empty() {
            self.add_pragma(Pragma::Unique {
                span: Span::none(),
                nodes: unique_nodes.into_iter().collect(),
            });
        }

        drop(is_reachable);
        swap(&mut self.edges, &mut clone.edges);
        Ok(())
    }

    fn add_repeat(&mut self, node: Node<Id>, identifiers: Vec<Id>) {
        // Merge with existing `@repeat` if possible.
        for pragma in &mut self.pragmas {
            if let Pragma::Repeat {
                nodes,
                identifiers: ids,
                ..
            } = pragma
            {
                if *ids == identifiers {
                    if let Err(index) = nodes.binary_search(&node) {
                        nodes.insert(index, node);
                    }
                    return;
                }
            }
        }

        self.add_pragma(Pragma::Repeat {
            span: Span::none(),
            nodes: vec![node],
            identifiers,
        });
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_repeats_and_uniques,
        small_unique,
        "begin, x: ; x, end: ;",
        adds "@unique begin end x;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_example,
        "begin, a: ; a, a: x = y[x]; a, end: x == z;",
        adds "@repeat a : x; @unique begin end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_example_rbg_board,
        "begin, a: ; a, a: board[coord] = b; a, end: board[coord] == b;",
        adds "@repeat a : board; @unique begin end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_example_rbg_board_with_pragma,
        "begin, a: ; a, a: board[coord] = b; a, end: board[coord] == b; @translatedFromRbg;",
        adds "@repeat a :; @unique begin end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_example_rbg_coord,
        "begin, a: ; a, a: coord = rx0y0; a, end: coord == rx0y0;",
        adds "@repeat a : coord; @unique begin end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_example_rbg_coord_with_pragma,
        "begin, a: ; a, a: coord = rx0y0; a, end: coord == rx0y0; @translatedFromRbg;",
        adds "@repeat a : coord; @unique begin end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        small_loop,
        "begin, x: ; x, y: ; y, x: ; y, end: ;",
        adds "@repeat x :; @unique begin end y;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        breakthrough_join,
        "
            begin, end: ? 4_30_4 -> 4_30_30;
            4_30_4, 4_30_9(bind_Coord_17: Coord): Coord(bind_Coord_17) != Coord(null);
            4_30_9(bind_Coord_17: Coord), 4_30_8: coord = Coord(bind_Coord_17);
            4_30_8, 130: board[coord] == w;
            130, 4_30_19: board[coord] = e;
            4_30_19, 4_30_17: direction[up][coord] != null;
            4_30_17, 4_30_30: board[direction[up][coord]] == e;
            4_30_17, 4_30_25: coord = direction[left][direction[up][coord]];
            4_30_17, 4_30_25: coord = direction[right][direction[up][coord]];
            4_30_25, 4_30_22: coord != null;
            4_30_22, 4_30_30: board[coord] != Piece(w);
        ",
        adds "@unique 130 4_30_17 4_30_19 4_30_22 4_30_25 4_30_30 4_30_4 4_30_8 4_30_9(bind_Coord_17: Coord) begin end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
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
        adds "@repeat 27 32 : coord; @unique 24 25 26 28 30 46 47 begin end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
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
        adds "@repeat e : x z; @unique a b begin c1 c2 d1 d2 end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
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
        ",
        adds "@unique a b begin c1 c2 d1 d2 e end;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_multiple,
        "
            begin, choice: x = 0;
            begin, choice: x = 1;
            choice, joined: ;
            joined, end: ;
        ",
        adds "@repeat choice : x; @unique begin end joined;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
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
        calculate_repeats_and_uniques,
        oware_double_loop,
        "
            begin, 151: ;
            132, 151: startSowing = currHole;
            151, 152: toSow = board[currHole];
            152, 153: board[currHole] = I__0;
            153, 154: toSow != I__0;
            153, end: toSow == I__0;
            154, 156: currHole = nextHole[currHole];
            156, 157: startSowing != currHole;
            156, 153: startSowing == currHole;
            157, 159: board[currHole] = incr[board[currHole]];
            159, 153: toSow = decr[toSow];
        ",
        adds "
            @repeat 153 : board startSowing toSow;
            @repeat 154 157 159 : board toSow;
            @repeat 156 : board currHole toSow;
            @unique 132 151 152 begin end;
        "
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_test,
        include_str!("../../../../../games/rg/repeatTest.rg"),
        adds "@repeat selectDir4 : pos; @unique begin end setScore win;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        repeat_test_big,
        include_str!("../../../../../games/rg/repeatTestBig.rg"),
        adds "@repeat goDown goLeft goRight goUp main : pos; @unique begin end setScore win1 win1Tag win2 win2Tag;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        simple_apply_test_1,
        include_str!("../../../../../games/rg/simpleApplyTest1.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        simple_apply_test_2,
        include_str!("../../../../../games/rg/simpleApplyTest2.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        simple_apply_test_3,
        include_str!("../../../../../games/rg/simpleApplyTest3.rg"),
        adds "@unique begin doneA doneB end extraB moveA moveB preend tagA0 tagA1 tagB0 tagB0same tagB1 tagB1same;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        tictactoe,
        include_str!("../../../../../games/rg/ticTacToe.rg"),
        adds "@unique begin check checkline checklineH1 checklineH2 checklineLR1 checklineLR2 checklineRL1 checklineRL2 checklineV1 checklineV2 checkwin chooseX chooseX(coordX: Coord) chooseY chooseY(coordY: Coord) end endcheckline endmove move nextturn preend set turn win win1 win2;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        breakthrough,
        include_str!("../../../../../games/rg/breakthrough.rg"),
        adds "@unique begin checkOwn continue done end findPawn findPawnPos(position: Position) forwardDirCheck forwardDirSet forwardDirSetP forwardMove leftDirCheck leftDirSet leftDirSetP leftMove lose move moved pawnExists rightDirCheck rightDirSet rightDirSetP rightMove score selectDir selectPos setPos(position: Position) turn win wincheck;"
    );
}
