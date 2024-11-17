use crate::ast::{Edge, Error, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl<Id: Clone + Ord> Game<Id> {
    pub fn join_exclusive_edges(&mut self) -> Result<(), Error<Id>> {
        let mut to_ignore = BTreeSet::new();
        let mut to_remove = BTreeSet::new();

        for (i, x) in self.edges.iter().enumerate() {
            if !to_remove.contains(&i) && x.is_conditional() {
                for (j, y) in self.edges.iter().enumerate() {
                    if x.lhs == y.lhs && x.rhs == y.rhs && x.label.is_negated(&y.label) {
                        to_ignore.insert(i);
                        to_remove.insert(j);
                    }
                }
            }
        }

        for index in to_ignore {
            Arc::make_mut(&mut self.edges[index]).skip();
        }

        for index in to_remove.into_iter().rev() {
            self.edges.remove(index);
        }

        self.join_complex();

        Ok(())
    }

    fn join_complex(&mut self) {
        let next_edges = self.next_edges();
        let prev_edges = self.prev_edges();
        let empty_set: BTreeSet<_> = BTreeSet::new();
        let mut to_remove = vec![];
        let mut to_add = vec![];
        for node in self.nodes() {
            let Some((complex_start, simple_path)) =
                split_edges(next_edges.get(&node).unwrap_or(&empty_set))
            else {
                continue;
            };
            let target = &simple_path.first().unwrap().rhs;
            let Some(complex_path) =
                build_complex_path(&next_edges, &prev_edges, complex_start, target)
            else {
                continue;
            };

            if paths_match(&complex_path, &simple_path) {
                to_remove.extend(complex_path);
                to_remove.extend(simple_path.into_iter().cloned());
                to_add.push(Arc::from(Edge::new_skip(node.clone(), target.clone())));
            }
        }

        self.edges.retain(|edge| !to_remove.contains(edge));
        self.edges.extend(to_add);
    }
}

fn build_complex_path<Id: Clone + Ord>(
    next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    start_edge: &Arc<Edge<Id>>,
    target: &Node<Id>,
) -> Option<Vec<Arc<Edge<Id>>>> {
    let mut complex_path = vec![start_edge.clone()];
    let mut current_edge = start_edge;

    while let Some(next_edge) = next_edges.get(&current_edge.rhs).and_then(singleton) {
        complex_path.push((*next_edge).clone());
        current_edge = *next_edge;
        // Check if there is exactly one incoming edge for every node other than the first and last
        singleton(prev_edges.get(&current_edge.lhs)?)?;
    }

    if complex_path.last().map(|edge| &edge.rhs) == Some(target) {
        Some(complex_path)
    } else {
        None
    }
}

fn singleton<T>(set: &BTreeSet<T>) -> Option<&T> {
    if set.len() == 1 {
        set.iter().next()
    } else {
        None
    }
}

fn paths_match<Id: PartialEq>(
    complex_path: &[Arc<Edge<Id>>],
    simple_path: &[&Arc<Edge<Id>>],
) -> bool {
    // For each condition on simple path "p" complex path contains "!p"
    // For each condition on complex path "p" simple path contains "!p"
    simple_path.iter().all(|simple| {
        complex_path
            .iter()
            .any(|complex| complex.label.is_negated(&simple.label))
    }) && complex_path.iter().all(|complex| {
        simple_path
            .iter()
            .any(|simple| simple.label.is_negated(&complex.label))
    })
}

#[expect(clippy::type_complexity)]
fn split_edges<'a, Id: Ord + Clone>(
    edges: &'a BTreeSet<&Arc<Edge<Id>>>,
) -> Option<(&'a Arc<Edge<Id>>, Vec<&'a Arc<Edge<Id>>>)> {
    if edges.len() < 3 || !edges.iter().all(|e| e.is_conditional()) {
        return None;
    }
    let first_rhs = &edges.first()?.rhs;
    let (first, second): (Vec<_>, Vec<_>) = edges.iter().partition(|edge| edge.rhs == *first_rhs);
    if first.len() == 1 {
        Some((first[0], second))
    } else {
        Some((second[0], first))
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        join_exclusive_edges,
        reachability1,
        "begin, end: ? a -> b;
        begin, end: ! a -> b;",
        "begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        reachability2,
        "begin, end: ? a -> b;
        begin, a: ! a -> b;"
    );

    test_transform!(
        join_exclusive_edges,
        reachability3,
        "begin, end: ? a -> b;
        begin, end: ! a -> b;
        begin, end: ? a -> c;",
        "begin, end: ;
        begin, end: ? a -> c;"
    );

    test_transform!(
        join_exclusive_edges,
        comparison1,
        "begin, end: a == b;
        begin, end: a != b;",
        "begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        comparison2,
        "begin, end: a == b;
        begin, end: a == b;"
    );

    test_transform!(
        join_exclusive_edges,
        complex1,
        "begin, end: 1 == 1;
        begin, end: 1 == 2;
        begin, end: 1 == 3;
        begin, b1: 1 != 1;
        c1, b2: ;
        b1, b2: 1 != 2;
        b2, end: 1 != 3;"
    );

    test_transform!(
        join_exclusive_edges,
        complex2,
        "begin, end: 1 == 1;
        begin, end: 1 == 2;        
        begin, b1: 1 != 1;
        b1, b2: 1 != 2;
        b2, end: 1 != 3;"
    );

    test_transform!(
        join_exclusive_edges,
        complex3,
        "begin, end: 1 == 1;
        begin, end: 1 == 2;
        begin, end: 1 == 3;
        begin, b1: 1 != 1;
        b1, end: 1 != 3;"
    );

    test_transform!(
        join_exclusive_edges,
        complex4,
        "begin, end: 1 == 1;
        begin, end: 1 == 2;
        begin, end: 1 == 3;
        begin, b1: 1 != 1;
        b1, b2: 1 != 2;
        b2, end: 1 != 3;",
        "begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        complex5,
        "begin, end: 1 != 1;
        begin, end: 1 == 2;
        begin, end: 1 != 3;
        begin, b1: 1 == 1;
        b1, b2: 1 != 2;
        b2, end: 1 == 3;",
        "begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        complex6,
        "begin, end: 1 == 1;
        begin, end: 1 == 2;
        begin, end: 1 == 3;
        begin, b1: 1 == 1;
        b1, b2: 1 != 2;
        b2, end: 1 == 3;"
    );

    test_transform!(
        join_exclusive_edges,
        complex7,
        "begin, b1: 1 == 1;
        b1, b2: 1 != 2;
        b2, end: 1 == 3;begin, end: 1 != 1;
        begin, end: 1 == 2;
        begin, end: 1 != 3;",
        "begin, end: ;"
    );
}
