use crate::ast::analyses::ConstantsAnalysis;
use crate::ast::{Error, Expression, Game, Label, Type, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

#[derive(Debug, Clone)]
struct Context<'a> {
    constant_exprs: &'a BTreeMap<Expression<Id>, Arc<Value<Id>>>,
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

    fn get_identifier(&self, expr: &Expression<Id>) -> Option<Id> {
        let value = match expr.uncast() {
            Expression::Reference { identifier } if !self.variables.contains(identifier) => {
                &Value::new(identifier.clone())
            }
            _ => self.constant_exprs.get(expr)?,
        };

        self.dereference_constant(value, false)
            .to_identifier()
            .cloned()
    }

    fn get_value(&self, expr: &Expression<Id>) -> Option<Value<Id>> {
        match expr.uncast() {
            Expression::Reference { identifier } if !self.variables.contains(identifier) => {
                self.constants.get(identifier).map_or_else(
                    || Some(Value::new(identifier.clone())),
                    |value| Some(self.dereference_constant(value, true).clone()),
                )
            }
            _ => Some(
                self.dereference_constant(self.constant_exprs.get(expr)?, true)
                    .clone(),
            ),
        }
    }
}

impl Game<Id> {
    pub fn propagate_constants(&mut self) -> Result<(), Error<Id>> {
        let constant_analysis = ConstantsAnalysis::from(&*self);
        let analysis = self.analyse(&constant_analysis);
        let default_constant_vars = &BTreeMap::new();
        let ConstantsAnalysis {
            constants,
            variables,
        } = &constant_analysis;
        let game = Self {
            constants: self.constants.clone(),
            typedefs: self.typedefs.clone(),
            variables: self.variables.clone(),
            ..Self::default()
        };

        for edge in &mut self.edges {
            if edge.label.is_player_assignment() {
                continue;
            }

            let edge = Arc::make_mut(edge);
            match &mut edge.label {
                Label::Assignment { lhs, rhs } => {
                    let context = Context {
                        constants,
                        variables,
                        constant_exprs: analysis.get(&edge.lhs).unwrap_or(default_constant_vars),
                    };

                    let new_lhs = eval_expression(lhs, &context, &game, false);
                    let new_rhs = eval_expression(rhs, &context, &game, false);

                    if new_lhs == new_rhs {
                        edge.skip();
                    } else {
                        *rhs = Arc::new(new_rhs);
                        if new_lhs != **lhs && new_lhs.is_access() {
                            *lhs = Arc::new(new_lhs)
                        }
                    }
                }
                Label::Comparison { lhs, rhs, negated } => {
                    let context = Context {
                        constant_exprs: analysis.get(&edge.lhs).unwrap_or(default_constant_vars),
                        constants,
                        variables,
                    };

                    *lhs = Arc::new(eval_expression(lhs, &context, &game, false));
                    *rhs = Arc::new(eval_expression(rhs, &context, &game, false));

                    let lhs_value = context.get_value(&lhs);
                    let rhs_value = context.get_value(&rhs);

                    // Skip comparisons between constants
                    if let (Some(lhs_value), Some(rhs_value)) = (lhs_value, rhs_value) {
                        if (lhs_value == rhs_value) != *negated {
                            edge.skip();
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn new_casted_expression(
        &self,
        id: Id,
        expression: &Expression<Id>,
        in_cast: bool,
    ) -> Option<Expression<Id>> {
        if !in_cast && self.is_symbol(&id) {
            let type_ = expression.infer(self).ok()?;
            if let Type::TypeReference { .. } = type_.as_ref() {
                Some(Expression::new_cast(type_, Arc::new(Expression::new(id))))
            } else {
                None
            }
        } else {
            Some(Expression::new(id))
        }
    }
}

fn eval_expression(
    expression: &Expression<Id>,
    context: &Context,
    game: &Game<Id>,
    in_cast: bool,
) -> Expression<Id> {
    context
        .get_identifier(expression)
        .and_then(|id| game.new_casted_expression(id, expression, in_cast))
        .unwrap_or_else(|| match expression {
            Expression::Access { lhs, rhs, span } => {
                // First, try to evaluate lhs as Value::Map and rhs as Value::Identifier
                eval_access(lhs, rhs, context, false).map_or_else(
                    || Expression::Access {
                        span: *span,
                        lhs: Arc::new(eval_expression(lhs, context, game, false)),
                        rhs: Arc::new(eval_expression(rhs, context, game, false)),
                    },
                    |value| {
                        value
                            .as_identifier()
                            .and_then(|id| game.new_casted_expression(id, expression, in_cast))
                            .unwrap_or_else(|| expression.clone())
                    },
                )
            }
            Expression::Cast { span, lhs, rhs } => Expression::Cast {
                span: *span,
                lhs: lhs.clone(),
                rhs: Arc::new(eval_expression(rhs, context, game, true)),
            },
            Expression::Reference { .. } => expression.clone(),
        })
}

fn eval_access(
    lhs: &Expression<Id>,
    rhs: &Expression<Id>,
    context: &Context,
    dereference_map: bool,
) -> Option<Value<Id>> {
    eval_as_map(lhs, context)?
        .get_entry(&eval_as_identifier(rhs, context)?)
        .map(|value| context.dereference_constant(value, dereference_map).clone())
}

fn eval_as_map(expression: &Expression<Id>, context: &Context) -> Option<Value<Id>> {
    context.get_value(expression).or_else(|| match expression {
        Expression::Access { lhs, rhs, .. } => eval_access(lhs, rhs, context, true),
        Expression::Cast { rhs, .. } => eval_as_map(rhs, context),
        Expression::Reference { .. } => None,
    })
}

fn eval_as_identifier(expression: &Expression<Id>, context: &Context) -> Option<Id> {
    context
        .get_identifier(expression)
        .or_else(|| match expression {
            Expression::Access { lhs, rhs, .. } => {
                eval_access(lhs, rhs, context, false).and_then(Value::as_identifier)
            }
            Expression::Cast { rhs, .. } => eval_as_identifier(rhs, context),
            Expression::Reference { .. } => None,
        })
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
        var y: A = 1;
        begin, a: y = A(*);
        a, b: y == down[x];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: y == A(2);"
    );

    test_transform!(
        propagate_constants,
        small1,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: y == down[board[x]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: y == A(1);"
    );

    test_transform!(
        propagate_constants,
        small2,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: A(y) == down[A(board[x])];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: A(y) == A(1);"
    );

    test_transform!(
        propagate_constants,
        small3,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: y == down[board[x]];
        b, c: x = y;
        c, a: ; "
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
        begin, a: A(3) == A(1);
        begin, b: A(3) == A(2);
        a, c: y = A(3);
        b, c: y = A(3);"
    );

    test_transform!(
        propagate_constants,
        small6,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        var y: AA = {:1};
        begin, a: y = AA(*);
        a, b: x = y[down[3]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        var y: AA = {:1};
        begin, a: y = AA(*);
        a, b: x = y[A(2)];"
    );

    test_transform!(
        propagate_constants,
        small7,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A -> A = {4:{:3}, :{2:2, :3}};
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: x = board[1][down[x]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A -> A = { 4: { :3 }, :{ 2: 2, :3 } };
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: x = A(2);"
    );

    test_transform!(
        propagate_constants,
        small8,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var board: A -> A -> A -> A = { :{2:{:2}, :{:3}}};
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: x = board[1][y][down[x]];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        const down: AA = { 4: 3, 3: 2, :1 };
        var board: A -> A -> A -> A = { :{ 2: { :2 }, :{ :3 } } };
        var x: A = 3;
        var y: A = 1;
        begin, a: y = A(*);
        a, b: x = board[1][y][A(2)];"
    );

    test_transform!(
        propagate_constants,
        small9,
        "type A = {1,2,3,4};
        type AA = A -> A;
        var board: A -> A = {4:3, :2};
        var x: A = 3;
        var y: A = 1;
        begin, a1: y = A(*);
        a1, a: board[1] = y;
        begin, b: x = board[2];
        a, c: x = board[2];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        var board: A -> A = { 4: 3, :2 };
        var x: A = 3;
        var y: A = 1;
        begin, a1: y = A(*);
        a1, a: board[1] = y;
        begin, b: x = A(2);
        a, c: x = board[2];"
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
        begin, end: x = A(a);"
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
        begin, end: x = A(a);"
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
        a1, end: x = A(a);"
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
        a1, end: x = A(a);"
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
        begin, end: x = A(a);"
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
        a, c: 1 != 1;
        a, d: 1 == 2;
        a, e: ;"
    );

    test_transform!(
        propagate_constants,
        constant_comparisons2,
        "type A = {1,2,3,4};
        begin, end: ;
        var x: A = 1;
        begin, a: x = A(*);
        a, b: x == 1;
        a, c: x != 1;
        a, d: x == 2;
        a, e: x != 2;"
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
        type AA = A -> A;
        const up: A -> A = {a: b, :c};
        const up1: A -> A = up;
        var board: AA = {:a};
        var x: A = a;
        begin, a: board = AA(*);
        a, b: x = A(*);
        b, c: c == board[up1[x]];",
        "type A = {a,b,c};
        type AA = A -> A;
        const up: A -> A = {a: b, :c};
        const up1: A -> A = up;
        var board: AA = {:a};
        var x: A = a;
        begin, a: board = AA(*);
        a, b: x = A(*);
        b, c: c == board[up[x]];"
    );

    test_transform!(
        propagate_constants,
        expr1,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var x: A = 3;
        var y: A = 1;
        var board: A -> A = {4:3, :2};
        begin, a: y = A(*);
        a, b: board[y] == 2;
        b, c: x = A(*);
        c, d: x == board[y];",
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var x: A = 3;
        var y: A = 1;
        var board: A -> A = {4:3, :2};
        begin, a: y = A(*);
        a, b: board[y] == 2;
        b, c: x = A(*);
        c, d: x == A(2);"
    );

    test_transform!(
        propagate_constants,
        expr2,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var x: A = 3;
        var y: A = 1;
        var board: A -> A = {4:3, :2};
        begin, a: y = A(*);
        a, b: board[y] == 2;
        b, c1: board[1] = 3;
        b, c2: x = A(*);
        c1, c: ;
        c2, c: ;
        c, end: x == board[y];"
    );

    test_transform!(
        propagate_constants,
        expr3,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var x: A = 3;
        var y: A = 1;
        var board: A -> A = {4:3, :2};
        begin, a: y = A(*);
        a, b: board[y] == 2;
        b, c1: y = A(*);
        b, c2: x = A(*);
        c1, c: ;
        c2, c: ;
        c, end: x == board[y];"
    );

    test_transform!(
        propagate_constants,
        expr4,
        "type A = {1,2,3,4};
        type AA = A -> A;
        const down: AA = {4:3, 3:2, :1};
        var x: A = 3;
        var y: A = 1;
        var board: A -> A = {4:3, :2};
        begin, a: y = A(*);
        a, b: board[y] == 2;
        b, c1: y = 4;
        b, c2: x = A(*);
        c1, c: ;
        c2, c: ;
        c, end: x == board[y];"
    );

    test_transform!(
        propagate_constants,
        expr5,
        "type A = {1,2,3,4};
        type AA = A -> A;
        var y: A = 1;
        var board: A -> A = {4:3, :2};
        begin, a: y = A(*);
        a, b: board[y] == 1;
        b, c: board[board[y]] = board[y];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        var y: A = 1;
        var board: A -> A = { 4: 3, :2 };
        begin, a: y = A(*);
        a, b: board[y] == 1;
        b, c: board[board[y]] = A(1);"
    );

    test_transform!(
        propagate_constants,
        expr6,
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        var x: A = 3;
        var y: A = 1;
        var board: A -> A = { 4: 3, :2 };
        begin, a: y = A(*);
        a, b: board[y] == 1;
        b, c: x = A(*);
        c, d: board2[x] = A(3);
        b, c: board2[board[y]] = board[y];",
        "type A = { 1, 2, 3, 4 };
        type AA = A -> A;
        var x: A = 3;
        var y: A = 1;
        var board: A -> A = { 4: 3, :2 };
        begin, a: y = A(*);
        a, b: board[y] == 1;
        b, c: x = A(*);
        c, d: board2[x] = A(3);
        b, c: board2[A(1)] = A(1);"
    );
}
