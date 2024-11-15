use crate::ast::{
    Constant, Edge, Error, Expression, Game, Label, Node, Type, Typedef, Value, ValueEntry,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::position::Span;

use super::{gen_fresh_node, max_node_id};

type Id = Arc<str>;
type NodeEdges = (Node<Id>, Vec<Edge<Id>>);
type TypeValue = (Arc<Type<Id>>, Arc<Value<Id>>);
const BOOL_FALSE: &str = "0";
const BOOL_TRUE: &str = "1";
const BOOL_TYPE: &str = "Bool";

impl Game<Id> {
    pub fn join_generators(&mut self) -> Result<(), Error<Id>> {
        while self.join_generators_step() {}
        Ok(())
    }

    fn join_generators_step(&mut self) -> bool {
        let next_edges = self.next_edges();
        let prev_edges = self.prev_edges();
        let span = Span::none();
        let mut max_id = max_node_id(&self.nodes());
        for (node, outgoing_edges) in &next_edges {
            if node.has_bindings() || outgoing_edges.len() < 2 {
                continue;
            }

            let Some((id, (target, edges_to_remove), (type_, value))) =
                self.collect_binding_comparisons(outgoing_edges, &next_edges, &prev_edges)
            else {
                continue;
            };
            let Some(id_type) = self
                .resolve_constant(&id)
                .map(|c| c.type_.clone())
                .or_else(|| self.resolve_variable(&id).map(|v| v.type_.clone()))
            else {
                continue;
            };
            let constant_id = (1..)
                .map(|x| Id::from(format!("joined_{x}")))
                .find(|x| self.constants.iter().all(|y| y.identifier != *x))
                .unwrap();
            let type_id = (1..)
                .map(|x| Id::from(format!("Joined_{x}")))
                .find(|x| self.typedefs.iter().all(|y| y.identifier != *x))
                .unwrap();
            let new_type = Arc::new(Type::new(type_id.clone()));
            let mut intermediate_node = gen_fresh_node(&mut max_id);
            let binding: Id = Arc::from(format!("bind_{type_id}"));
            intermediate_node.add_binding(binding.clone(), new_type.clone());

            let bool_type = Arc::new(Type::new(Arc::from(BOOL_TYPE)));
            let check_expr = Expression::Access {
                span,
                lhs: Arc::new(Expression::Access {
                    span,
                    lhs: new_expr(constant_id.clone()),
                    rhs: new_expr(id.clone()),
                }),
                rhs: new_expr(binding.clone()),
            };
            let first_label = Label::Comparison {
                lhs: Arc::new(check_expr),
                rhs: new_expr(Arc::from(BOOL_TRUE)),
                negated: false,
            };
            let first_edge = Edge::new((*node).clone(), intermediate_node.clone(), first_label);
            let second_label = Label::Assignment {
                lhs: new_expr(id),
                rhs: new_expr(binding),
            };
            let second_edge = Edge::new(intermediate_node, target, second_label);

            self.edges.retain(|edge| !edges_to_remove.contains(edge));
            self.typedefs
                .push(Typedef::new(span, type_id.clone(), type_));

            let constant_type = Type::Arrow {
                lhs: id_type,
                rhs: Arc::new(Type::Arrow {
                    lhs: new_type,
                    rhs: bool_type,
                }),
            };
            self.constants.push(Constant::new(
                span,
                constant_id.clone(),
                Arc::new(constant_type),
                value,
            ));
            self.edges.push(first_edge);
            self.edges.push(second_edge);

            return true;
        }

        false
    }

    fn collect_binding_comparisons(
        &self,
        edges: &BTreeSet<&Edge<Id>>,
        next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Edge<Id>>>,
        prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Edge<Id>>>,
    ) -> Option<(Id, NodeEdges, TypeValue)> {
        let first = edges.iter().next()?;
        let target = next_edges.get(&first.rhs)?.first()?.rhs.clone();
        let Label::Comparison { lhs, .. } = &first.label else {
            return None;
        };
        let identifier = lhs.uncast().as_reference()?;
        let mut generators = vec![];
        let mut edges_to_remove = vec![];
        for edge in edges {
            if let Label::Comparison { lhs, rhs, negated } = &edge.label {
                if *negated || lhs.uncast().as_reference()? != identifier {
                    return None;
                }
                let rhs = rhs.uncast().as_reference()?;
                let outgoing_edges = next_edges.get(&edge.rhs)?;
                let outgoing_edge = outgoing_edges.first()?;
                if outgoing_edges.len() != 1
                    || prev_edges.get(&edge.rhs)?.len() != 1
                    || outgoing_edge.rhs != target
                {
                    return None;
                }
                edges_to_remove.push((*edge).clone());
                edges_to_remove.push((*outgoing_edge).clone());
                let generator_members = self.get_generator_members(outgoing_edge, identifier)?;
                generators.push((rhs.clone(), generator_members));
            }
        }
        let (type_, value) = merge_generators(generators);

        Some((
            identifier.clone(),
            (target, edges_to_remove),
            (Arc::new(type_), Arc::new(value)),
        ))
    }

    fn get_generator_members(&self, edge: &Edge<Id>, identifier: &Id) -> Option<Vec<Id>> {
        let Label::Assignment { lhs, rhs } = &edge.label else {
            return None;
        };
        if lhs.uncast().as_reference()? != identifier {
            return None;
        }
        let rhs = rhs.uncast().as_reference()?;
        if let Some((binding, type_)) = edge.get_binding(rhs) {
            if binding == rhs {
                return type_.values(self).ok();
            }
        } else if self.resolve_constant(rhs).is_none() && self.resolve_variable(rhs).is_none() {
            return Some(vec![rhs.clone()]);
        }
        None
    }
}

fn merge_generators(generators: Vec<(Id, Vec<Id>)>) -> (Type<Id>, Value<Id>) {
    let mut possible_outcomes: BTreeMap<Id, BTreeSet<Id>> = BTreeMap::new();
    let mut all_outcomes: BTreeSet<Id> = BTreeSet::new();
    for (condition, members) in generators {
        all_outcomes.extend(members.iter().cloned());
        possible_outcomes
            .entry(condition)
            .or_default()
            .extend(members);
    }
    let always_false = ValueEntry::new_default(Arc::from(Value::new(Arc::from(BOOL_FALSE))));
    let mut entries: Vec<_> = possible_outcomes
        .into_iter()
        .map(|(condition, possible_outcomes)| {
            let mut entries: Vec<_> = possible_outcomes
                .into_iter()
                .map(|outcome| {
                    ValueEntry::new(
                        Span::none(),
                        Some(outcome),
                        Arc::from(Value::new(Arc::from(BOOL_TRUE))),
                    )
                })
                .collect();
            entries.push(always_false.clone());
            ValueEntry::new(
                Span::none(),
                Some(condition),
                Arc::from(Value::Map {
                    span: Span::none(),
                    entries,
                }),
            )
        })
        .collect();
    let default_outcome = ValueEntry::new_default(Arc::from(Value::Map {
        span: Span::none(),
        entries: vec![always_false],
    }));
    entries.push(default_outcome);
    let value = Value::Map {
        span: Span::none(),
        entries,
    };

    let type_ = Type::Set {
        span: Span::none(),
        identifiers: all_outcomes.into_iter().collect(),
    };

    (type_, value)
}

fn new_expr(id: Id) -> Arc<Expression<Id>> {
    Arc::new(Expression::new(id))
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        join_generators,
        small1,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        type Type2 = {1,2,3};
        type Type3 = {1,3,6};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, a2(t2: Type2): coord == All(2);
        begin, a3(t3: Type3): coord == All(3);
        a1(t1: Type1), end: coord = t1;
        a2(t2: Type2), end: coord = t2;
        a3(t3: Type3), end: coord = t3;",
        "type All = { 1, 2, 3, 4, 5, 6 };
        type Type1 = { 3, 4, 5 };
        type Type2 = { 1, 2, 3 };
        type Type3 = { 1, 3, 6 };
        type Joined_1 = { 1, 2, 3, 4, 5, 6 };
        const joined_1: All -> Joined_1 -> Bool = { 1: { 3: 1, 4: 1, 5: 1, :0 }, 2: { 1: 1, 2: 1, 3: 1, :0 }, 3: { 1: 1, 3: 1, 6: 1, :0 }, :{ :0 } };
        var coord: All = 1;
        begin, 1(bind_Joined_1: Joined_1): joined_1[coord][bind_Joined_1] == 1;
        1(bind_Joined_1: Joined_1), end: coord = bind_Joined_1;"
    );

    test_transform!(
        join_generators,
        small2,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, f: coord == 1;
        f, e: coord = 2;
        a1(t1: Type1), end: coord = t1;"
    );

    test_transform!(
        join_generators,
        small3,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, f: ;
        a1(t1: Type3), end: coord = t1;"
    );

    test_transform!(
        join_generators,
        small4,
        "type All = {1,2,3,4,5,6};
        type Type1 = {3,4,5};
        type Type2 = {1,2,3};
        type Type3 = {1,3,6};
        var coord : All = 1;
        begin, a1(t1: Type1): coord == All(1);
        begin, a2(t2: Type2): coord == All(2);
        begin, a2(t2: Type2): ;
        begin, a3(t3: Type3): coord == All(3);
        a1(t1: Type1), end: coord = t1;
        a2(t2: Type2), end: coord = t2;
        a3(t3: Type3), end: coord = t3;"
    );
}
