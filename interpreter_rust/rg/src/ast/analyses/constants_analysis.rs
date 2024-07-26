use super::Analysis;
use crate::ast::{Edge, Expression, Game, Label, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type ConstantValue = Arc<crate::ast::Value<Id>>;

#[derive(PartialEq)]
pub struct Context {
    variables: BTreeSet<Id>,
    constants: BTreeMap<Id, ConstantValue>,
}

pub struct ConstantsAnalysis;

impl Analysis for ConstantsAnalysis {
    type Domain = BTreeMap<Id, ConstantValue>;
    type Context = Context;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(program: &Game<Id>) -> Self::Domain {
        program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), v.default_value.clone()))
            .collect()
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        // Join the two maps, if a key is present in both maps with different values, remove it
        for (key, value) in b {
            if let Some(a_value) = a.get(&key) {
                if a_value != &value {
                    a.remove(&key);
                }
            } else {
                a.insert(key, value);
            }
        }
        a
    }

    fn kill(input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        match &edge.label.as_var_assignment() {
            Some((identifier, _)) if !edge.label.is_map_assignment() => input
                .into_iter()
                .filter(|(id, _)| id != *identifier)
                .collect(),
            _ => input,
        }
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>, ctx: &Self::Context) -> Self::Domain {
        if let Some((identifier, value)) = as_constant_assignment(&edge.label, &input, ctx) {
            input.insert(identifier, value);
        }
        input
    }

    fn get_context(program: &Game<super::Id>) -> Self::Context {
        let bindings = program
            .edges
            .iter()
            .flat_map(|e| e.bindings().iter().map(|b| b.0.clone()).collect::<Vec<_>>());
        let variables = program
            .variables
            .iter()
            .map(|v| v.identifier.clone())
            .chain(bindings)
            .collect();

        let constants = program
            .constants
            .iter()
            .map(|c| (c.identifier.clone(), c.value.clone()))
            .collect();
        Context {
            variables,
            constants,
        }
    }
}

fn as_constant_assignment(
    label: &Label<Id>,
    knowledge: &BTreeMap<Id, ConstantValue>,
    ctx: &Context,
) -> Option<(Id, ConstantValue)> {
    let (id, expr) = label.as_var_assignment()?;
    if label.is_map_assignment() {
        return None;
    }
    let value = evaluate_constant(expr, knowledge, ctx)?;
    Some((id.clone(), value))
}

fn evaluate_constant(
    expr: &Expression<Id>,
    knowledge: &BTreeMap<Id, ConstantValue>,
    ctx: &Context,
) -> Option<ConstantValue> {
    match expr {
        Expression::Access { lhs, rhs, .. } => {
            let map = evaluate_constant(lhs, knowledge, ctx)?;
            let key = evaluate_constant(rhs, knowledge, ctx)?;
            match (map.as_ref(), key.as_ref()) {
                (Value::Map { entries, .. }, Value::Element { identifier }) => {
                    let id = Some(identifier).cloned();
                    let entry = entries
                        .iter()
                        .find(|entry| entry.identifier == id)
                        .or_else(|| entries.iter().find(|entry| entry.identifier.is_none()));
                    entry.map(|entry| entry.value.clone())
                }
                _ => None,
            }
        }
        Expression::Cast { rhs, .. } => evaluate_constant(rhs, knowledge, ctx),
        Expression::Reference { identifier } if !ctx.variables.contains(identifier) => ctx
            .constants
            .get(identifier)
            .cloned()
            .or(Some(Arc::new(Value::new(identifier.clone())))),
        Expression::Reference { identifier } => knowledge.get(identifier).cloned(),
    }
}
