use crate::ast::{Edge, Error, Game, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type EdgeAndPath<Id> = (Arc<Edge<Id>>, Vec<Arc<Edge<Id>>>);

impl Game<Id> {
    pub fn skip_redundant_tags(&mut self) -> Result<(), Error<Id>> {
        // If the game uses the `visible`, then we leave the tags as they are,
        // just in case. In the future, we could make the analysis smarter.
        let visible = Id::from("visible");
        if self
            .edges
            .iter()
            .all(|edge| !edge.label.has_variable(&visible))
        {
            // `find_redundant_tag` walks the automaton in both directions. As
            // the `skip` doesn't affect the paths, we can cache these maps.
            let clone = Self {
                edges: self.edges.clone(),
                ..Self::default()
            };
            let next_edges = clone.next_edges();
            let prev_edges = clone.prev_edges();

            while let Some(index) = self.find_redundant_tag(&next_edges, &prev_edges) {
                Arc::make_mut(&mut self.edges[index]).skip();
            }
        }

        Ok(())
    }

    /// A tag is _redundant_, if it...
    ///   1. ...has exactly one predecessor and is its only successor.
    ///   2. ...has exactly one successor and is its only predecessor.
    ///   3. ...is either a plain `Tag` or there's no modification of the
    ///      variable used in `TagVariable` on the path from the predecessor nor
    ///      to the successor.
    ///
    /// (A `player` assignment is also a valid predecessor/successor.)
    fn find_redundant_tag(
        &self,
        next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
        prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    ) -> Option<usize> {
        fn check(edge: &Arc<Edge<Id>>, edge_and_path: Option<EdgeAndPath<Id>>) -> bool {
            edge_and_path.is_some_and(|edge_and_path| {
                *edge == edge_and_path.0
                    && edge.label.as_tag_variable().is_none_or(|identifier| {
                        !edge_and_path
                            .1
                            .into_iter()
                            .any(|edge| edge.is_assignment_to(identifier))
                    })
            })
        }

        for (index, edge) in self.edges.iter().enumerate() {
            if (edge.label.is_tag() || edge.label.is_tag_variable())
                && self
                    .find_next(&edge.rhs, next_edges)
                    .is_none_or(|(next, _)| check(edge, self.find_prev(&next.lhs, prev_edges)))
                && self
                    .find_prev(&edge.lhs, prev_edges)
                    .is_none_or(|(prev, _)| check(edge, self.find_next(&prev.rhs, next_edges)))
            {
                return Some(index);
            }
        }

        None
    }

    fn find_prev(
        &self,
        node: &Node<Id>,
        prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    ) -> Option<EdgeAndPath<Id>> {
        let mut seen = BTreeSet::new();
        let mut queue = vec![(node, vec![])];
        let mut prev: Option<EdgeAndPath<Id>> = None;
        while let Some((rhs, path)) = queue.pop() {
            if let Some(edges) = prev_edges.get(&rhs) {
                for edge in edges {
                    if edge.label.is_player_assignment()
                        || edge.label.is_tag()
                        || edge.label.is_tag_variable()
                    {
                        if prev.is_none() {
                            let _ = prev.insert(((*edge).clone(), path.clone()));
                            continue;
                        }

                        return None;
                    }

                    if seen.insert(&edge.lhs) {
                        let mut path = path.clone();
                        path.push((*edge).clone());
                        queue.push((&edge.lhs, path));
                    }
                }
            }
        }

        prev
    }

    fn find_next(
        &self,
        node: &Node<Id>,
        next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    ) -> Option<EdgeAndPath<Id>> {
        let mut seen = BTreeSet::new();
        let mut queue = vec![(node, vec![])];
        let mut next: Option<EdgeAndPath<Id>> = None;
        while let Some((lhs, path)) = queue.pop() {
            if let Some(edges) = next_edges.get(&lhs) {
                for edge in edges {
                    if edge.label.is_player_assignment()
                        || edge.label.is_tag()
                        || edge.label.is_tag_variable()
                    {
                        if next.is_none() {
                            let _ = next.insert(((*edge).clone(), path.clone()));
                            continue;
                        }

                        return None;
                    }

                    if seen.insert(&edge.rhs) {
                        let mut path = path.clone();
                        path.push((*edge).clone());
                        queue.push((&edge.rhs, path));
                    }
                }
            }
        }

        next
    }
}

impl Edge<Id> {
    fn is_assignment_to(&self, identifier: &Id) -> bool {
        matches!(self.label.as_var_assignment(), Some(x) if x == identifier)
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        skip_redundant_tags,
        simple_one,
        "begin, q1: player = keeper; q1, q2: $a; q2, end: player = keeper;",
        "begin, q1: player = keeper; q1, q2: ; q2, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        simple_two,
        "begin, q1: player = keeper; q1, q2: $a; q2, q3: $b; q3, end: player = keeper;",
        "begin, q1: player = keeper; q1, q2: ; q2, q3: ; q3, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        split_equal,
        "begin, q1: player = keeper; q1, q2: $a; q2, end: player = keeper; q1, q3: $a; q3, end: player = keeper;"
    );

    test_transform!(
        skip_redundant_tags,
        split_different,
        "begin, q1: player = keeper; q1, q2: $a; q2, end: player = keeper; q1, q3: $b; q3, end: player = keeper;"
    );
}
