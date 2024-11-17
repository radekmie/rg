use crate::ast::analyses::ConstantsAnalysis;
use crate::ast::{Edge, Error, Expression, Game, Label, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

struct Context<'a> {
    constant_vars: &'a BTreeMap<Id, Arc<Value<Id>>>,
    constants: &'a BTreeMap<Id, Arc<Value<Id>>>,
    variables: &'a BTreeSet<Id>,
}

impl Context<'_> {
    fn dereference_constant<'a>(
        &'a self,
        value: &'a Value<Id>,
        dereference_map: bool,
    ) -> &'a Value<Id> {
        if let Value::Element { identifier } = value {
            self.constants
                .get(identifier)
                .filter(|dereferenced| !dereferenced.is_map() || dereference_map)
                .map_or_else(
                    || value,
                    |dereferenced| self.dereference_constant(dereferenced, dereference_map),
                )
        } else {
            value
        }
    }

    fn get_identifier(&self, identifier: &Id, edge: &Edge<Id>) -> Option<Id> {
        if edge.has_binding(identifier) {
            None
        } else if self.variables.contains(identifier) {
            self.dereference_constant(self.constant_vars.get(identifier)?, false)
                .to_identifier()
                .cloned()
        } else {
            self.dereference_constant(&Value::new(identifier.clone()), false)
                .to_identifier()
                .cloned()
        }
    }

    fn get_value(&self, identifier: &Id, edge: &Edge<Id>) -> Option<Value<Id>> {
        if edge.has_binding(identifier) {
            None
        } else if self.variables.contains(identifier) {
            Some(
                self.dereference_constant(self.constant_vars.get(identifier)?, true)
                    .clone(),
            )
        } else {
            self.constants.get(identifier).map_or_else(
                || Some(Value::new(identifier.clone())),
                |value| Some(self.dereference_constant(value, true).clone()),
            )
        }
    }
}

