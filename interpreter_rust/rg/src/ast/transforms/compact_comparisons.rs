use super::gen_fresh_node;
use crate::ast::{Edge, Error, Expression, Game, Label, Type};
use std::collections::BTreeSet;
use std::iter;
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn compact_comparisons(&mut self) -> Result<(), Error<Id>> {
        let mut to_compat = vec![];
        for node in self.nodes() {
            let Some(((expr, unused_members), edge)) = self
                .outgoing_edges(node)
                .find_map(|edge| self.try_compact_edge(edge).zip(Some(edge)))
            else {
                continue;
            };
            to_compat.push((edge.clone(), expr, unused_members));
        }

        if to_compat.is_empty() {
            return Ok(());
        }

        self.edges.retain(|edge| {
            !to_compat.iter().any(|(old_edge, expr, _)| {
                edge.lhs == old_edge.lhs && edge.rhs == old_edge.rhs && {
                    let Label::Comparison { lhs, rhs, .. } = &edge.label else {
                        return false;
                    };
                    lhs == expr || rhs == expr
                }
            })
        });

        let nodes: BTreeSet<_> = self.nodes().into_iter().cloned().collect();

        for (edge, expr, unused_members) in to_compat {
            let nodes = unused_members.iter().map(|id| {
                let mut node =
                    gen_fresh_node(format!("{}_{expr}_{id}", edge.lhs.literal()), &nodes);
                let mut bindings: Vec<_> = edge.lhs.parts.iter().skip(1).cloned().collect();
                node.parts.append(&mut bindings);
                node
            });
            let lhss = iter::once(edge.lhs.clone()).chain(nodes.clone());
            let rhss = nodes.chain(iter::once(edge.rhs.clone()));
            let labels = unused_members
                .iter()
                .map(|id| Label::Comparison {
                    lhs: expr.clone(),
                    rhs: Arc::new(Expression::new(id.clone())),
                    negated: true,
                })
                .chain(iter::once(Label::new_skip()));

            let pairs = lhss.zip(rhss).zip(labels);
            let xs = pairs
                .into_iter()
                .map(|((lhs, rhs), label)| Edge::new(lhs, rhs, label));
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

    fn try_compact_edge(&self, edge: &Edge<Id>) -> Option<(Arc<Expression<Id>>, Vec<Id>)> {
        let (expr, ids) = self.lhs_or_rhs(edge)?;
        let type_ = expr.infer(self, Some(edge)).ok()?;
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
            Some((expr.clone(), unused_members))
        }
    }

    fn lhs_or_rhs<'a>(
        &'a self,
        edge: &'a Edge<Id>,
    ) -> Option<(&'a Arc<Expression<Id>>, Vec<&'a Id>)> {
        let Label::Comparison { lhs, rhs, negated } = &edge.label else {
            return None;
        };
        if *negated {
            return None;
        }
        get_same_comparisons(lhs, edge, self.outgoing_edges(&edge.lhs))
            .map(|ids| (lhs, ids))
            .or_else(|| {
                get_same_comparisons(rhs, edge, self.outgoing_edges(&edge.lhs))
                    .map(|ids| (rhs, ids))
            })
    }
}

fn get_same_comparisons<'a>(
    expr: &'a Arc<Expression<Id>>,
    same_as: &'a Edge<Id>,
    edges: impl Iterator<Item = &'a Edge<Id>>,
) -> Option<Vec<&'a Id>> {
    let same_comparisons: Vec<_> = edges
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
                    rhs.as_reference()
                } else {
                    lhs.as_reference()
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
        begin, __gen_0_begin_x_3: x != 3;
        __gen_0_begin_x_3, end: ;"
    );

    test_transform!(
        compact_comparisons,
        no_compact,
        "type A = {1,2,3};
        var x: A = 1;
        begin, end: x == 1;
        begin, end: x != 3;"
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
        begin, __gen_0_begin_x_3: x != 3;
        __gen_0_begin_x_3, end: ;"
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
        begin, __gen_1_begin_x_3: x != 3;
        __gen_1_begin_x_3, end: ;"
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
        begin, __gen_0_begin_x_4: x != 4;
        __gen_0_begin_x_4, __gen_0_begin_x_5: x != 5;
        __gen_0_begin_x_5, end: ;"
    );

    test_transform!(
        compact_comparisons,
        chain_binding,
        "type A = {1,2,3,4, 5};
        begin(x: A), end: x == 1;
        begin(x: A), end: x == 2;
        begin(x: A), end: x == 3;",
        "type A = { 1, 2, 3, 4, 5 };
        begin(x: A), __gen_0_begin_x_4(x: A): x != 4;
        __gen_0_begin_x_4(x: A), __gen_0_begin_x_5(x: A): x != 5;
        __gen_0_begin_x_5(x: A), end: ;"
    );
}
