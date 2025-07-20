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
    type Domain = BTreeMap<Expression<Id>, Arc<Value<Id>>>;

    fn bot(&self) -> Self::Domain {
        Self::Domain::default()
    }

    fn extreme(&self, program: &Game<Id>, _ctx: &Self::Context) -> Self::Domain {
        program
            .variables
            .iter()
            .map(|v| {
                (
                    Expression::new(v.identifier.clone()),
                    v.default_value.clone(),
                )
            })
            .collect()
    }

    fn join(&self, mut a: Self::Domain, b: Self::Domain, _ctx: &Self::Context) -> Self::Domain {
        // Keep only keys present in both maps with the same value.
        a.retain(|key, value| b.get(key) == Some(value));
        a
    }

    fn get_context(&self, program: &Game<Id>) -> Self::Context {
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
        &self,
        mut input: Self::Domain,
        edge: &Arc<Edge<Id>>,
        ctx: &Self::Context,
    ) -> Self::Domain {
        if let Some((expr, value)) = as_constant_assignment(edge, &input, ctx)
            .or_else(|| as_constant_comparison(edge, &input, ctx))
        {
            input.insert(expr.clone(), value);
        } else if let Some(identifier) = &edge.label.as_var_assignment() {
            input.retain(|expr, _| !expr.has_variable(identifier));
        }

        input
    }

    fn with_reachability(&self) -> bool {
        true
    }
}

fn as_constant_assignment<'a>(
    edge: &'a Edge<Id>,
    knowledge: &<ConstantsAnalysis as Analysis>::Domain,
    ctx: &Context,
) -> Option<(&'a Expression<Id>, Arc<Value<Id>>)> {
    match &edge.label {
        Label::Assignment { lhs, rhs } => Some((lhs, evaluate_constant(rhs, knowledge, ctx)?)),
        _ => None,
    }
}

fn as_constant_comparison<'a>(
    edge: &'a Edge<Id>,
    knowledge: &<ConstantsAnalysis as Analysis>::Domain,
    ctx: &Context,
) -> Option<(&'a Expression<Id>, Arc<Value<Id>>)> {
    if let Label::Comparison {
        lhs,
        rhs,
        negated: false,
    } = &edge.label
    {
        let lhs = lhs.uncast();
        let rhs = rhs.uncast();

        let lhs_value = evaluate_constant(lhs, knowledge, ctx);
        let rhs_value = evaluate_constant(rhs, knowledge, ctx);

        return match (lhs_value, rhs_value) {
            (None, Some(rhs_value)) => Some((lhs, rhs_value)),
            (Some(lhs_value), None) => Some((rhs, lhs_value)),
            _ => None,
        };
    }

    None
}

fn evaluate_constant(
    expr: &Expression<Id>,
    knowledge: &<ConstantsAnalysis as Analysis>::Domain,
    ctx: &Context,
) -> Option<Arc<Value<Id>>> {
    match expr {
        _ if knowledge.contains_key(expr) => knowledge.get(expr).cloned(),
        Expression::Access { lhs, rhs, .. } => {
            let lhs = evaluate_constant(lhs, knowledge, ctx)?;
            let rhs = evaluate_constant(rhs, knowledge, ctx)?;
            dereference_constant(&rhs, ctx)
                .to_identifier()
                .and_then(|identifier| {
                    dereference_constant(&lhs, ctx)
                        .get_entry(identifier)
                        .map(|entry| Arc::new(entry.clone()))
                })
        }
        Expression::Cast { rhs, .. } => evaluate_constant(rhs, knowledge, ctx),
        Expression::Reference { identifier } if ctx.is_variable(identifier) => None,
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
    use super::{Analysis, ConstantsAnalysis};
    use crate::ast::{Game, Node};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn format_analysis(
        analysis: BTreeMap<Node<Arc<str>>, <ConstantsAnalysis as Analysis>::Domain>,
    ) -> String {
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
                Game::test_analysis(
                    $source,
                    $expect,
                    Box::new(|_| ConstantsAnalysis),
                    Box::new(format_analysis),
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
