use crate::ast::{Error, Game, Label};
use std::sync::Arc;

impl<Id: Clone + Ord> Game<Id> {
    pub fn serialize_conditions(&mut self) -> Result<(), Error<Id>> {
        'outer: loop {
            let next_edges = self.next_edges();
            'inner: for edges in next_edges.values() {
                // At least two pairs of edges are needed. However, the positive
                // ones will be chained, not parallel.
                if edges.len() < 3 {
                    continue;
                }

                // All edges must be conditions.
                let mut negatives = vec![];
                let mut positives = vec![];
                for edge in edges {
                    match &edge.label {
                        Label::Comparison { negated, .. } | Label::Reachability { negated, .. } => {
                            if *negated {
                                negatives.push(*edge);
                            } else {
                                positives.push(*edge);
                            }
                        }
                        _ => {
                            continue 'inner;
                        }
                    }
                }

                // Swap the sides if the condition is inverted.
                if positives.len() > 1 {
                    (negatives, positives) = (positives, negatives);
                }

                // All negatives must target the same node.
                if negatives.iter().any(|edge| edge.rhs != negatives[0].rhs) {
                    continue;
                }

                // Verify whether the positive conditions are chained, i.e.,
                // have exactly one outgoing edge.
                let Some(mut chain_tip) = positives.pop() else {
                    continue;
                };

                if !positives.is_empty() {
                    continue;
                }

                let mut changes = vec![];
                while !negatives.is_empty() {
                    let Some(negative_index) = negatives
                        .iter()
                        .position(|negative| negative.label.is_negated(&chain_tip.label))
                    else {
                        continue 'inner;
                    };

                    let negative = negatives.swap_remove(negative_index).clone();
                    changes.push((negative, chain_tip.lhs.clone()));

                    // It was the last condition.
                    if negatives.is_empty() {
                        break;
                    }

                    let Some(next_chain_tip) = next_edges
                        .get(&chain_tip.rhs)
                        .filter(|edges| edges.len() == 1)
                        .and_then(|edges| edges.first())
                    else {
                        continue 'inner;
                    };

                    chain_tip = next_chain_tip;
                }

                // Delay the evaluation of the negative condition to when the
                // positive condition is evaluated.
                for (negative, positive_lhs) in changes {
                    if let Some(negative) = self.edges.iter_mut().find(|edge| **edge == negative) {
                        Arc::make_mut(negative).lhs = positive_lhs;
                    }
                }

                continue 'outer;
            }

            break;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        serialize_conditions,
        mixed_labels,
        "a, b: ! p1 -> q1;
        a, b: ! p2 -> q2;
        a, b: $ x;
        a, c: ? p1 -> q1;
        c, d: ? p2 -> q2;"
    );

    test_transform!(
        serialize_conditions,
        mixed_targets,
        "a, b: ! p1 -> q1;
        a, x: ! p2 -> q2;
        a, c: ? p1 -> q1;
        c, d: ? p2 -> q2;"
    );

    test_transform!(
        serialize_conditions,
        chain_discontinued,
        "a, b: ! p1 -> q1;
        a, x: ! p2 -> q2;
        a, c: ? p1 -> q1;
        c, d: $ x;"
    );

    test_transform!(
        serialize_conditions,
        basic_2,
        "a, b: ! p1 -> q1;
        a, b: ! p2 -> q2;
        a, c: ? p1 -> q1;
        c, d: ? p2 -> q2;",
        "a, b: ! p1 -> q1;
        c, b: ! p2 -> q2;
        a, c: ? p1 -> q1;
        c, d: ? p2 -> q2;"
    );

    test_transform!(
        serialize_conditions,
        basic_3,
        "a, b: ! p1 -> q1;
        a, b: ! p2 -> q2;
        a, b: ! p3 -> q3;
        a, c: ? p1 -> q1;
        c, d: ? p2 -> q2;
        d, e: ? p3 -> q3;",
        "a, b: ! p1 -> q1;
        c, b: ! p2 -> q2;
        d, b: ! p3 -> q3;
        a, c: ? p1 -> q1;
        c, d: ? p2 -> q2;
        d, e: ? p3 -> q3;"
    );

    test_transform!(
        serialize_conditions,
        inverted_2,
        "a, b: ? p1 -> q1;
        a, b: ? p2 -> q2;
        a, c: ! p1 -> q1;
        c, d: ! p2 -> q2;",
        "a, b: ? p1 -> q1;
        c, b: ? p2 -> q2;
        a, c: ! p1 -> q1;
        c, d: ! p2 -> q2;"
    );

    test_transform!(
        serialize_conditions,
        inverted_3,
        "a, b: ? p1 -> q1;
        a, b: ? p2 -> q2;
        a, b: ? p3 -> q3;
        a, c: ! p1 -> q1;
        c, d: ! p2 -> q2;
        d, e: ! p3 -> q3;",
        "a, b: ? p1 -> q1;
        c, b: ? p2 -> q2;
        d, b: ? p3 -> q3;
        a, c: ! p1 -> q1;
        c, d: ! p2 -> q2;
        d, e: ! p3 -> q3;"
    );
}
