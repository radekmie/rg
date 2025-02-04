use super::Analysis;
use crate::ast::{Edge, Expression, Game, Label, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

#[derive(Default, PartialEq, Eq)]
pub struct Context {
    pub constants: BTreeMap<Id, Arc<Value<Id>>>,
    pub variables: BTreeSet<Id>,
}

impl Context {
    fn get_constant(&self, id: &Id) -> Option<&Arc<Value<Id>>> {
        self.constants.get(id)
    }

    fn is_variable(&self, id: &Id) -> bool {
        self.variables.contains(id)
    }
}

pub struct ConstantsAnalysis;

impl Analysis for ConstantsAnalysis {
    type Context = Context;
    type Domain = BTreeMap<Id, Arc<Value<Id>>>;

    fn bot() -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        program
            .variables
            .iter()
            .map(|v| (v.identifier.clone(), v.default_value.clone()))
            .collect()
    }

    fn join(mut a: Self::Domain, b: Self::Domain, _ctx: &Self::Context) -> Self::Domain {
        // Keep only keys present in both maps with the same value.
        a.retain(|key, value| b.get(key) == Some(value));
        a
    }

    fn get_context(program: &Game<Id>) -> Self::Context {
        let mut ctx = Self::Context::default();
        for constant in &program.constants {
            let value = constant.value.clone();
            ctx.constants.insert(constant.identifier.clone(), value);
        }

        ctx.variables = program
            .variables
            .iter()
            .map(|v| v.identifier.clone())
            .collect();
        ctx
    }

    // We can't use the default implementation, because it doesn't work for cases like:
    // x = 1;
    // x = y[x]; <- here `kill` removes `x` from `input` before `gen`, so `x` in lhs is not recognised as a constant
    fn transfer(
        mut input: Self::Domain,
        edge: &Arc<Edge<Id>>,
        ctx: &Self::Context,
    ) -> Self::Domain {
        if let Some((identifier, value)) = as_constant_assignment(edge, &input, ctx)
            .or_else(|| as_constant_comparison(edge, &input, ctx))
        {
            input.insert(identifier, value);
        } else if let Some((identifier, _)) = &edge.label.as_var_assignment() {
            input.remove(*identifier);
        }

        input
    }
}

fn as_constant_assignment(
    edge: &Edge<Id>,
    knowledge: &BTreeMap<Id, Arc<Value<Id>>>,
    ctx: &Context,
) -> Option<(Id, Arc<Value<Id>>)> {
    if edge.label.is_map_assignment() {
        return None;
    }
    let (id, expr) = edge.label.as_var_assignment()?;
    Some((id.clone(), evaluate_constant(expr, knowledge, ctx, edge)?))
}

fn as_constant_comparison(
    edge: &Edge<Id>,
    knowledge: &BTreeMap<Id, Arc<Value<Id>>>,
    ctx: &Context,
) -> Option<(Id, Arc<Value<Id>>)> {
    if let Label::Comparison {
        lhs,
        rhs,
        negated: false,
    } = &edge.label
    {
        let lhs = lhs.uncast();
        let rhs = rhs.uncast();

        let can_be_replaced = |id: &Id| ctx.is_variable(id) && !knowledge.contains_key(id);
        if lhs.is_reference_and(can_be_replaced) {
            let value = evaluate_constant(rhs, knowledge, ctx, edge)?;
            return lhs.as_reference().map(|id| (id.clone(), value));
        }

        if rhs.is_reference_and(can_be_replaced) {
            let value = evaluate_constant(lhs, knowledge, ctx, edge)?;
            return rhs.as_reference().map(|id| (id.clone(), value));
        }
    }

    None
}

fn evaluate_constant(
    expr: &Expression<Id>,
    knowledge: &BTreeMap<Id, Arc<Value<Id>>>,
    ctx: &Context,
    edge: &Edge<Id>,
) -> Option<Arc<Value<Id>>> {
    match expr {
        Expression::Access { lhs, rhs, .. } => {
            let lhs = evaluate_constant(lhs, knowledge, ctx, edge)?;
            let rhs = evaluate_constant(rhs, knowledge, ctx, edge)?;
            dereference_constant(&rhs, ctx)
                .to_identifier()
                .and_then(|identifier| {
                    dereference_constant(&lhs, ctx)
                        .get_entry(identifier)
                        .map(|entry| Arc::new(entry.clone()))
                })
        }
        Expression::Cast { rhs, .. } => evaluate_constant(rhs, knowledge, ctx, edge),
        Expression::Reference { identifier } if ctx.is_variable(identifier) => {
            knowledge.get(identifier).cloned()
        }
        Expression::Reference { identifier } => ctx
            .get_constant(identifier)
            .cloned()
            .or_else(|| Some(Arc::new(Value::new(identifier.clone())))),
    }
}

