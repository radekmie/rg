use crate::ast::{Edge, Error, Game, Pragma};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::position::Span;

const LIMIT_HARD: usize = 150;
const LIMIT_SOFT: usize = 100;

impl Game<Arc<str>> {
    pub fn calculate_tag_indexes(&mut self) -> Result<(), Error<Arc<str>>> {
        let next_edges = self.next_edges();
        let mut tag_indexes: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        for Edge { label, rhs, .. } in self.edges.iter().map(Arc::as_ref) {
            if label.is_player_assignment() {
                let mut seen = BTreeSet::new();
                let mut queue = vec![(rhs, 0)];
                while let Some((lhs, index)) = queue.pop() {
                    // TODO: Handle cycles with tags.
                    if index == LIMIT_HARD {
                        tag_indexes.remove(lhs);
                        break;
                    }

                    let maybe_edges = next_edges.get(&lhs);
                    if seen.insert((lhs, index)) {
                        for edge in maybe_edges.into_iter().flatten() {
                            if edge.label.is_tag() || edge.label.is_tag_variable() {
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
            // TODO: Handle cycles with tags.
            if *index.as_ref().unwrap_or_else(|index| index) >= LIMIT_SOFT {
                continue;
            }

            let span = Span::none();
            self.add_pragma(match index {
                Ok(index) => Pragma::TagIndex { span, nodes, index },
                Err(index) => Pragma::TagMaxIndex { span, nodes, index },
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        calculate_tag_indexes,
        cycle_immediate,
        "begin, x: player = keeper; x, x: $ tag; x, end:;"
    );

    test_transform!(
        calculate_tag_indexes,
        cycle_delayed,
        "begin, x: player = keeper; x, y:; y, y: $ tag; y, end:;"
    );

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

    test_transform!(
        calculate_tag_indexes,
        tag_variable,
        "begin, x: player = keeper; x, y: $$ v; y, end:;",
        adds "@tagIndex x : 0;"
    );
}
