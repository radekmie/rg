use crate::ast::analyses::ReachableNodes;
use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn prune_unreachable_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        let reachable_nodes = self.analyse(&ReachableNodes::new_with_reachability());
        self.edges.retain(|edge| {
            reachable_nodes
                .get(&edge.lhs)
                .is_some_and(|reachable| *reachable)
        });

        let next_edges = self.next_edges();
        self.edges
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, edge)| {
                !edge.rhs.is_end()
                    && !next_edges.contains_key(&edge.rhs)
                    && !self.is_reachability_target(&edge.rhs)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|index| {
                self.edges.remove(index);
            });

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(prune_unreachable_nodes, simple, 
        "type Piece = { empty, red, yellow, border };
type Player = { red, yellow };
type Position = { V__1_1, V__1_2, V__1_3, V__1_4, V__1_5, V__1_6, V__2_1, V__2_2, V__2_3, V__2_4, V__2_5, V__2_6, V__3_1, V__3_2, V__3_3, V__3_4, V__3_5, V__3_6, V__4_1, V__4_2, V__4_3, V__4_4, V__4_5, V__4_6, V__5_1, V__5_2, V__5_3, V__5_4, V__5_5, V__5_6, V__6_1, V__6_2, V__6_3, V__6_4, V__6_5, V__6_6, V__7_1, V__7_2, V__7_3, V__7_4, V__7_5, V__7_6, null };
type TopPosition = { V__1_6, V__2_6, V__3_6, V__4_6, V__5_6, V__6_6, V__7_6 };
type Score = { 50, 0, 100 };
type Bool = { 0, 1 };
type Goals = Player -> Score;
type Visibility = Player -> Bool;
type PlayerOrSystem = { red, yellow, keeper, random };
const up: Position -> Position = { :null, V__1_1: V__1_2, V__1_2: V__1_3, V__1_3: V__1_4, V__1_4: V__1_5, V__1_5: V__1_6, V__2_1: V__2_2, V__2_2: V__2_3, V__2_3: V__2_4, V__2_4: V__2_5, V__2_5: V__2_6, V__3_1: V__3_2, V__3_2: V__3_3, V__3_3: V__3_4, V__3_4: V__3_5, V__3_5: V__3_6, V__4_1: V__4_2, V__4_2: V__4_3, V__4_3: V__4_4, V__4_4: V__4_5, V__4_5: V__4_6, V__5_1: V__5_2, V__5_2: V__5_3, V__5_3: V__5_4, V__5_4: V__5_5, V__5_5: V__5_6, V__6_1: V__6_2, V__6_2: V__6_3, V__6_3: V__6_4, V__6_4: V__6_5, V__6_5: V__6_6, V__7_1: V__7_2, V__7_2: V__7_3, V__7_3: V__7_4, V__7_4: V__7_5, V__7_5: V__7_6 };
const right: Position -> Position = { :null, V__1_1: V__2_1, V__1_2: V__2_2, V__1_3: V__2_3, V__1_4: V__2_4, V__1_5: V__2_5, V__1_6: V__2_6, V__2_1: V__3_1, V__2_2: V__3_2, V__2_3: V__3_3, V__2_4: V__3_4, V__2_5: V__3_5, V__2_6: V__3_6, V__3_1: V__4_1, V__3_2: V__4_2, V__3_3: V__4_3, V__3_4: V__4_4, V__3_5: V__4_5, V__3_6: V__4_6, V__4_1: V__5_1, V__4_2: V__5_2, V__4_3: V__5_3, V__4_4: V__5_4, V__4_5: V__5_5, V__4_6: V__5_6, V__5_1: V__6_1, V__5_2: V__6_2, V__5_3: V__6_3, V__5_4: V__6_4, V__5_5: V__6_5, V__5_6: V__6_6, V__6_1: V__7_1, V__6_2: V__7_2, V__6_3: V__7_3, V__6_4: V__7_4, V__6_5: V__7_5, V__6_6: V__7_6 };
const down: Position -> Position = { :null, V__1_2: V__1_1, V__1_3: V__1_2, V__1_4: V__1_3, V__1_5: V__1_4, V__1_6: V__1_5, V__2_2: V__2_1, V__2_3: V__2_2, V__2_4: V__2_3, V__2_5: V__2_4, V__2_6: V__2_5, V__3_2: V__3_1, V__3_3: V__3_2, V__3_4: V__3_3, V__3_5: V__3_4, V__3_6: V__3_5, V__4_2: V__4_1, V__4_3: V__4_2, V__4_4: V__4_3, V__4_5: V__4_4, V__4_6: V__4_5, V__5_2: V__5_1, V__5_3: V__5_2, V__5_4: V__5_3, V__5_5: V__5_4, V__5_6: V__5_5, V__6_2: V__6_1, V__6_3: V__6_2, V__6_4: V__6_3, V__6_5: V__6_4, V__6_6: V__6_5, V__7_2: V__7_1, V__7_3: V__7_2, V__7_4: V__7_3, V__7_5: V__7_4, V__7_6: V__7_5 };
const left: Position -> Position = { :null, V__2_1: V__1_1, V__2_2: V__1_2, V__2_3: V__1_3, V__2_4: V__1_4, V__2_5: V__1_5, V__2_6: V__1_6, V__3_1: V__2_1, V__3_2: V__2_2, V__3_3: V__2_3, V__3_4: V__2_4, V__3_5: V__2_5, V__3_6: V__2_6, V__4_1: V__3_1, V__4_2: V__3_2, V__4_3: V__3_3, V__4_4: V__3_4, V__4_5: V__3_5, V__4_6: V__3_6, V__5_1: V__4_1, V__5_2: V__4_2, V__5_3: V__4_3, V__5_4: V__4_4, V__5_5: V__4_5, V__5_6: V__4_6, V__6_1: V__5_1, V__6_2: V__5_2, V__6_3: V__5_3, V__6_4: V__5_4, V__6_5: V__5_5, V__6_6: V__5_6, V__7_1: V__6_1, V__7_2: V__6_2, V__7_3: V__6_3, V__7_4: V__6_4, V__7_5: V__6_5, V__7_6: V__6_6 };
const opponent: Player -> Player = { :yellow, yellow: red };
var board: Position -> Piece = { :empty, null: border };
var pos: Position = V__1_1;
var me: Player = red;
var goals: Goals = { :50 };
var player: PlayerOrSystem = keeper;
var visible: Visibility = { :1 };
begin, rules_begin: ;
rules_begin, rules_1: ;
rules_1, turn_begin: ;
turn_begin, turn_1: player = me;
turn_1, turn_2: pos = TopPosition(*);
turn_2, turn_3: board[pos] == empty;
turn_3, turn_4: $$ pos;
turn_4, turn_5: player = keeper;
turn_5, goDown_begin: ;
goDown_begin, goDown_3: board[down[pos]] != empty;
goDown_begin, goDown_4: board[down[pos]] == empty;
goDown_3, goDown_end: ;
goDown_4, goDown_5: pos = down[pos];
goDown_5, goDown_2: ;
goDown_2, goDown_7: board[down[pos]] != empty;
goDown_2, goDown_8: board[down[pos]] == empty;
goDown_7, goDown_end: ;
goDown_8, goDown_9: pos = down[pos];
goDown_9, goDown_6: ;
goDown_6, goDown_11: board[down[pos]] != empty;
goDown_6, goDown_12: board[down[pos]] == empty;
goDown_11, goDown_end: ;
goDown_12, goDown_13: pos = down[pos];
goDown_13, goDown_10: ;
goDown_10, goDown_15: board[down[pos]] != empty;
goDown_10, goDown_16: board[down[pos]] == empty;
goDown_15, goDown_end: ;
goDown_16, goDown_17: pos = down[pos];
goDown_17, goDown_14: ;
goDown_14, goDown_19: board[down[pos]] != empty;
goDown_14, goDown_20: board[down[pos]] == empty;
goDown_19, goDown_end: ;
goDown_20, goDown_21: pos = down[pos];
goDown_21, goDown_18: ;
goDown_18, goDown_1: ;
goDown_1, goDown_end: ;
goDown_end, turn_6: ;
win_call_1, win_begin: ;
win_begin, turn_9_win_2: board[up[left[pos]]] == me;
turn_9_win_2, turn_9_win_3: board[up[left[up[left[pos]]]]] == me;
turn_9_win_3, turn_9_win_5: board[up[left[up[left[up[left[pos]]]]]]] == me;
turn_9_win_5, turn_9_win_4: ;
turn_9_win_3, turn_9_win_6: board[down[right[pos]]] == me;
turn_9_win_6, turn_9_win_4: ;
turn_9_win_4, turn_9_win_1: ;
win_begin, turn_9_win_7: board[left[pos]] == me;
turn_9_win_7, turn_9_win_8: board[left[left[pos]]] == me;
turn_9_win_8, turn_9_win_10: board[left[left[left[pos]]]] == me;
turn_9_win_10, turn_9_win_9: ;
turn_9_win_8, turn_9_win_11: board[right[pos]] == me;
turn_9_win_11, turn_9_win_9: ;
turn_9_win_9, turn_9_win_1: ;
win_begin, turn_9_win_12: board[down[left[pos]]] == me;
turn_9_win_12, turn_9_win_13: board[down[left[down[left[pos]]]]] == me;
turn_9_win_13, turn_9_win_15: board[down[left[down[left[down[left[pos]]]]]]] == me;
turn_9_win_15, turn_9_win_14: ;
turn_9_win_13, turn_9_win_16: board[up[right[pos]]] == me;
turn_9_win_16, turn_9_win_14: ;
turn_9_win_14, turn_9_win_1: ;
win_begin, turn_9_win_17: board[down[pos]] == me;
turn_9_win_17, turn_9_win_18: board[down[down[pos]]] == me;
turn_9_win_18, turn_9_win_19: board[down[down[down[pos]]]] == me;
turn_9_win_19, turn_9_win_1: ;
win_begin, turn_9_win_20: board[down[right[pos]]] == me;
turn_9_win_20, turn_9_win_21: board[down[right[down[right[pos]]]]] == me;
turn_9_win_21, turn_9_win_23: board[down[right[down[right[down[right[pos]]]]]]] == me;
turn_9_win_23, turn_9_win_22: ;
turn_9_win_21, turn_9_win_24: board[up[left[pos]]] == me;
turn_9_win_24, turn_9_win_22: ;
turn_9_win_22, turn_9_win_1: ;
win_begin, turn_9_win_25: board[right[pos]] == me;
turn_9_win_25, turn_9_win_26: board[right[right[pos]]] == me;
turn_9_win_26, turn_9_win_28: board[right[right[right[pos]]]] == me;
turn_9_win_28, turn_9_win_27: ;
turn_9_win_26, turn_9_win_29: board[left[pos]] == me;
turn_9_win_29, turn_9_win_27: ;
turn_9_win_27, turn_9_win_1: ;
win_begin, turn_9_win_30: board[up[right[pos]]] == me;
turn_9_win_30, turn_9_win_31: board[up[right[up[right[pos]]]]] == me;
turn_9_win_31, turn_9_win_33: board[up[right[up[right[up[right[pos]]]]]]] == me;
turn_9_win_33, turn_9_win_32: ;
turn_9_win_31, turn_9_win_34: board[down[left[pos]]] == me;
turn_9_win_34, turn_9_win_32: ;
turn_9_win_32, turn_9_win_1: ;
turn_9_win_1, win_end: ;
turn_6, turn_7: ? win_call_1 -> win_end;
turn_6, turn_8: ! win_call_1 -> win_end;
turn_7, turn_10: goals[me] = 100;
turn_10, turn_11: goals[opponent[me]] = 0;
turn_11, end: player = keeper;
turn_8, turn_12: board[pos] = me;
existsNonempty_call_1, existsNonempty_begin: ;
existsNonempty_begin, turn_15_existsNonempty_1: pos = TopPosition(*);
turn_15_existsNonempty_1, turn_15_existsNonempty_2: board[pos] == empty;
turn_15_existsNonempty_2, existsNonempty_end: ;
turn_12, turn_14: ? existsNonempty_call_1 -> existsNonempty_end;
turn_12, turn_13: ! existsNonempty_call_1 -> existsNonempty_end;
turn_13, end: player = keeper;
turn_14, turn_16: me = opponent[me];
turn_16, turn_end: ;
turn_end, rules_3: ;
rules_3, rules_1: ;
rules_2, rules_end: ;
rules_end, end: ;

"
    
    );

}