fn dereference_constant<'a>(value: &'a Arc<Value<Id>>, ctx: &'a Context) -> &'a Arc<Value<Id>> {
    match value.as_ref() {
        Value::Element { identifier } if ctx.constants.contains_key(identifier) => {
            dereference_constant(&ctx.constants[identifier], ctx)
        }
        _ => value,
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{Game, Node, Value};
    use crate::parsing::parser::parse_with_errors;
    use map_id::MapId;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    type Domain = BTreeMap<Arc<str>, Arc<Value<Arc<str>>>>;

    fn parse(input: &str) -> Game<Arc<str>> {
        let (game, errors) = parse_with_errors(input);
        assert!(errors.is_empty(), "Parse errors: {errors:?}");
        game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
    }

    fn format_analysis(analysis: BTreeMap<Node<Arc<str>>, Domain>) -> String {
        let mut result = String::new();
        result.push('\n');
        for (node, constants) in analysis {
            result.push_str(&format!("        {node}:\n"));
            for (variable, value) in constants {
                result.push_str(&format!("            {variable} = {value}\n"));
            }
        }
        result
    }

    macro_rules! test {
        ($name:ident, $source:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let game = parse($source);
                let analysis = game.analyse::<super::ConstantsAnalysis>(true);
                let actual = format_analysis(analysis);
                let actual = actual.trim();
                let expect = $expect.trim();
                assert!(
                    actual == expect,
                    "\n\n>>> Actual: <<<\n        {actual}\n>>> Expect: <<<\n        {expect}\n"
                );
            }
        };
    }

    test!(
        simple,
        "
        type A = {a,b,c};
        const cst1: A = a;
        var x: A = cst1;
        var y: A = a;
        var z: A -> A = { b: cst1, :c };
        begin, a1: y = x;
        a1, end: x = c;",
        "a1:
            x = cst1
            y = cst1
            z = { b: cst1, :c }
        begin:
            x = cst1
            y = a
            z = { b: cst1, :c }
        end:
            x = c
            y = cst1
            z = { b: cst1, :c }"
    );

    test!(
        simple_fork,
        "
        type A = {a,b,c};
        var x: A = a;
        begin, a1: x = b;
        begin, a2: x = c;
        a1, end: ;
        a2, end: ;",
        "a1:
            x = b
        a2:
            x = c
        begin:
            x = a
        end:"
    );

    test!(
        simple_loop,
        "type A = {a,b,c};
        var x: A = a;
        begin, a1: x = b;
        a1, a1: x = b;
        a1, end: ;",
        "a1:
            x = b
        begin:
            x = a
        end:
            x = b"
    );

    test!(
        simple_loop2,
        "type A = {a,b,c};
        var x: A = a;
        begin, a1: x = b;
        a1, a1(bind: A): x = bind;
        a1(bind: A), a1: ;
        a1, end: ;",
        "a1:
        a1(bind: A):
        begin:
            x = a
        end:"
    );

    test!(
        simple_loop3,
        "type A = {a,b,c};
        var x: A = a;
        begin, a1: x = b;
        a1, a2: x = c;
        a2, a1: ;
        a1, end: ;",
        "a1:
        a2:
            x = c
        begin:
            x = a
        end:"
    );

    test!(
        const_dependency1,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const cst3: A -> A -> A = { b: { :cst2 }, :{ :cst1 } };
        var x: A -> A -> A = cst3;",
        "begin:
            x = cst3"
    );

    test!(
        const_dependency2,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const cst3: A -> A -> A = { :{ :cst2 }, a: { :cst1 } };
        var x: A -> A -> A = cst3;
        var y: A = cst2;
        begin, end: y = x[y][y];",
        "begin:
            x = cst3
            y = cst2
        end:
            x = cst3
            y = cst1"
    );

    test!(
        simple_variable,
        "type A = {a,b,c};
        const cst1: A = c;
        var x: A = b;
        var y: A = cst1;
        begin, a1: x = y;
        begin, a2: x = x;",
        "a1:
            x = cst1
            y = cst1
        a2:
            x = b
            y = cst1
        begin:
            x = b
            y = cst1"
    );

    test!(
        expr_access,
        "type A = {a,b,c};
        const cst1: A = c;
        const cst2: A -> A = { b: cst1, :b };
        var x: A = b;
        var y: A -> A = cst2;
        begin, end: x = y[x];",
        "begin:
            x = b
            y = cst2
        end:
            x = cst1
            y = cst2"
    );

    test!(
        binding_loop1,
        "type Alpha = {a1,b1,c1,d1};
        var alpha: Alpha = a1;

        begin, 1: ;
        1, 2(bind_2: Alpha): Alpha(d1) == alpha;
        2(bind_2: Alpha), 3: alpha = bind_2;
        3, 4: $ bind_2;
        4, 1: ;
        3, end: ;",
        "1:
        2(bind_2: Alpha):
            alpha = d1
        3:
        4:
        begin:
            alpha = a1
        end:"
    );

    test!(
        binding_loop2,
        "type Piece = { empty, red, yellow };
        type Player = { red, yellow };
        type Position = { null, v__1_1};
        type Score = { 50, 0, 100 };
        var board: Position -> Piece = { :empty };
        var position: Position = null;
        var piece: Piece = empty;

        begin, turn_2(bind_1: Position): piece = red;
        turn_2(bind_1: Position), turn_5(bind_1: Position): board[bind_1] == empty;
        turn_5(bind_1: Position), turn_11: board[position] = piece;
        turn_11, end: ;",
        "begin:
            board = { :empty }
            piece = empty
            position = null
        end:
            piece = red
            position = null
        turn_11:
            piece = red
            position = null
        turn_2(bind_1: Position):
            board = { :empty }
            piece = red
            position = null
        turn_5(bind_1: Position):
            board = { :empty }
            piece = red
            position = null"
    );

    test!(
        binding_loop3,
        "type Alpha = {a1,b1,c1,d1};
        var alpha: Alpha = d1;
        begin, 1: ;
        1, 2(bind_1: Alpha): bind_1 = c1;
        2(bind_1: Alpha), 3: alpha = bind_1;
        3, 4: $ bind_2;
        4, 1: ;
        3, end: ;",
        "1:
        2(bind_1: Alpha):
        3:
        4:
        begin:
            alpha = d1
        end:"
    );

    test!(
        binding_loop4,
        "type Alpha = {a1,b1,c1,d1};
        var bind_1: Alpha = a1;
        var alpha: Alpha = d1;
        begin, 1: ;
        1, 2(bind_1: Alpha): bind_1 = c1;
        2(bind_1: Alpha), 3: alpha = bind_1;
        3, 4: $ bind_2;
        4, 1: ;
        3, end: ;",
        "1:
        2(bind_1: Alpha):
        3:
        4:
        begin:
            alpha = d1
            bind_1 = a1
        end:"
    );

    test!(
        comparison1,
        "type A = {a,b,c};
        var x: A = a;
        begin, end: c == x;",
        "begin:
            x = a
        end:
            x = a"
    );

    test!(
        comparison2,
        "type A = {a,b,c};
        var x: A = a;
        var y: A = b;
        begin, end: x == y;",
        "begin:
            x = a
            y = b
        end:
            x = a
            y = b"
    );

    test!(
        comparison3,
        "type A = {a,b,c};
        var x: A = a;
        var y: A = b;
        begin, end: x != y;",
        "begin:
            x = a
            y = b
        end:
            x = a
            y = b"
    );

    test!(
        comparison4,
        "type A = {a,b,c};
        var x: A = a;
        var y: A -> A = { b: a, :b };
        begin, a1: x = c;
        a1, c: ;
        begin, c: ;
        c, end: x == y[a];",
        "a1:
            x = c
            y = { b: a, :b }
        begin:
            x = a
            y = { b: a, :b }
        c:
            y = { b: a, :b }
        end:
            x = b
            y = { b: a, :b }"
    );

    test!(
        comparison5,
        "type A = {a,b,c};
        var x: A = a;
        var y: A = b;
        begin, a(bind_1: A): x = bind_1;
        a(bind_1: A), end: x == bind_1;",
        "a(bind_1: A):
            y = b
        begin:
            x = a
            y = b
        end:
            y = b"
    );

    test!(
        comparison6,
        "type A = {a,b,c};
        var x: A = a;
        begin, a1: x = c;
        begin, b1: ;
        a1, b1: ;
        b1, end: x == b;",
        "a1:
            x = c
        b1:
        begin:
            x = a
        end:
            x = b"
    );
}
