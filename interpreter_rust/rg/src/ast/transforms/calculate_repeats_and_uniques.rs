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
        let reachable_nodes = self.analyse(&ReachableNodes::new());
        let reaching_assignments = self.analyse(&ReachingAssignments::from(&*self));

        // Temporary clone for `check_reachability`.
        let mut clone = Self::default();
        swap(&mut self.edges, &mut clone.edges);
        let check_reachability = clone.make_check_reachability(false);

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
                && !check_reachability(&node, &node).is_reachable()
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

        drop(check_reachability);
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
        hex_simple,
        "@disjointExhaustive a : b c;
        type Position = { 0, 1, 2 };
        const check: Position -> Bool = { :1 };
        const left: Position -> Position = { :0, 2: 1 };
        const right: Position -> Position = { :2, 0: 1 };
        var board: Position -> Bool = { :0 };
        var position: Position = 0;
        begin, end: ? a -> b;
        a, b: board[position] == 1;
        a, c: board[position] != 1;
        c, d: position = left[position];
        c, d: position = right[position];
        d, a: check[position] == 1;",
        adds "@repeat d : position; @unique a b begin c end;"
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
        breakthrough_rbg_tag_variable,
        "
            @disjoint 19 : 17 17;
            @translatedFromRbg;
            begin, 4: player = white;
            4, 7: coord = Coord(*);
            7, 10: board[coord] == w;
            10, 12: $$ coord;
            12, 9: board[coord] = e;
            9, 16: coord = direction_up[coord];
            16, 17: board[coord] == e;
            16, 19: coord = direction_left[coord];
            16, 19: coord = direction_right[coord];
            19, 17: board[coord] == b;
            19, 17: board[coord] == e;
            17, 27: $$ coord;
            27, 26: plaer = keeper;
        ",
        adds "@repeat 17 19 : coord; @unique 10 12 16 26 27 4 7 9 begin;"
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
        adds "@unique begin check checkForEmpty checkForEmptyX checkForEmptyY checkline checklineH1 checklineH2 checklineLR1 checklineLR2 checklineRL1 checklineRL2 checklineV1 checklineV2 checkwin chooseX chooseY choosenX choosenY emptyExists end endcheckline endmove move nextturn preend set turn win win1 win2;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        breakthrough,
        include_str!("../../../../../games/rg/breakthrough.rg"),
        adds "@unique begin checkOwn continue done end findPawn findPawnPos forwardDirCheck forwardDirSet forwardDirSetP forwardMove leftDirCheck leftDirSet leftDirSetP leftMove lose move moved pawnExists rightDirCheck rightDirSet rightDirSetP rightMove score selectDir selectPos setPos turn win wincheck;"
    );

    test_transform!(
        calculate_repeats_and_uniques,
        breakthrough_hrg_half,
        "
            @artificialTag F L R;
            @disjoint turn_10 : turn_6 turn_6;
            @disjointExhaustive turn_18 : turn_19 turn_21;
            @disjointExhaustive turn_21 : turn_19 turn_begin;
            type Piece = { blackPawn, empty, whitePawn, border };
            type Player = { black, white };
            type Score = { 0, 100 };
            type Position = { V__0_0, V__0_1, V__0_2, V__0_3, V__0_4, V__0_5, V__0_6, V__0_7, V__1_0, V__1_1, V__1_2, V__1_3, V__1_4, V__1_5, V__1_6, V__1_7, V__2_0, V__2_1, V__2_2, V__2_3, V__2_4, V__2_5, V__2_6, V__2_7, V__3_0, V__3_1, V__3_2, V__3_3, V__3_4, V__3_5, V__3_6, V__3_7, V__4_0, V__4_1, V__4_2, V__4_3, V__4_4, V__4_5, V__4_6, V__4_7, V__5_0, V__5_1, V__5_2, V__5_3, V__5_4, V__5_5, V__5_6, V__5_7, V__6_0, V__6_1, V__6_2, V__6_3, V__6_4, V__6_5, V__6_6, V__6_7, V__7_0, V__7_1, V__7_2, V__7_3, V__7_4, V__7_5, V__7_6, V__7_7 };
            type PositionOrNull = { null, V__0_0, V__0_1, V__0_2, V__0_3, V__0_4, V__0_5, V__0_6, V__0_7, V__1_0, V__1_1, V__1_2, V__1_3, V__1_4, V__1_5, V__1_6, V__1_7, V__2_0, V__2_1, V__2_2, V__2_3, V__2_4, V__2_5, V__2_6, V__2_7, V__3_0, V__3_1, V__3_2, V__3_3, V__3_4, V__3_5, V__3_6, V__3_7, V__4_0, V__4_1, V__4_2, V__4_3, V__4_4, V__4_5, V__4_6, V__4_7, V__5_0, V__5_1, V__5_2, V__5_3, V__5_4, V__5_5, V__5_6, V__5_7, V__6_0, V__6_1, V__6_2, V__6_3, V__6_4, V__6_5, V__6_6, V__6_7, V__7_0, V__7_1, V__7_2, V__7_3, V__7_4, V__7_5, V__7_6, V__7_7 };
            type Bool = { 0, 1 };
            type Goals = Player -> Score;
            type Visibility = Player -> Bool;
            type PlayerOrSystem = { black, white, keeper, random };
            const up: PositionOrNull -> PositionOrNull = { :V__7_0, null: null, V__0_0: V__0_0, V__0_1: V__0_0, V__0_2: V__0_1, V__0_3: V__0_2, V__0_4: V__0_3, V__0_5: V__0_4, V__0_6: V__0_5, V__0_7: V__0_6, V__1_0: V__1_0, V__1_1: V__1_0, V__1_2: V__1_1, V__1_3: V__1_2, V__1_4: V__1_3, V__1_5: V__1_4, V__1_6: V__1_5, V__1_7: V__1_6, V__2_0: V__2_0, V__2_1: V__2_0, V__2_2: V__2_1, V__2_3: V__2_2, V__2_4: V__2_3, V__2_5: V__2_4, V__2_6: V__2_5, V__2_7: V__2_6, V__3_0: V__3_0, V__3_1: V__3_0, V__3_2: V__3_1, V__3_3: V__3_2, V__3_4: V__3_3, V__3_5: V__3_4, V__3_6: V__3_5, V__3_7: V__3_6, V__4_0: V__4_0, V__4_1: V__4_0, V__4_2: V__4_1, V__4_3: V__4_2, V__4_4: V__4_3, V__4_5: V__4_4, V__4_6: V__4_5, V__4_7: V__4_6, V__5_0: V__5_0, V__5_1: V__5_0, V__5_2: V__5_1, V__5_3: V__5_2, V__5_4: V__5_3, V__5_5: V__5_4, V__5_6: V__5_5, V__5_7: V__5_6, V__6_0: V__6_0, V__6_1: V__6_0, V__6_2: V__6_1, V__6_3: V__6_2, V__6_4: V__6_3, V__6_5: V__6_4, V__6_6: V__6_5, V__6_7: V__6_6, V__7_2: V__7_1, V__7_3: V__7_2, V__7_4: V__7_3, V__7_5: V__7_4, V__7_6: V__7_5, V__7_7: V__7_6 };
            const upLeft: PositionOrNull -> PositionOrNull = { :null, V__1_0: V__0_0, V__1_1: V__0_0, V__1_2: V__0_1, V__1_3: V__0_2, V__1_4: V__0_3, V__1_5: V__0_4, V__1_6: V__0_5, V__1_7: V__0_6, V__2_0: V__1_0, V__2_1: V__1_0, V__2_2: V__1_1, V__2_3: V__1_2, V__2_4: V__1_3, V__2_5: V__1_4, V__2_6: V__1_5, V__2_7: V__1_6, V__3_0: V__2_0, V__3_1: V__2_0, V__3_2: V__2_1, V__3_3: V__2_2, V__3_4: V__2_3, V__3_5: V__2_4, V__3_6: V__2_5, V__3_7: V__2_6, V__4_0: V__3_0, V__4_1: V__3_0, V__4_2: V__3_1, V__4_3: V__3_2, V__4_4: V__3_3, V__4_5: V__3_4, V__4_6: V__3_5, V__4_7: V__3_6, V__5_0: V__4_0, V__5_1: V__4_0, V__5_2: V__4_1, V__5_3: V__4_2, V__5_4: V__4_3, V__5_5: V__4_4, V__5_6: V__4_5, V__5_7: V__4_6, V__6_0: V__5_0, V__6_1: V__5_0, V__6_2: V__5_1, V__6_3: V__5_2, V__6_4: V__5_3, V__6_5: V__5_4, V__6_6: V__5_5, V__6_7: V__5_6, V__7_0: V__6_0, V__7_1: V__6_0, V__7_2: V__6_1, V__7_3: V__6_2, V__7_4: V__6_3, V__7_5: V__6_4, V__7_6: V__6_5, V__7_7: V__6_6 };
            const upRight: PositionOrNull -> PositionOrNull = { :null, V__0_0: V__1_0, V__0_1: V__1_0, V__0_2: V__1_1, V__0_3: V__1_2, V__0_4: V__1_3, V__0_5: V__1_4, V__0_6: V__1_5, V__0_7: V__1_6, V__1_0: V__2_0, V__1_1: V__2_0, V__1_2: V__2_1, V__1_3: V__2_2, V__1_4: V__2_3, V__1_5: V__2_4, V__1_6: V__2_5, V__1_7: V__2_6, V__2_0: V__3_0, V__2_1: V__3_0, V__2_2: V__3_1, V__2_3: V__3_2, V__2_4: V__3_3, V__2_5: V__3_4, V__2_6: V__3_5, V__2_7: V__3_6, V__3_0: V__4_0, V__3_1: V__4_0, V__3_2: V__4_1, V__3_3: V__4_2, V__3_4: V__4_3, V__3_5: V__4_4, V__3_6: V__4_5, V__3_7: V__4_6, V__4_0: V__5_0, V__4_1: V__5_0, V__4_2: V__5_1, V__4_3: V__5_2, V__4_4: V__5_3, V__4_5: V__5_4, V__4_6: V__5_5, V__4_7: V__5_6, V__5_0: V__6_0, V__5_1: V__6_0, V__5_2: V__6_1, V__5_3: V__6_2, V__5_4: V__6_3, V__5_5: V__6_4, V__5_6: V__6_5, V__5_7: V__6_6, V__6_0: V__7_0, V__6_1: V__7_0, V__6_2: V__7_1, V__6_3: V__7_2, V__6_4: V__7_3, V__6_5: V__7_4, V__6_6: V__7_5, V__6_7: V__7_6 };
            var board: PositionOrNull -> Piece = { :empty, V__0_0: blackPawn, V__0_1: blackPawn, V__0_6: whitePawn, V__0_7: whitePawn, V__1_0: blackPawn, V__1_1: blackPawn, V__1_6: whitePawn, V__1_7: whitePawn, V__2_0: blackPawn, V__2_1: blackPawn, V__2_6: whitePawn, V__2_7: whitePawn, V__3_0: blackPawn, V__3_1: blackPawn, V__3_6: whitePawn, V__3_7: whitePawn, V__4_0: blackPawn, V__4_1: blackPawn, V__4_6: whitePawn, V__4_7: whitePawn, V__5_0: blackPawn, V__5_1: blackPawn, V__5_6: whitePawn, V__5_7: whitePawn, V__6_0: blackPawn, V__6_1: blackPawn, V__6_6: whitePawn, V__6_7: whitePawn, V__7_0: blackPawn, V__7_1: blackPawn, V__7_6: whitePawn, V__7_7: whitePawn, null: border };
            var pos: PositionOrNull = null;
            var goals: Goals = { :0 };
            var player: PlayerOrSystem = keeper;
            var visible: Visibility = { :1 };
            begin, turn_begin: ;
            turn_begin, turn_1: player = Player(white);
            turn_1, turn_2: pos = Position(*);
            turn_2, turn_3: board[pos] == Piece(whitePawn);
            turn_3, turn_4: board[pos] = empty;
            turn_4, turn_5: $$ pos;
            turn_5, turn_7: pos = up[pos];
            turn_7, turn_8: $ F;
            turn_8, turn_6: board[pos] == empty;
            turn_5, turn_11: pos = upLeft[pos];
            turn_11, turn_10: $ L;
            turn_5, turn_13: pos = upRight[pos];
            turn_13, turn_10: $ R;
            turn_10, turn_6: board[pos] == empty;
            turn_10, turn_6: board[pos] == Piece(blackPawn);
            turn_6, turn_16: $$ pos;
            turn_16, turn_17: player = keeper;
            turn_17, turn_18: board[pos] = Piece(whitePawn);
            turn_18, turn_19: up[pos] == pos;
            turn_18, turn_21: up[pos] != pos;
            existsOppPawn_call_1, turn_22_existsOppPawn_1: pos = Position(*);
            turn_22_existsOppPawn_1, existsOppPawn_end: board[pos] == Piece(blackPawn);
            turn_21, turn_begin: ? existsOppPawn_call_1 -> existsOppPawn_end;
            turn_21, turn_19: ! existsOppPawn_call_1 -> existsOppPawn_end;
            turn_19, turn_23: goals[Player(white)] = 100;
            turn_23, end: player = keeper;
        ",
        adds "@unique begin end existsOppPawn_call_1 existsOppPawn_end turn_1 turn_10 turn_11 turn_13 turn_16 turn_17 turn_18 turn_19 turn_2 turn_21 turn_22_existsOppPawn_1 turn_23 turn_3 turn_4 turn_5 turn_6 turn_7 turn_8 turn_begin;"
    );
}
