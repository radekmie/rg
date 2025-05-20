use crate::ast::{Edge, Error, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn compact_reachability(&mut self) -> Result<(), Error<Id>> {
        let (reachability_starts, reachability_ends) = self.reachability_starts_ends();
        let next_edges = self.next_edges();
        let prev_edges = self.prev_edges();

        let starts_moved = reachability_to_move(
            reachability_starts,
            &next_edges,
            |edge| edge.label.is_skip(),
            |edge| &edge.rhs,
        );

        let ends_moved = reachability_to_move(
            reachability_ends,
            &prev_edges,
            |edge| edge.label.is_skip() || edge.label.is_assignment(),
            |edge| &edge.lhs,
        );

        if starts_moved.is_empty() && ends_moved.is_empty() {
            return Ok(());
        }

        for edge in &mut self.edges {
            if let Label::Reachability {
                lhs,
                rhs,
                span,
                negated,
            } = &edge.label
            {
                let new_lhs = starts_moved.get(lhs);
                let new_rhs = ends_moved.get(rhs);
                if new_lhs.is_some() || new_rhs.is_some() {
                    let new_lhs = new_lhs.unwrap_or(&lhs);
                    let new_rhs = new_rhs.unwrap_or(&rhs);
                    Arc::make_mut(edge).label = Label::Reachability {
                        lhs: (*new_lhs).clone(),
                        rhs: (*new_rhs).clone(),
                        span: *span,
                        negated: *negated,
                    };
                }
            }
        }

        Ok(())
    }

    fn reachability_starts_ends(&self) -> (BTreeSet<&Node<Id>>, BTreeSet<&Node<Id>>) {
        let mut lhss = BTreeSet::new();
        let mut rhss = BTreeSet::new();
        for edge in self.edges.iter() {
            if let Label::Reachability { lhs, rhs, .. } = &edge.label {
                lhss.insert(lhs);
                rhss.insert(rhs);
            }
        }
        (lhss, rhss)
    }
}

fn reachability_to_move(
    reachability_targets: BTreeSet<&Node<Id>>,
    next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    condition: impl Fn(&Arc<Edge<Id>>) -> bool,
    step: impl Fn(&Arc<Edge<Id>>) -> &Node<Id>,
) -> BTreeMap<Node<Id>, Node<Id>> {
    let mut to_move: BTreeMap<_, _> = BTreeMap::new();

    for node in reachability_targets {
        let mut curr = node;
        while let Some(nexts) = next_edges.get(curr) {
            let next = if nexts.len() == 1 {
                nexts.iter().next().unwrap()
            } else {
                break;
            };
            if condition(next) {
                curr = step(next);
            } else {
                break;
            }
        }
        if curr != node {
            to_move.insert((*node).clone(), (*curr).clone());
        }
    }
    to_move
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        compact_reachability,
        compact_start,
        "a, b: ? x -> y;
        x, m: ;
        m, y: 1 == 1;",
        "a, b: ? m -> y;
        x, m: ;
        m, y: 1 == 1;"
    );

    test_transform!(
        compact_reachability,
        compact_end,
        "a, b: ? x -> y;
        x, n: 1 == 1;
        n, y: ;",
        "a, b: ? x -> n;
        x, n: 1 == 1;
        n, y: ;"
    );

    test_transform!(
        compact_reachability,
        compact_end_assign,
        "a, b: ? x -> y;
        x, n: 1 == 1;
        n, y: z = 1;",
        "a, b: ? x -> n;
        x, n: 1 == 1;
        n, y: z = 1;"
    );

    test_transform!(
        compact_reachability,
        compact_both,
        "a, b: ? x -> y;
        x, m: ;
        m, n: 1 == 1;
        n, y: ;",
        "a, b: ? m -> n;
        x, m: ;
        m, n: 1 == 1;
        n, y: ;"
    );

    test_transform!(
        compact_reachability,
        skip_chain_start,
        "a, b: ? x -> y;
        x, m1: ;
        m1, m2: ;
        m2, y: 1 == 1;",
        "a, b: ? m2 -> y;
        x, m1: ;
        m1, m2: ;
        m2, y: 1 == 1;"
    );

    test_transform!(
        compact_reachability,
        skip_chain_end,
        "a, b: ? x -> y;
        x, n1: 1 == 1;
        n1, n2: ;
        n2, y: ;",
        "a, b: ? x -> n1;
        x, n1: 1 == 1;
        n1, n2: ;
        n2, y: ;"
    );

    test_transform!(
        compact_reachability,
        fork_both,
        "a, b: ? x -> y;
        x, m1: ;
        x, m2: ;
        m1, y: ;
        m2, y: ;"
    );

    test_transform!(
        compact_reachability,
        negated_reachability,
        "a, b: ! x -> y;
        x, m: ;
        m, y: 1 == 1;",
        "a, b: ! m -> y;
        x, m: ;
        m, y: 1 == 1;"
    );
}
