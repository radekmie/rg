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

        let nodes: BTreeSet<_> = self.nodes().iter().map(|n| (*n).clone()).collect();

        for (edge, expr, unused_members) in to_compat {
            let nodes = unused_members
                .iter()
                .map(|id| gen_fresh_node(format!("{expr}_{id}_{}", edge.lhs), &nodes));
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
