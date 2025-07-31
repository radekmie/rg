use crate::ast::{Edge, Error, Game, Label, Node, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl<Id: Clone + Ord> Game<Id> {
    pub fn join_exclusive_edges(&mut self) -> Result<(), Error<Id>> {
        let next_edges = self.next_edges();
        let prev_edges = self.prev_edges();
        let mut to_add = BTreeSet::new();
        let mut to_remove = BTreeSet::new();
        let artificial_tags = self.pragmas.iter().filter_map(|p| match p {
            Pragma::ArtificialTag { tags, .. } => Some(tags),
            _ => None,
        });
        let artificial_tags: BTreeSet<&Id> = artificial_tags.flatten().collect();
        let is_artificial_tag =
            |label: &Label<Id>| label.is_tag_and(|t| artificial_tags.contains(t));

        for edges in next_edges.values() {
            let edge = edges.iter().next().unwrap();
            if edge.is_conditional() {
                if let Some(e2) = edges.iter().find(|e| e.label.is_negated(&edge.label)) {
                    let Some(path1) = build_path(&next_edges, &prev_edges, edge, None) else {
                        continue;
                    };
                    let target = &path1.last().unwrap().rhs;
                    let Some(path2) = build_path(&next_edges, &prev_edges, e2, Some(target)) else {
                        continue;
                    };
                    let path2_slice: Vec<_> = path2.iter().collect();
                    if paths_match(&path1, &path2_slice, &is_artificial_tag) {
                        to_add.insert(Arc::from(Edge::new_skip(edge.lhs.clone(), target.clone())));
                        to_remove.extend(path1);
                        to_remove.extend(path2);
                    }
                }
            }
        }

        let empty_set: BTreeSet<_> = BTreeSet::new();
        for node in self.nodes() {
            let Some((complex_start, simple_path)) = split_edges(
                next_edges.get(&node).unwrap_or(&empty_set),
                &artificial_tags,
            ) else {
                continue;
            };
            let target = &simple_path.first().unwrap().rhs;
            let Some(complex_path) =
                build_path(&next_edges, &prev_edges, complex_start, Some(target))
            else {
                continue;
            };
            if paths_match(&complex_path, &simple_path, &is_artificial_tag) {
                to_remove.extend(complex_path);
                to_remove.extend(simple_path.into_iter().cloned());
                to_add.insert(Arc::from(Edge::new_skip(node.clone(), target.clone())));
            }
        }

        self.edges.extend(to_add);
        self.edges.retain(|edge| !to_remove.contains(edge));

        Ok(())
    }
}

fn build_path<Id: Clone + Ord>(
    next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    start_edge: &Arc<Edge<Id>>,
    target: Option<&Node<Id>>,
) -> Option<Vec<Arc<Edge<Id>>>> {
    let mut complex_path = vec![start_edge.clone()];
    let mut current_edge = start_edge;

    while let Some(next_edge) = next_edges.get(&current_edge.rhs).and_then(singleton) {
        let prev_edges = prev_edges.get(&next_edge.lhs)?;

        // Check if there is exactly one incoming edge for every node other than the first and last
        if singleton(prev_edges).is_none() {
            match target {
                Some(target) if target != &next_edge.lhs => return None,
                _ => return Some(complex_path),
            }
        }
        complex_path.push((*next_edge).clone());
        current_edge = *next_edge;
    }

    if target.is_none() || complex_path.last().map(|edge| &edge.rhs) == target {
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

fn paths_match<Id: Ord>(
    complex_path: &[Arc<Edge<Id>>],
    simple_path: &[&Arc<Edge<Id>>],
    is_artificial_tag: &dyn Fn(&Label<Id>) -> bool,
) -> bool {
    // For each condition on simple path "p" complex path contains "!p"
    // For each condition on complex path "p" simple path contains "!p"
    simple_path.iter().all(|simple| {
        is_artificial_tag(&simple.label)
            || simple.label.is_skip()
            || complex_path
                .iter()
                .any(|complex| complex.label.is_negated(&simple.label))
    }) && complex_path.iter().all(|complex| {
        is_artificial_tag(&complex.label)
            || complex.label.is_skip()
            || simple_path
                .iter()
                .any(|simple| simple.label.is_negated(&complex.label))
    })
}

#[expect(clippy::type_complexity)]
fn split_edges<'a, Id: Ord + Clone>(
    edges: &'a BTreeSet<&Arc<Edge<Id>>>,
    artificial_tags: &BTreeSet<&Id>,
) -> Option<(&'a Arc<Edge<Id>>, Vec<&'a Arc<Edge<Id>>>)> {
    if edges.len() < 3
        || !edges
            .iter()
            .all(|e| e.is_conditional() || e.label.is_tag_and(|t| artificial_tags.contains(t)))
    {
        return None;
    }
    let first_rhs = &edges.first()?.rhs;
    let (first, second): (Vec<_>, Vec<_>) = edges.iter().partition(|edge| edge.rhs == *first_rhs);
    if first.len() == 1 {
        Some((first[0], second))
    } else if second.len() == 1 {
        Some((second[0], first))
    } else {
        None
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
        "begin, end: ? a -> c;
        begin, end: ;"
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

    test_transform!(
        join_exclusive_edges,
        complex_aritficial_tags1,
        "@artificialTag foo bar;
        begin, end: 1 == 1;
        begin, end: 1 == 2;
        begin, end: 1 == 3;
        begin, b1: 1 != 1;
        b1, b2: 1 != 2;
        b2, b3: $ foo;
        b3, b4: $ bar;
        b4, end: 1 != 3;
        end, end1: 1 != 3;",
        "@artificialTag foo bar;
        end, end1: 1 != 3;
        begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        complex_aritficial_tags2,
        "@artificialTag foo bar;
        begin, end: 1 == 1;
        begin, end: 1 == 2;
        begin, end: 1 == 3;
        begin, b1: $ foo;
        b1, b2: 1 != 1;
        b2, b3: 1 != 2;
        b3, b4: $ bar;
        b4, end: 1 != 3;",
        "@artificialTag foo bar;
        begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        complex_aritficial_tags3,
        "@artificialTag foo bar;
        begin, end: 1 == 1;
        begin, end: $ foo;
        begin, end: 1 == 2;
        begin, end: 1 == 3;
        begin, b1: 1 != 1;
        b1, b2: 1 != 2;
        b2, end: 1 != 3;",
        "@artificialTag foo bar;
        begin, end: ;"
    );

    test_transform!(
        join_exclusive_edges,
        pentago,
        "@artificialTag t1 t2;
        2481, 2482: ? 999 -> 1000;
        2482, 2483: $t1 ;
        2483, 2187: $t2 ;
        2481, 2187: ! 999 -> 1000;",
        "@artificialTag t1 t2;
        2481, 2187: ;"
    );
}
