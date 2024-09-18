use crate::ast::analyses::ConstantsAnalysis;
use crate::ast::{Constant, Edge, Error, Expression, Game, Label, Value, Variable};
use std::collections::BTreeMap;
use std::sync::Arc;

type Id = Arc<str>;
type ConstantVars = BTreeMap<Id, Arc<Value<Id>>>;

struct Context<'a> {
    constants: &'a Vec<Constant<Id>>,
    variables: &'a Vec<Variable<Id>>,
    constant_vars: &'a ConstantVars,
}

impl Context<'_> {
    fn get(&self, identifier: &Id, edge: &Edge<Id>) -> Option<Value<Id>> {
        if edge.has_binding(identifier) {
            None
        } else if self.variables.iter().any(|v| &v.identifier == identifier) {
            self.constant_vars
                .get(identifier)
                .map(|value| value.as_ref().clone())
        } else {
            self.constants
                .iter()
                .find(|c| &c.identifier == identifier)
                .map(|c| c.value.as_ref().clone())
                .or_else(|| Some(Value::new(identifier.clone())))
        }
    }
}

impl Game<Id> {
    pub fn propagate_constants(&mut self) -> Result<(), Error<Id>> {
        let analysis = self.analyse::<ConstantsAnalysis>(true);
        let default_constant_vars = &BTreeMap::new();
        let constants = &self.constants;
        let variables = &self.variables;
        for edge in &mut self.edges {
            if edge.label.is_player_assignment() {
                continue;
            }
            let edge_clone = edge.clone();
            match &mut edge.label {
                Label::Assignment { lhs, rhs } => {
                    let context = Context {
                        constants,
                        variables,
                        constant_vars: analysis.get(&edge.lhs).unwrap_or(default_constant_vars),
                    };
                    let lhs_opt = eval_expression(lhs, &context, &edge_clone);
                    let rhs_opt = eval_expression(rhs, &context, &edge_clone);
                    if let Some(new_rhs) = rhs_opt {
                        if let Some(new_lhs) = lhs_opt {
                            if new_lhs == new_rhs {
                                edge.skip();
                            } else {
                                *rhs = Arc::new(new_rhs);
                            }
                        } else {
                            *rhs = Arc::new(new_rhs);
                        }
                    }
                }
                Label::Comparison { lhs, rhs, .. } => {
                    let context = Context {
                        constants,
                        variables,
                        constant_vars: analysis.get(&edge.lhs).unwrap_or(default_constant_vars),
                    };
                    let lhs_opt = eval_expression(lhs, &context, &edge_clone);
                    let rhs_opt = eval_expression(rhs, &context, &edge_clone);
                    if let Some(new_lhs) = lhs_opt {
                        *lhs = Arc::new(new_lhs);
                    }
                    if let Some(new_rhs) = rhs_opt {
                        *rhs = Arc::new(new_rhs);
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }
}

fn eval_expression(
    expression: &Arc<Expression<Id>>,
    context: &'_ Context,
    edge: &Edge<Id>,
) -> Option<Expression<Id>> {
    match expression.as_ref() {
        Expression::Access { span, lhs, rhs } => {
            let lhs_val = eval_value(lhs, context, edge);
            let rhs_val = eval_value(rhs, context, edge);
            match (lhs_val, rhs_val) {
                (Some(Value::Map { entries, .. }), Some(Value::Element { identifier })) => {
                    let entry = entries
                        .iter()
                        .find(|entry| {
                            entry
                                .identifier
                                .as_ref()
                                .is_some_and(|id| *id == identifier)
                        })
                        .or_else(|| entries.iter().find(|entry| entry.identifier.is_none()));

                    entry.and_then(|entry| entry.value.as_ref().clone().to_expression())
                }
                (
                    Some(Value::Element {
                        identifier: l_identifier,
                    }),
                    Some(Value::Element {
                        identifier: r_identifier,
                    }),
                ) => Some(Expression::Access {
                    span: *span,
                    lhs: Arc::new(Expression::new(l_identifier)),
                    rhs: Arc::new(Expression::new(r_identifier)),
                }),
                (Some(Value::Element { identifier }), None) => Some(Expression::Access {
                    span: *span,
                    lhs: Arc::new(Expression::new(identifier)),
                    rhs: rhs.clone(),
                }),
                (None, Some(Value::Element { identifier })) => Some(Expression::Access {
                    span: *span,
                    lhs: lhs.clone(),
                    rhs: Arc::new(Expression::new(identifier)),
                }),
                _ => None,
            }
        }
        Expression::Cast { span, lhs, rhs } => {
            let rhs = eval_expression(rhs, context, edge)?;
            Some(Expression::Cast {
                span: *span,
                lhs: lhs.clone(),
                rhs: Arc::new(rhs),
            })
        }
        Expression::Reference { .. } => {
            eval_value(expression, context, edge).and_then(Value::to_expression)
        }
    }
}

fn eval_value(
    expression: &Arc<Expression<Id>>,
    context: &Context,
    edge: &Edge<Id>,
) -> Option<Value<Id>> {
    match expression.as_ref() {
        Expression::Access { lhs, rhs, .. } => {
            let rhs = eval_value(rhs, context, edge)?;
            let lhs = eval_value(lhs, context, edge)?;
            rhs.as_identifier()
                .and_then(|identifier| lhs.get_entry(identifier).cloned())
        }
        Expression::Cast { rhs, .. } => eval_value(rhs, context, edge),
        Expression::Reference { identifier } => context.get(identifier, edge),
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        propagate_constants,
        small,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var x: A = 3;
        begin, a(bind_1: A): bind_1 == down[x];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var x: A = 3;
        begin, a(bind_1: A): bind_1 == 2;"
    );

    test_transform!(
        propagate_constants,
        small1,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        begin, a(bind_1: A): bind_1 == down[board[x]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        begin, a(bind_1: A): bind_1 == 1;"
    );

    test_transform!(
        propagate_constants,
        small2,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        begin, a(bind_1: A): A(bind_1) == down[A(board[x])];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        begin, a(bind_1: A): A(bind_1) == 1;"
    );

    test_transform!(
        propagate_constants,
        small3,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        begin, a(bind_1: A): bind_1 == down[board[x]];
        a(bind_1: A), a: x = bind_1;
        a, begin: ; "
    );

    test_transform!(
        propagate_constants,
        small4,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        begin, a(x: A): x == down[board[2]];
        a(x: A), a: $ x;
        a, begin: x == 4;",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        begin, a(x: A): x == 1;
        a(x: A), a: $ x;
        a, begin: 3 == 4;"
    );

    test_transform!(
        propagate_constants,
        small5,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var y: A = 1;
        var x: A = 3;
        begin, a: x == down[2];
        begin, b: A(x) == down[3];
        a, c: y = x;
        b, c: y = x;",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var y: A = 1;
        var x: A = 3;
        begin, a: 3 == 1;
        begin, b: A(3) == 2;
        a, c: y = 3;
        b, c: y = 3;"
    );

    // test_transform!(
    //     propagate_constants,
    //     small6,
    //     "turn_11, end: ;",
    //     "x, z: ;"
    // );

    // test_transform!(
    //     propagate_constants,
    //     small7,
    //     "turn_11, end: ;",
    //     "x, z: ;"
    // );

    // test_transform!(
    //     propagate_constants,
    //     small8,
    //     "turn_11, end: ;",
    //     "x, z: ;"
    // );

    // test_transform!(
    //     propagate_constants,
    //     small9,
    //     "turn_11, end: ;",
    //     "x, z: ;"
    // );
}
