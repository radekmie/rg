use super::Analysis;
use crate::ast::{Edge, Expression, Game, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type ConstantValue = Arc<Value<Id>>;

#[derive(Default, PartialEq, Eq)]
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

    fn extreme(program: &Game<Id>, ctx: &Context) -> Self::Domain {
        program
            .variables
            .iter()
            .map(|v| {
                (
                    v.identifier.clone(),
                    dereference_constant(&v.default_value, ctx),
                )
            })
            .collect()
    }

    fn join(mut a: Self::Domain, b: Self::Domain) -> Self::Domain {
        // Join the two maps, don't remove the key only if it is present in both maps with the same value
        for (key, value) in &b {
            if let Some(a_value) = a.get(key) {
                if a_value != value {
                    a.remove(key);
                }
            }
        }

        for key in a.clone().keys() {
            if !b.contains_key(key) {
                a.remove(key);
            }
        }
        a
    }

    fn kill(input: Self::Domain, edge: &Edge<Id>, _ctx: &Self::Context) -> Self::Domain {
        match &edge.label.as_var_assignment() {
            Some((identifier, _)) => input
                .into_iter()
                .filter(|(id, _)| id != *identifier)
                .collect(),
            _ => input,
        }
    }

    fn gen(mut input: Self::Domain, edge: &Edge<Id>, ctx: &Self::Context) -> Self::Domain {
        if let Some((identifier, value)) = as_constant_assignment(edge, &input, ctx) {
            input.insert(identifier, value);
        }
        input
    }

    fn get_context(program: &Game<Id>) -> Self::Context {
        let mut ctx = Self::Context::default();
        for constant in &program.constants {
            let value = dereference_constant(&constant.value, &ctx);
            ctx.constants.insert(constant.identifier.clone(), value);
        }
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
        ctx.variables = variables;
        ctx
    }

    // We can't use the default implementation, because it doesn't work for cases like:
    // x = 1;
    // x = y[x]; <- here `kill` removes `x` from `input` before `gen`, so `x` in lhs is not recognised as a constant
    fn transfer(mut input: Self::Domain, edge: &Edge<Id>, ctx: &Self::Context) -> Self::Domain {
        if let Some((identifier, value)) = as_constant_assignment(edge, &input, ctx) {
            input.insert(identifier, value);
            input
        } else {
            match &edge.label.as_var_assignment() {
                Some((identifier, _)) => input
                    .into_iter()
                    .filter(|(id, _)| id != *identifier)
                    .collect(),
                _ => input,
            }
        }
    }
}

fn as_constant_assignment(
    edge: &Edge<Id>,
    knowledge: &BTreeMap<Id, ConstantValue>,
    ctx: &Context,
) -> Option<(Id, ConstantValue)> {
    let (id, expr) = edge.label.as_var_assignment()?;
    if edge.label.is_map_assignment() {
        return None;
    }
    let value = evaluate_constant(expr, knowledge, ctx, edge)?;
    Some((id.clone(), value))
}

fn evaluate_constant(
    expr: &Expression<Id>,
    knowledge: &BTreeMap<Id, ConstantValue>,
    ctx: &Context,
    edge: &Edge<Id>,
) -> Option<ConstantValue> {
    match expr {
        Expression::Access { lhs, rhs, .. } => {
            let lhs = evaluate_constant(lhs, knowledge, ctx, edge)?;
            let rhs = evaluate_constant(rhs, knowledge, ctx, edge)?;
            rhs.to_identifier().and_then(|identifier| {
                lhs.get_entry(identifier)
                    .map(|entry| Arc::new(entry.clone()))
            })
        }
        Expression::Cast { rhs, .. } => evaluate_constant(rhs, knowledge, ctx, edge),
        Expression::Reference { identifier } if edge.has_binding(identifier) => None,
        Expression::Reference { identifier } if ctx.variables.contains(identifier) => {
            knowledge.get(identifier).cloned()
        }
        Expression::Reference { identifier } => ctx
            .constants
            .get(identifier)
            .cloned()
            // .map(|value| dereference_constant(value.clone(), ctx))
            .or(Some(Arc::new(Value::new(identifier.clone())))),
    }
}

fn dereference_constant(value: &ConstantValue, ctx: &Context) -> ConstantValue {
    match value.as_ref() {
        Value::Element { identifier } if ctx.constants.contains_key(identifier) => {
            ctx.constants[identifier].clone()
        }
        Value::Map { entries, span } => {
            let mut entries = entries.clone();
            for entry in &mut entries {
                entry.value = dereference_constant(&entry.value, ctx);
            }
            Arc::new(Value::Map {
                entries,
                span: *span,
            })
        }
        Value::Element { .. } => value.clone(),
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
            x = a
            y = a
            z = { b: a, :c }
        begin:
            x = a
            y = a
            z = { b: a, :c }
        end:
            x = c
            y = a
            z = { b: a, :c }"
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
        const_dependency,
        "type A = {a,b,c};
        const cst1: A = a;
        const cst2: A = cst1;
        const cst3: A -> A -> A = { b: { :cst2 }, :{ :cst1 } };
        var x: A -> A -> A = cst3;",
        "begin:
            x = { b: { :a }, :{ :a } }"
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
            x = c
            y = c
        a2:
            x = b
            y = c
        begin:
            x = b
            y = c"
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
            y = { b: c, :b }
        end:
            x = c
            y = { b: c, :b }"
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

    // TODO: Add tests for binding shadowing
}
