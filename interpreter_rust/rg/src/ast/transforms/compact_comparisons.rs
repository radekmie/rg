use crate::ast::{Edge, Error, Expression, Game, Label, Node, Type};
use std::{iter, sync::Arc};

type Id = Arc<str>;

impl Game<Id> {
    pub fn compact_comparisons(&mut self) -> Result<(), Error<Id>> {
        let mut to_compat = vec![];

        for node in self.nodes() {
            let Some(next_node) = self.next_node(node) else {
                continue;
            };
            let Some((expr, ids)) = self.same_id_comparisons(node) else {
                continue;
            };
            let Some(unused_members) = expr
                .infer(&self, self.outgoing_edges(node).next())
                .ok()
                .and_then(|type_| {
                    self.get_type_members(&type_).map(|members| {
                        members
                            .iter()
                            .filter(|id| !ids.contains(id))
                            .cloned()
                            .collect::<Vec<_>>()
                    })
                })
            else {
                continue;
            };

            if unused_members.len() < ids.len() {
                to_compat.push((
                    node.clone(),
                    next_node.clone(),
                    expr.clone(),
                    unused_members,
                ));
            }
        }

        if to_compat.is_empty() {
            return Ok(());
        }

        self.edges
            .retain(|edge| !to_compat.iter().any(|(node, _, _, _)| edge.lhs == *node));

        for (node, next_node, expr, unused_members) in to_compat {
            let nodes = unused_members
                .iter()
                .map(|id| Node::new(Id::from(format!("__gen_{node}_{id}"))));
            let lhss = iter::once(node.clone()).chain(nodes.clone());
            let rhss = nodes.chain(iter::once(next_node));
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

    fn get_type_members<'a>(&'a self, type_: &'a Type<Arc<str>>) -> Option<&'a Vec<Id>> {
        match type_ {
            Type::Set { identifiers, .. } => Some(identifiers),
            Type::TypeReference { identifier } => self
                .resolve_typedef(identifier)
                .map(|type_| self.get_type_members(&type_.type_))
                .flatten(),
            _ => None,
        }
    }

    fn same_id_comparisons<'a>(
        &'a self,
        node: &'a Node<Id>,
    ) -> Option<(&'a Arc<Expression<Id>>, Vec<&'a Id>)> {
        let edge = self.outgoing_edges(node).next()?;
        let Label::Comparison {
            lhs,
            rhs,
            negated: false,
        } = &edge.label
        else {
            return None;
        };
        get_compared_to(lhs, self.outgoing_edges(node))
            .map(|ids| (lhs, ids))
            .or_else(|| get_compared_to(rhs, self.outgoing_edges(node)).map(|ids| (rhs, ids)))
    }
}

fn get_compared_to<'a>(
    expr: &Arc<Expression<Id>>,
    edges: impl Iterator<Item = &'a Edge<Arc<str>>>,
) -> Option<Vec<&'a Id>> {
    edges
        .map(|edge| {
            let Label::Comparison {
                lhs,
                rhs,
                negated: false,
            } = &edge.label
            else {
                return None;
            };
            if lhs == expr {
                rhs.uncast().as_reference()
            } else if rhs == expr {
                lhs.uncast().as_reference()
            } else {
                None
            }
        })
        .collect()
}