impl Game<Id> {
    pub fn propagate_constants(&mut self) -> Result<(), Error<Id>> {
        let (analysis, context) = self.analyse_with_context::<ConstantsAnalysis>(true);
        let default_constant_vars = &BTreeMap::new();
        let constants = &context.constants;
        let variables = &context.variables;
        let mut unreachable_edges = Vec::new();

        for edge in &mut self.edges {
            if edge.label.is_player_assignment() {
                continue;
            }

            let edge_clone = edge.clone();
            let edge = Arc::make_mut(edge);
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
                Label::Comparison { lhs, rhs, negated } => {
                    let context = Context {
                        constant_vars: analysis.get(&edge.lhs).unwrap_or(default_constant_vars),
                        constants,
                        variables,
                    };

                    *lhs = Arc::new(eval_expression(lhs, &context, &edge_clone));
                    *rhs = Arc::new(eval_expression(rhs, &context, &edge_clone));

                    let lhs_value = lhs
                        .uncast()
                        .as_reference()
                        .and_then(|id| context.get_value(id, &edge_clone));
                    let rhs_value = rhs
                        .uncast()
                        .as_reference()
                        .and_then(|id| context.get_value(id, &edge_clone));

                    // Skip or remove comparisons between constants
                    if let (Some(lhs_value), Some(rhs_value)) = (lhs_value, rhs_value) {
                        if lhs_value == rhs_value {
                            if *negated {
                                unreachable_edges.push(edge.clone());
                            } else {
                                edge.skip();
                            }
                        } else if *negated {
                            edge.skip();
                        } else {
                            unreachable_edges.push(edge.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        self.edges.retain(|edge| !unreachable_edges.contains(edge));

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
            eval_access(lhs, rhs, context, edge, false).map_or_else(
                || Expression::Access {
                    span: *span,
                    lhs: Arc::new(eval_expression(lhs, context, edge)),
                    rhs: Arc::new(eval_expression(rhs, context, edge)),
                },
                |value| {
                    value
                        .as_identifier()
                        .map_or_else(|| expression.clone(), Expression::new)
                },
            )
        }
        Expression::Cast { span, lhs, rhs } => Expression::Cast {
            span: *span,
            lhs: lhs.clone(),
            rhs: Arc::new(eval_expression(rhs, context, edge)),
        },
        Expression::Reference { identifier } => context
            .get_identifier(identifier, edge)
            .map_or_else(|| expression.clone(), Expression::new),
    }
}

fn eval_access(
    lhs: &Expression<Id>,
    rhs: &Expression<Id>,
    context: &Context,
    edge: &Edge<Id>,
    dereference_map: bool,
) -> Option<Value<Id>> {
    eval_as_map(lhs, context, edge)?
        .get_entry(&eval_as_identifier(rhs, context, edge)?)
        .map(|value| context.dereference_constant(value, dereference_map).clone())
}

fn eval_as_map(
    expression: &Expression<Id>,
    context: &Context,
    edge: &Edge<Id>,
) -> Option<Value<Id>> {
    match expression {
        Expression::Access { lhs, rhs, .. } => eval_access(lhs, rhs, context, edge, true),
        Expression::Cast { rhs, .. } => eval_as_map(rhs, context, edge),
        Expression::Reference { identifier } => context.get_value(identifier, edge),
    }
}

fn eval_as_identifier(
    expression: &Expression<Id>,
    context: &Context,
    edge: &Edge<Id>,
) -> Option<Id> {
    match expression {
        Expression::Access { lhs, rhs, .. } => {
            eval_access(lhs, rhs, context, edge, false).and_then(Value::as_identifier)
        }
        Expression::Cast { rhs, .. } => eval_as_identifier(rhs, context, edge),
        Expression::Reference { identifier } => context.get_identifier(identifier, edge),
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
        a, c: y = 3;
        b, c: y = 3;"
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
        const_dependency1,
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

    test_transform!(
        propagate_constants,
        const_dependency2,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const cst3: A -> A -> A = { b: { a:cst2, :c }, :{ :cst1 } };
        var x: A -> A -> A = cst3;
        begin, end: x = cst3[b][cst2];",
        "type A = { a, b, c };
        const cst1: A = a;
        const cst2: A = cst1;
        const cst3: A -> A -> A = { b: { a:cst2, :c }, :{ :cst1 } };
        var x: A -> A -> A = cst3;
        begin, end: x = a;"
    );

    test_transform!(
        propagate_constants,
        const_dependency3,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const board: A -> A = { b: cst2, :c };
        const cst3: A -> A -> A = { a: board, :{ a:cst2, :c } };
        var x: A = c;
        var y: A -> A = { :cst1 };
        begin, a1: y = cst3[cst2];
        a1, end: x = y[b];",
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const board: A -> A = { b: cst2, :c };
        const cst3: A -> A -> A = { a: board, :{ a:cst2, :c } };
        var x: A = c;
        var y: A -> A = { :cst1 };
        begin, a1: y = board;
        a1, end: x = a;"
    );

    test_transform!(
        propagate_constants,
        const_dependency4,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const board: A -> A = { b: cst2, :c };
        const board1: A -> A = board;
        const cst3: A -> A -> A = { a: board1, :{ a:cst2, :c } };
        var x: A = c;
        var y: A -> A = { :cst1 };
        begin, a1: y = cst3[cst2];
        a1, end: x = y[b];",
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const board: A -> A = { b: cst2, :c };
        const board1: A -> A = board;
        const cst3: A -> A -> A = { a: board1, :{ a:cst2, :c } };
        var x: A = c;
        var y: A -> A = { :cst1 };
        begin, a1: y = board;
        a1, end: x = a;"
    );

    test_transform!(
        propagate_constants,
        const_dependency5,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const board: A -> A = { b: cst2, :c };
        const board1: A -> A = board;
        const cst3: A -> A -> A = { a: board1, :{ a:cst2, :c } };
        var x: A = c;
        begin, end: x = cst3[cst2][b];",
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const board: A -> A = { b: cst2, :c };
        const board1: A -> A = board;
        const cst3: A -> A -> A = { a: board1, :{ a:cst2, :c } };
        var x: A = c;
        begin, end: x = a;"
    );

    test_transform!(
        propagate_constants,
        constant_comparisons1,
        "type A = {1,2,3,4};
        begin, end: ;
        a, b: 1 == 1;
        a, c: 1 != 1;
        a, d: 1 == 2;
        a, e: 1 != 2;",
        "type A = { 1, 2, 3, 4 };
        begin, end: ;
        a, b: ;
        a, e: ;"
    );

    test_transform!(
        propagate_constants,
        constant_comparisons2,
        "type A = {1,2,3,4};
        begin, end: ;
        a(bind_1: A), b: bind_1 == 1;
        a(bind_1: A), c: bind_1 != 1;
        a(bind_1: A), d: bind_1 == 2;
        a(bind_1: A), e: bind_1 != 2;"
    );

    test_transform!(
        propagate_constants,
        player_constant1,
        "type Player = {white, black};
        var player: Player = white;
        begin, a: player = black;
        a, end: player == black;",
        "type Player = { white, black };
        var player: Player = white;
        begin, a: player = black;
        a, end: ;"
    );

    test_transform!(
        propagate_constants,
        player_constant2,
        "type Player = {white, black};
        var player: Player = white;
        begin, a: player = white;
        a, end: player == white;",
        "type Player = { white, black };
        var player: Player = white;
        begin, a: player = white;
        a, end: ;"
    );

    test_transform!(
        propagate_constants,
        inside_subst,
        "type A = {a,b,c};
        const up: A -> A = {a: b, :c};
        const up1: A -> A = up;
        begin, a(board: A -> A)(x: A): c == board[up1[x]];",
        "type A = { a, b, c };
        const up: A -> A = { a: b, :c };
        const up1: A -> A = up;
        begin, a(board: A -> A)(x: A): c == board[up[x]];"
    );
}
