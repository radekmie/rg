use super::{gen_fresh_node, max_node_id};
use crate::ast::{Edge, Error, Expression, Game, Label, Node, Type};
use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::Arc;

type Id = Arc<str>;
type ToCompact = (Arc<Expression<Id>>, Arc<Type<Id>>, Vec<Id>);

impl Game<Id> {
    pub fn compact_comparisons(&mut self) -> Result<(), Error<Id>> {
        let mut to_compat = vec![];
        let next_edges = self.next_edges();
        for node in self.nodes() {
            let Some(((expr, type_, unused_members), edge)) =
                next_edges.get(node).and_then(|edges| {
                    edges.iter().find_map(|edge| {
                        self.try_compact_edge(edge, &next_edges)
                            .zip(Some((**edge).clone()))
                    })
                })
            else {
                continue;
            };
            to_compat.push((edge.clone(), expr, type_, unused_members));
        }

        if to_compat.is_empty() {
            return Ok(());
        }

        self.edges.retain(|edge| {
            !to_compat.iter().any(|(old_edge, expr, _, _)| {
                edge.lhs == old_edge.lhs && edge.rhs == old_edge.rhs && {
                    let Label::Comparison { lhs, rhs, .. } = &edge.label else {
                        return false;
                    };
                    lhs == expr || rhs == expr
                }
            })
        });

        let mut max_id = max_node_id(&self.nodes());

        for (edge, expr, type_, unused_members) in to_compat {
            let nodes: Vec<_> = unused_members
                .iter()
                .map(|_| gen_fresh_node(&mut max_id))
                .collect();
            let lhss = iter::once(edge.lhs.clone()).chain(nodes.clone());
            let rhss = nodes.into_iter().chain(iter::once(edge.rhs.clone()));
            let labels = unused_members
                .iter()
                .map(|id| Label::Comparison {
                    lhs: expr.clone(),
                    rhs: Arc::new(Expression::new_cast(
                        type_.clone(),
                        Arc::new(Expression::new(id.clone())),
                    )),
                    negated: true,
                })
                .chain(iter::once(Label::new_skip()));

            let pairs = lhss.zip(rhss).zip(labels);
            let xs = pairs
                .into_iter()
                .map(|((lhs, rhs), label)| Edge::new(lhs, rhs, label))
                .map(Arc::from);
            self.edges.extend(xs);
        }

        Ok(())
    }

    fn get_type_members<'a>(&'a self, type_: &'a Type<Id>) -> Option<&'a Vec<Id>> {
        match type_ {
            Type::Set { identifiers, .. } => Some(identifiers),
            Type::TypeReference { identifier } => self
                .resolve_typedef(identifier)
                .and_then(|type_| self.get_type_members(&type_.type_)),
            Type::Arrow { .. } => None,
        }
    }

    fn try_compact_edge(
        &self,
        edge: &Edge<Id>,
        next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    ) -> Option<ToCompact> {
        let (expr, ids) = self.lhs_or_rhs(edge, next_edges)?;
        let type_ = expr.infer(self).ok()?;
        let type_members = self.get_type_members(&type_)?;
        if ids.iter().any(|id| !type_members.contains(id))
            || type_members.iter().filter(|id| !ids.contains(id)).count() >= ids.len()
        {
            None
        } else {
            let unused_members = type_members
                .iter()
                .filter(|id| !ids.contains(id))
                .cloned()
                .collect();
            Some((expr.clone(), type_.clone(), unused_members))
        }
    }

    fn lhs_or_rhs<'a>(
        &'a self,
        edge: &'a Edge<Id>,
        next_edges: &'a BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    ) -> Option<(&'a Arc<Expression<Id>>, Vec<&'a Id>)> {
        let Label::Comparison { lhs, rhs, negated } = &edge.label else {
            return None;
        };
        if *negated {
            return None;
        }
        let outgoing_edges = next_edges.get(&edge.lhs)?;
        get_same_comparisons(lhs, edge, outgoing_edges)
            .map(|ids| (lhs, ids))
            .or_else(|| get_same_comparisons(rhs, edge, outgoing_edges).map(|ids| (rhs, ids)))
    }
}

fn get_same_comparisons<'a>(
    expr: &'a Arc<Expression<Id>>,
    same_as: &'a Edge<Id>,
    edges: &'a BTreeSet<&Arc<Edge<Id>>>,
) -> Option<Vec<&'a Id>> {
    let same_comparisons: Vec<_> = edges
        .iter()
        .filter(|edge| {
            let Label::Comparison { lhs, rhs, .. } = &edge.label else {
                return false;
            };
            lhs == expr || rhs == expr
        })
        .collect();
    if same_comparisons.len() < 2 || same_comparisons.iter().any(|edge| edge.rhs != same_as.rhs) {
        None
    } else {
        same_comparisons
            .into_iter()
            .map(|edge| {
                let Label::Comparison { lhs, rhs, negated } = &edge.label else {
                    return None;
                };
                if *negated {
                    None
                } else if lhs == expr {
                    rhs.uncast().as_reference()
                } else {
                    lhs.uncast().as_reference()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        compact_comparisons,
        small,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x == 2;",
        "type A = {1,2,3};
        var x: A = 1;
        begin, 1: x != A(3);
        1, end: ;"
    );

    test_transform!(
        compact_comparisons,
        with_cast,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x == A(2);",
        "type A = {1,2,3};
        var x: A = 1;
        begin, 1: x != A(3);
        1, end: ;"
    );

    test_transform!(
        compact_comparisons,
        no_compact,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x != A(3);"
    );

    test_transform!(
        compact_comparisons,
        skip_compat,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x == 2;
        begin, end: x == 3;",
        "type A = { 1, 2, 3 };
        var x: A = 1;
        begin, end: ;"
    );

    test_transform!(
        compact_comparisons,
        no_compact_skip,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x != 2;
        begin, end: x == 3;",
        "type A = { 1, 2, 3 };
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x != 2;
        begin, end: x == 3;"
    );

    test_transform!(
        compact_comparisons,
        not_member,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x == abc;"
    );

    test_transform!(
        compact_comparisons,
        not_identifier,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x == a[1];
        begin, end: x == 3;"
    );

    test_transform!(
        compact_comparisons,
        different_sides,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: 2 == x;",
        "type A = { 1, 2, 3 };
        var x: A = 1;
        begin, 1: x != A(3);
        1, end: ;"
    );

    test_transform!(
        compact_comparisons,
        multiple_expressions,
        "type A = {1,2,3};
        var x: A = 1;
        var y: A = 2;
        begin, end: x == 1;
        begin, end: x == 2;
        begin, end: y == 2;",
        "type A = { 1, 2, 3 };
        var x: A = 1;
        var y: A = 2;
        begin, end: y == 2;
        begin, 1: x != A(3);
        1, end: ;"
    );

    test_transform!(
        compact_comparisons,
        chain,
        "type A = {1,2,3,4, 5};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x == 2;
        begin, end: x == 3;",
        "type A = { 1, 2, 3, 4, 5 };
        var x: A = 1;
        begin, 1: x != A(4);
        1, 2: x != A(5);
        2, end: ;"
    );
}
