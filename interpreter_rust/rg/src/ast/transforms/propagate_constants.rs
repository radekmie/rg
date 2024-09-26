use crate::ast::analyses::ConstantsAnalysis;
use crate::ast::{Edge, Error, Expression, Game, Label, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type ConstantValue = Arc<Value<Id>>;
type ConstantVars = BTreeMap<Id, ConstantValue>;

struct Context<'a> {
    constants: &'a BTreeMap<Id, ConstantValue>,
    variables: &'a BTreeSet<Id>,
    constant_vars: &'a ConstantVars,
}

impl Context<'_> {
    fn get(&self, identifier: &Id, edge: &Edge<Id>) -> Option<Value<Id>> {
        if edge.has_binding(identifier) {
            None
        } else if self.variables.contains(identifier) {
            self.constant_vars
                .get(identifier)
                .map(|value| value.as_ref().clone())
        } else {
            self.constants
                .get(identifier)
                .map(|value| value.as_ref().clone())
                .or_else(|| Some(Value::new(identifier.clone())))
        }
    }
}

impl Game<Id> {
    pub fn propagate_constants(&mut self) -> Result<(), Error<Id>> {
        let (analysis, context) = self.analyse_with_context::<ConstantsAnalysis>(true);
        let default_constant_vars = &BTreeMap::new();
        let constants = &context.constants;
        let variables = &context.variables;
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
                    let new_lhs = eval_expression(lhs, &context, &edge_clone);
                    let new_rhs = eval_expression(rhs, &context, &edge_clone);
                    if new_lhs == new_rhs {
                        edge.skip();
                    } else {
                        *rhs = Arc::new(new_rhs);
                    }
                }
                Label::Comparison { lhs, rhs, .. } => {
                    let context = Context {
                        constants,
                        variables,
                        constant_vars: analysis.get(&edge.lhs).unwrap_or(default_constant_vars),
                    };
                    let new_lhs = eval_expression(lhs, &context, &edge_clone);
                    let new_rhs = eval_expression(rhs, &context, &edge_clone);
                    *lhs = Arc::new(new_lhs);
                    *rhs = Arc::new(new_rhs);
                }
                _ => (),
            }
        }

        Ok(())
    }
}

fn eval_expression(
    expression: &Expression<Id>,
    context: &Context,
    edge: &Edge<Id>,
) -> Expression<Id> {
    match expression {
        Expression::Access { lhs, rhs, span } => {
            // First, try to evaluate lhs as Value::Map and rhs as Value::Identifier
            let map_value = eval_as_map(lhs, context, edge).and_then(|lhs_val| {
                eval_as_identifier(rhs, context, edge).map(|rhs_val| (lhs_val, rhs_val))
            });
            if let Some((lhs_val, rhs_val)) = map_value {
                let entry = lhs_val.get_entry(&rhs_val);
                entry
                    .and_then(|value| value.clone().as_expression())
                    .unwrap_or_else(|| expression.clone())
            } else {
                let lhs = eval_expression(lhs, context, edge);
                let rhs = eval_expression(rhs, context, edge);
                Expression::Access {
                    span: *span,
                    lhs: Arc::new(lhs),
                    rhs: Arc::new(rhs),
                }
            }
        }
        Expression::Cast { span, lhs, rhs } => {
            let rhs = eval_expression(rhs, context, edge);
            Expression::Cast {
                span: *span,
                lhs: lhs.clone(),
                rhs: Arc::new(rhs),
            }
        }
        Expression::Reference { identifier } => context
            .get(identifier, edge)
            .and_then(Value::as_expression)
            .unwrap_or_else(|| expression.clone()),
    }
}

fn eval_as_map(
    expression: &Expression<Id>,
    context: &Context,
    edge: &Edge<Id>,
) -> Option<Value<Id>> {
    match expression {
        Expression::Access { lhs, rhs, .. } => {
            eval_as_map(lhs, context, edge).and_then(|value_map| {
                let identifier = eval_as_identifier(rhs, context, edge)?;
                value_map
                    .get_entry(&identifier)
                    .filter(|value| value.is_map())
                    .cloned()
            })
        }
        Expression::Cast { rhs, .. } => eval_as_map(rhs, context, edge),
        Expression::Reference { identifier } => context.get(identifier, edge).filter(Value::is_map),
    }
}

fn eval_as_identifier(
    expression: &Expression<Id>,
    context: &Context,
    edge: &Edge<Id>,
) -> Option<Id> {
    match expression {
        Expression::Access { lhs, rhs, .. } => {
            eval_as_map(lhs, context, edge).and_then(|value_map| {
                let identifier = eval_as_identifier(rhs, context, edge)?;
                value_map
                    .get_entry(&identifier)
                    .and_then(|value| value.clone().as_identifier())
            })
        }
        Expression::Cast { rhs, .. } => eval_as_identifier(rhs, context, edge),
        Expression::Reference { identifier } => {
            context.get(identifier, edge).and_then(Value::as_identifier)
        }
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
        begin, b: ;
        b, a(x: A): x == down[board[2]];
        a(x: A), a: $ x;
        a, b: x == 4;",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        begin, b: ;
        b, a(x: A): x == down[board[2]];
        a(x: A), a: $ x;
        a, b: x == 4;"
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
        a, c: ;
        b, c: y = 2;"
    );

    test_transform!(
        propagate_constants,
        small6,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        begin, a(bind_1: AA): x = bind_1[down[3]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        begin, a(bind_1: AA): x = bind_1[2];"
    );

    test_transform!(
        propagate_constants,
        small7,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A -> A = {4:{:3}, :{2:2, :3}};
        var x: A = 3;
        begin, a(bind_1: A): x = board[1][down[x]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A -> A = { 4: { :3 }, :{ 2: 2, :3 } };
        var x: A = 3;
        begin, a(bind_1: A): x = 2;"
    );

    test_transform!(
        propagate_constants,
        small8,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A -> A -> A = { :{2:{:2}, :{:3}}};
        var x: A = 3;
        begin, a(bind_1: A): x = board[1][bind_1][down[x]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A -> A -> A = { :{ 2: { :2 }, :{ :3 } } };
        var x: A = 3;
        begin, a(bind_1: A): x = board[1][bind_1][2];"
    );

    test_transform!(
        propagate_constants,
        small9,
        "type A = {1,2,3,4};
        type AA = A -> A;
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        begin, a(bind_1: A): board[1] = bind_1;
        begin, b: x = board[2];
        a(bind_1: A), a: x = board[2];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        begin, a(bind_1: A): board[1] = bind_1;
        begin, b: x = 2;
        a(bind_1: A), a: x = board[2];"
    );

    test_transform!(
        propagate_constants,
        const_dependency,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const cst3: A -> A -> A = { b: { :cst2 }, :{ :cst1 } };
        var x: A -> A -> A = cst3;
        begin, end: x = cst3[b][c];",
        "type A = { a, b, c };
        const cst1: A = a;
        const cst2: A = cst1;
        const cst3: A -> A -> A = { b: { :cst2 }, :{ :cst1 } };
        var x: A -> A -> A = cst3;
        begin, end: x = a;"
    );
}
