use std::{
    collections::{BTreeMap, BTreeSet},
    iter,
    sync::Arc,
};

use utils::position::Span;

use crate::ast::{Edge, Error, Game, Label, Node};

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
            for path in paths.iter() {
                for edge in path.iter() {
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
                    let mut sorted_labels: Vec<_> = path.iter().map(|e| &e.label).collect();
                    sorted_labels.sort_by(|a, b| {
                        label_frequencies
                            .get(b)
                            .unwrap_or(&0)
                            .cmp(label_frequencies.get(a).unwrap_or(&0))
                            .then_with(|| a.cmp(b))
                    });

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
                .flat_map(|path| path.iter().map(|e| e))
                .collect::<BTreeSet<_>>();

            self.edges.retain(|e| !to_remove.contains(e));
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
}
