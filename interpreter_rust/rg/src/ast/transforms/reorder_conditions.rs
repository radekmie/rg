use crate::ast::{Edge, Error, Expression, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;

impl Game<Id> {
    pub fn reorder_conditions(&mut self) -> Result<(), Error<Id>> {
        let mut last_node_idx = -1;
        while self.reorder_conditions_once(&mut last_node_idx) {}
        Ok(())
    }

    fn reorder_conditions_once(&mut self, last_node_idx: &mut i32) -> bool {
        let next_edges = self.next_edges();
        let prev_edges = self.prev_edges();
        let reachability_targets = self.reachability_targets();
        for (idx, (node, edges)) in next_edges.iter().enumerate() {
            if ((idx as i32) <= *last_node_idx)
                || next_edges.get(node).is_none_or(|edges| {
                    edges.iter().map(|e| &e.rhs).collect::<BTreeSet<_>>().len() != edges.len()
                        || edges
                            .iter()
                            .map(|e| &e.label)
                            .collect::<BTreeSet<_>>()
                            .len()
                            < 2
                })
            {
                continue;
            }

            let paths: Vec<_> = edges
                .iter()
                .filter(|e| !e.label.is_assignment())
                .map(|e| collect_path(&next_edges, &prev_edges, &reachability_targets, e))
                .collect();

            if paths.len() < 2 {
                continue;
            }

            // Count frequency of each label across all paths
            let mut label_frequencies: BTreeMap<&Label<Id>, usize> = BTreeMap::new();
            for path in &paths {
                for edge in path {
                    *label_frequencies.entry(&edge.label).or_insert(0) += 1;
                }
            }

            // If no edge has a label that appears more than once, we cannot reorder
            if label_frequencies.values().all(|&count| count < 2) {
                continue;
            }

            // Sort paths based on label frequencies and labels
            let new_paths: Vec<_> = paths
                .iter()
                .flat_map(|path| {
                    let nodes: Vec<_> = path.iter().map(|e| &e.rhs).collect();
                    let unsorted_labels = path.iter().map(|e| &e.label).collect();
                    let sorted_labels: Vec<_> = sort_labels(unsorted_labels, &label_frequencies);

                    let skip_label = &Label::new_skip();
                    let labels = sorted_labels.into_iter().chain(iter::repeat(skip_label));

                    let lhss = iter::once(node).chain(nodes.iter());
                    lhss.zip(nodes.iter())
                        .zip(labels)
                        .map(|((lhs, rhs), label)| Edge {
                            lhs: (**lhs).clone(),
                            rhs: (**rhs).clone(),
                            label: (*label).clone(),
                            span: Span::none(),
                        })
                        .map(Arc::new)
                        .collect::<Vec<_>>()
                })
                .collect();

            let to_remove = paths
                .iter()
                .flat_map(|path| path.iter())
                .collect::<BTreeSet<_>>();

            self.edges.retain(|edge| !to_remove.contains(edge));
            self.edges.extend(new_paths);
            *last_node_idx = idx as i32;
            return true;
        }

        false
    }
}

fn collect_path(
    next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    reachability_targets: &BTreeSet<&Node<Id>>,
    start_edge: &Arc<Edge<Id>>,
) -> Vec<Arc<Edge<Id>>> {
    let mut path = vec![start_edge.clone()];
    let mut current_edge = start_edge;

    // Check if there is exactly one incoming edge for every node
    while let Some(edges) = next_edges.get(&current_edge.rhs) {
        if singleton(prev_edges.get(&current_edge.rhs).unwrap()).is_none()
            || reachability_targets.contains(&current_edge.rhs)
        {
            break;
        }
        if let Some(next_edge) = singleton(edges).filter(|e| {
            !(e.label.is_assignment()
                // We don't want to loose duplicated tags
                || (e.label.is_tag() && path.iter().any(|p| p.label == e.label)))
        }) {
            path.push((*next_edge).clone());
            current_edge = *next_edge;
        } else {
            break;
        }
    }

    path
}

fn singleton<T>(set: &BTreeSet<T>) -> Option<&T> {
    if set.len() == 1 {
        set.iter().next()
    } else {
        None
    }
}

// Insertion sort for labels based on frequency and then lexicographically
fn sort_labels<'a>(
    labels: Vec<&'a Label<Id>>,
    label_frequencies: &BTreeMap<&Label<Id>, usize>,
) -> Vec<&'a Label<Id>> {
    let compare = |a: &&Label<Id>, b: &&Label<Id>| {
        label_frequencies
            .get(b)
            .unwrap_or(&0)
            .cmp(label_frequencies.get(a).unwrap_or(&0))
            .then_with(|| a.cmp(b))
    };

    let mut arr = labels;
    for i in 1..arr.len() {
        let mut j = i;
        let cur = arr[i];

        if label_frequencies.get(&cur).is_none_or(|c| *c < 2) {
            continue; // Skip labels that do not appear more than once
        }

        while j > 0 && compare(&cur, &arr[j - 1]).is_lt() && can_reorder(cur, arr[j - 1]) {
            arr[j] = arr[j - 1];
            j -= 1;
        }

        arr[j] = cur;
    }
    arr
}

