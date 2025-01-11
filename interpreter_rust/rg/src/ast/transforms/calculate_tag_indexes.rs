use crate::ast::{Edge, Error, Game, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::position::Span;

impl Game<Arc<str>> {
    pub fn calculate_tag_indexes(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges = self.next_edges();
        let mut tag_indexes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        for Edge { label, rhs, .. } in self.edges.iter().map(Arc::as_ref) {
            if label.is_player_assignment() {
                let mut seen = BTreeSet::new();
                let mut queue = vec![(rhs, 0)];
                while let Some((lhs, index)) = queue.pop() {
                    let maybe_edges = next_edges.get(&lhs);
                    if seen.insert((lhs, index)) {
                        for edge in maybe_edges.into_iter().flatten() {
                            if edge.label.is_tag() {
                                queue.push((&edge.rhs, index + 1));
                                tag_indexes.entry(lhs.clone()).or_default().insert(index);
                            } else if !edge.label.is_player_assignment() {
                                queue.push((&edge.rhs, index));
                            }
                        }
                    }
                }
            }
        }

        self.pragmas.retain(|pragma| {
            !matches!(pragma, Pragma::TagIndex { .. } | Pragma::TagMaxIndex { .. })
        });

        let tag_indexes_by_index = tag_indexes.into_iter().fold(
            BTreeMap::new(),
            |mut groups: BTreeMap<_, Vec<_>>, (node, indexes)| {
                let maybe_index = match indexes.len() {
                    0 => None,
                    1 => indexes.first().copied().map(Ok),
                    _ => indexes.into_iter().max().map(Err),
                };

                if let Some(index) = maybe_index {
                    let nodes = groups.entry(index).or_default();
                    let index = nodes.partition_point(|x| *x < node);
                    nodes.insert(index, node.clone());
                }

                groups
            },
        );

        for (index, nodes) in tag_indexes_by_index {
            let pragma = match index {
                Ok(index) => Pragma::TagIndex {
                    span: Span::none(),
                    nodes,
                    index,
                },
                Err(index) => Pragma::TagMaxIndex {
                    span: Span::none(),
                    nodes,
                    index,
                },
            };

            let index = self.pragmas.partition_point(|x| *x < pragma);
            self.pragmas.insert(index, pragma);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_tag_indexes,
        complex_4,
        include_str!("../../../../../games/rg/simpleApplyTest4.rg"),
        adds "
            @tagIndex moveB tagA1 tagB1 : 0;
            @tagIndex tagB0 : 1;
            @tagMaxIndex doneB : 2;
        "
    );
}