// If latter_label e1 and first_label e2 are comparisons (assignments are not allowed here),
// we cannot reorder them if e1 uses any of the expressions in e2 for indexing.
// Reordering here could lead to incorrect indexing behavior.
fn can_reorder(latter_label: &Label<Id>, first_label: &Label<Id>) -> bool {
    if let (
        Label::Comparison {
            lhs: lhs_first,
            rhs: rhs_first,
            ..
        },
        Label::Comparison {
            lhs: lhs_latter,
            rhs: rhs_latter,
            ..
        },
    ) = (first_label, latter_label)
    {
        let expressions_in_first = [lhs_first.as_ref(), rhs_first.as_ref()];
        let used_for_indexing = uses_for_indexing(&expressions_in_first, lhs_latter)
            || uses_for_indexing(&expressions_in_first, rhs_latter);
        return !used_for_indexing;
    }
    true
}

fn uses_for_indexing(
    expressions_in_first: &[&Expression<Arc<str>>; 2],
    dangerous_expression: &Expression<Id>,
) -> bool {
    match dangerous_expression {
        Expression::Access { lhs, rhs, .. } => {
            expressions_in_first.contains(&rhs.as_ref())
                || uses_for_indexing(expressions_in_first, lhs)
                || uses_for_indexing(expressions_in_first, rhs)
        }
        Expression::Cast { rhs, .. } => uses_for_indexing(expressions_in_first, rhs),
        Expression::Reference { .. } => false,
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        reorder_conditions,
        basic,
        "begin, a: 1 == 1;
        begin, b: 2 == 2;
        begin, c: 3 == 3;
        a, a1: 4 == 4;
        b, b1: ;
        b1, b2: 4 == 4;
        c, c1: 2 == 2;",
        "begin, a: 4 == 4;
        a, a1: 1 == 1;
        begin, b: 2 == 2;
        b, b1: 4 == 4;
        b1, b2: ;
        begin, c: 2 == 2;
        c, c1: 3 == 3;"
    );

    test_transform!(
        reorder_conditions,
        used_in_indexing,
        "begin, a: 1 == x;
        a, a1: z[x] == 4;
        begin, b: x == 2;
        b, b1: z[x] == 4;"
    );

    test_transform!(
        reorder_conditions,
        subexpr_used_in_indexing,
        "begin, b: y[x] == 2;
        b, b1: z[x] == 4;
        begin, a: 1 == y[x];
        a, a1: z[x] == 4;",
        "begin, b: z[x] == 4;
        b, b1: y[x] == 2;
        begin, a: z[x] == 4;
        a, a1: 1 == y[x];"
    );

    test_transform!(
        reorder_conditions,
        partial,
        "begin, a: 1 == x;
        a, a1: 3 == 3;
        a1, a2: z[x] == 4;
        begin, b: x == 2;
        b, b1: 4 == 4;
        b1, b2: z[x] == 4;",
        "begin, a: 1 == x;
        a, a1: z[x] == 4;
        a1, a2: 3 == 3;
        begin, b: x == 2;
        b, b1: z[x] == 4;
        b1, b2: 4 == 4;"
    );
}
