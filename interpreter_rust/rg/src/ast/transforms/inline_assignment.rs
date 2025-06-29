use crate::ast::analyses::{Analysis, ReachingDefinitions};
use crate::ast::{Edge, Error, Expression, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::Arc;

type Id = Arc<str>;
type ToInline = (Id, Arc<Expression<Id>>, BTreeSet<Arc<Edge<Id>>>);

impl Game<Id> {
    /// For each assignment to a variable `x = expr` :
    ///   - collect references to `x`, until it's reassigned
    ///   - check if `x` can be inlined
    ///   - if yes, replace `x` with `expr` on every usage, and change assignment to skip
    ///
    /// Assignment `x = expr` can be inlined, if:
    /// 1. `expr` contains no bindings (TODO: This can be improved if all usages of `x` share this binding)
    /// 2. On every usage of `x` until its reassigned:
    ///   - the only reaching definition of `x` is from that assignment
    ///   - all variables in `expr` have the same values as in the assignment
    pub fn inline_assignment(&mut self) -> Result<(), Error<Id>> {
        let (to_inline, to_skip) = self.collect_to_inline();
        for edge in &mut self.edges {
            if to_skip.contains(edge) {
                Arc::make_mut(edge).skip();
            } else {
                let inlines: Vec<_> = to_inline
                    .iter()
                    .filter(|(_, _, usages)| usages.contains(edge))
                    .collect();
                for (to_replace, new_expr, _) in inlines {
                    Arc::make_mut(edge).label = edge
                        .label
                        .substitute_variable_readonly(to_replace, new_expr);
                }
            }
        }

        Ok(())
    }

    fn collect_to_inline(&self) -> (BTreeSet<ToInline>, BTreeSet<Arc<Edge<Id>>>) {
        let reaching_definitions = self.analyse(ReachingDefinitions);
        let next_edges = self.next_edges();
        let check_reachability = self.make_check_reachability(false);
        let mut to_inline = BTreeSet::new();
        let mut to_skip = BTreeSet::new();
        let mut modified_edges = BTreeSet::new();

        for edge in &self.edges {
            // Don't inline AssignmentAny
            if let Some((identifier, rhs)) = edge.label.as_assignment() {
                if edge.label.is_player_assignment()
                    || modified_edges.contains(edge)
                    || (edge.label.is_goals_assignment()
                        && check_reachability(&edge.rhs, &Node::new(Id::from("end")))
                            .is_reachable())
                {
                    continue;
                }

                let Some(current_definitions) = reaching_definitions.get(&edge.lhs) else {
                    continue;
                };

                let vars_in_rhs =
                    rhs.map_or_else(|_| BTreeSet::new(), |expr| expr.used_variables());
                let defs_on_assignment =
                    used_definitions(current_definitions, &vars_in_rhs, identifier);
                let Some(usages) = maybe_inline_assignment(
                    &next_edges,
                    &reaching_definitions,
                    edge,
                    identifier,
                    &defs_on_assignment,
                    &vars_in_rhs,
                ) else {
                    continue;
                };
                let usage_already_modified =
                    usages.iter().any(|usage| modified_edges.contains(usage));
                if usage_already_modified {
                    continue;
                }
                match rhs {
                    _ if (edge.label.is_map_assignment()) && !usages.is_empty() => {}
                    Err(_) if !usages.is_empty() => {}
                    Err(_) => {
                        modified_edges.insert(edge.clone());
                        to_skip.insert((*edge).clone());
                    }
                    Ok(rhs) => {
                        modified_edges.insert(edge.clone());
                        modified_edges.extend(usages.iter().cloned());
                        to_inline.insert((
                            (*identifier).clone(),
                            self.new_rhs(identifier, rhs),
                            usages,
                        ));
                        to_skip.insert((*edge).clone());
                    }
                }
            }
        }
        (to_inline, to_skip)
    }

    fn new_rhs(&self, variable: &Id, expr: &Arc<Expression<Id>>) -> Arc<Expression<Id>> {
        if let Expression::Reference { identifier } = expr.as_ref() {
            if self.is_symbol(identifier) {
                return self
                    .infer_or_none(variable)
                    .filter(|t| t.is_reference())
                    .map_or_else(
                        || expr.clone(),
                        |t| Arc::new(Expression::new_cast(t.clone(), expr.clone())),
                    );
            }
        }
        expr.clone()
    }
}

fn maybe_inline_assignment(
    next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    reaching_definitions: &BTreeMap<Node<Id>, <ReachingDefinitions as Analysis>::Domain>,
    def_edge: &Edge<Id>,
    id: &Id,
    defs_on_assignment: &BTreeMap<&Id, Option<&Arc<Edge<Id>>>>,
    vars_in_definition: &BTreeSet<&Id>,
) -> Option<BTreeSet<Arc<Edge<Id>>>> {
    let mut queue = vec![&def_edge.rhs];
    let mut seen = BTreeSet::new();
    let mut to_inline = BTreeSet::new();
    while let Some(lhs) = queue.pop() {
        if !seen.insert(lhs) {
            continue;
        }

        let Some(edges) = next_edges.get(&lhs) else {
            continue;
        };

        for edge in edges {
            let vars_in_label = edge.label.used_variables();
            if vars_in_label.contains(id) {
                let defs_on_usage = reaching_definitions.get(lhs).unwrap();
                let defs_on_usage = used_definitions(defs_on_usage, vars_in_definition, id);
                if !can_replace_usage(id, def_edge, defs_on_assignment, &defs_on_usage) {
                    return None;
                }

                to_inline.insert((*edge).clone());
            }

            if !is_reassigned(&edge.label, id) {
                if !seen.contains(&edge.rhs) {
                    queue.push(&edge.rhs);
                }

                if let Label::Reachability { lhs, .. } = &edge.label {
                    if !seen.contains(lhs) {
                        queue.push(lhs);
                    }
                }
            }
        }
    }

    Some(to_inline)
}

fn used_definitions<'a>(
    defs: &'a <ReachingDefinitions as Analysis>::Domain,
    variables: &'a BTreeSet<&Id>,
    identifier: &'a Id,
) -> BTreeMap<&'a Id, Option<&'a Arc<Edge<Id>>>> {
    variables
        .iter()
        .chain(iter::once(&identifier))
        .map(|var| (*var, defs.get(*var).and_then(|def| def.as_all())))
        .collect()
}

fn can_replace_usage(
    to_replace: &Id,
    def_edge: &Edge<Id>,
    defs_on_assignment: &BTreeMap<&Id, Option<&Arc<Edge<Id>>>>,
    defs_on_usage: &BTreeMap<&Id, Option<&Arc<Edge<Id>>>>,
) -> bool {
    defs_on_usage
        .get(to_replace)
        .is_some_and(|def| def.is_some_and(|def| def.as_ref() == def_edge))
        && defs_on_assignment.iter().all(|(var, on_def)| {
            *var == to_replace
                || defs_on_usage
                    .get(var)
                    .is_some_and(|on_use| on_def == on_use)
        })
}

fn is_reassigned(label: &Label<Id>, id: &Id) -> bool {
    matches!(label.as_var_assignment(), Some(identifier) if identifier == id)
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        inline_assignment,
        small,
        "type A = { y };
        var x: A = y;
        var d: A = y;
        begin, t2: x = y;
        t2, t3: z = d;
        t3, t5: d = x;
        t5, t6: a2 = z;
        t6, end: a2 == x;",
        "type A = { y };
        var x: A = y;
        var d: A = y;
        begin, t2: ;
        t2, t3: z = d;
        t3, t5: d = A(y);
        t5, t6: a2 = z;
        t6, end: a2 == A(y);"
    );

    test_transform!(
        inline_assignment,
        only_skip,
        "type A = { z };
        var x: A = z;
        var y: A = z;
        begin, t1: x = y[z];
        t1, t2: y = z;
        t2, end: y == z;",
        "type A = { z };
        var x: A = z;
        var y: A = z;
        begin, t1: ;
        t1, t2: ;
        t2, end: A(z) == z;"
    );

    test_transform!(
        inline_assignment,
        in_lhs,
        "type A = { y };
        var x: A = y;
        begin, t1: x = y[z];
        t1, t2: a[x] = x;
        t2, end: x = z;",
        "type A = { y };
        var x: A = y;
        begin, t1: ;
        t1, t2: a[y[z]] = y[z];
        t2, end: ;"
    );

    test_transform!(
        inline_assignment,
        double_assignment,
        "begin, t1: x = y[z];
        t1, t2: x == z;
        t2, t3: x = z[y];
        t3, end: x == y;",
        "begin, t1: ;
        t1, t2: y[z] == z;
        t2, t3: ;
        t3, end: z[y] == y;"
    );

    test_transform!(
        inline_assignment,
        assign_any_no_usages,
        "begin, t1: x = Foo(*);
        t1, end: y == z;",
        "begin, t1: ;
        t1, end: y == z;"
    );

    test_transform!(
        inline_assignment,
        assign_any_usages,
        "begin, t1: x = Foo(*);
        t1, end: x == z;"
    );

    test_transform!(
        inline_assignment,
        assign_map_no_usages,
        "begin, t1: ;
        t1, end: y == z;"
    );

    test_transform!(
        inline_assignment,
        assign_map_usages,
        "begin, t1: x[y] = 1;
        t1, end: x[1] == z;"
    );

    test_transform!(
        inline_assignment,
        assign_any_map_no_usages,
        "begin, t1: ;
        t1, end: y == z;"
    );

    test_transform!(
        inline_assignment,
        assign_any_map_usages,
        "begin, t1: x[y] = Foo(*);
        t1, end: x[1] == z;"
    );

    test_transform!(
        inline_assignment,
        reachability,
        "begin, t1: x = y[z];
        t1, t2: ? e1 -> e2;
        t2, end: z = x;
        e1, e2: y = x;",
        "begin, t1: ;
        t1, t2: ? e1 -> e2;
        t2, end: z = y[z];
        e1, e2: y = y[z];"
    );

    test_transform!(
        inline_assignment,
        reassigned_var,
        "begin, t1: y[z] = z; 
        t1, t2: x = y[z];
        t2, t3: y[z] = z;
        t3, end: y == x;"
    );

    test_transform!(
        inline_assignment,
        generator,
        "begin, t: x = Foo(*);
        t, t1: ;
        t1, end: z == x;"
    );

    test_transform!(
        inline_assignment,
        skip_map_assignment,
        "type A = { 1 };
        var x: A -> A = { :1 };
        begin, t1: x[y] = z;
        t1, end: ;",
        "type A = { 1 };
        var x: A -> A = { :1 };
        begin, t1: ;
        t1, end: ;"
    );

    test_transform!(
        inline_assignment,
        dont_inline_map_assignment,
        "begin, t1: x[y] = z;
        t1, end: x[z] == 2;"
    );

    test_transform!(
        inline_assignment,
        skip_goals_assignment,
        "type A = { 1 };
        var goals: A = 1;
        begin, t1: ? a -> b;
        a, b: goals[x] = y;
        t1, end: ;",
        "type A = { 1 };
        var goals: A = 1;
        begin, t1: ? a -> b;
        a, b: ;
        t1, end: ;"
    );

    test_transform!(
        inline_assignment,
        dont_inline_goals_assignment,
        "begin, t1: goals[x] = y;
        t1, t2: x == y;
        t2, end: x == y;"
    );

    test_transform!(
        inline_assignment,
        dont_inline_fork,
        "begin, a2: y = 3;
        a2, a3: ;
        a2, a4: y = 1;
        a3, a5: ;
        a4, a5: ;
        a5, end: y == A(3);"
    );

    test_transform!(
        inline_assignment,
        dont_inline_loop,
        "type A = {1,2,3};
        const cord: A -> A = {:1};
        var y: A = 2;
        begin, a2: y = 3;
        a2, a3: ;
        a3, a4: y = cord[y];
        a4, a2: ;
        a4, end: y == A(3);"
    );

    test_transform!(
        inline_assignment,
        inline_loop,
        "type A = {1,2,3};
        const cord: A -> A = {:1};
        var y: A = 2;
        begin, a2: y = 3;
        a2, a3: ;
        a3, a4: y = cord[y];
        a4, a2: y = 2;
        a4, end: y == A(3);",
        "type A = {1,2,3};
        const cord: A -> A = {:1};
        var y: A = 2;
        begin, a2: y = 3;
        a2, a3: ;
        a3, a4: ;
        a4, a2: y = 2;
        a4, end: cord[y] == A(3);"
    );

    test_transform!(
        inline_assignment,
        knightthrough_small,
        "begin, rules_begin: ;
        rules_begin, move_10: position = direction[me][position];
        move_10, turn_6: position = down[position];
        turn_6, turn_9: position != null;
        turn_9, end: ;
        turn_9, rules_begin: ;",
        "begin, rules_begin: ;
        rules_begin, move_10: ;
        move_10, turn_6: position = down[direction[me][position]];
        turn_6, turn_9: position != null;
        turn_9, end: ;
        turn_9, rules_begin: ;"
    );

    test_transform!(
        inline_assignment,
        ttt_rbg,
        "begin, end: ? 32 -> 33;
        100, 102: coord != null;
        101, 105: coord = direction[right][coord];
        102, 101: board[coord] == x;
        105, 107: coord != null;
        107, 33: board[coord] == x;
        32, 100: coord = direction[right][coord];",
        "begin, end: ? 32 -> 33;
        100, 102: coord != null;
        101, 105: ;
        102, 101: board[coord] == x;
        105, 107: direction[right][coord] != null;
        107, 33: board[direction[right][coord]] == x;
        32, 100: coord = direction[right][coord];"
    );

    test_transform!(
        inline_assignment,
        tag_variable,
        "type A = { y };
        begin, t1: x = y;
        t1, t2: $$ x;
        t2, end: x == y;"
    );

    test_transform!(
        inline_assignment,
        assign_before_loop,
        "begin, a: x = 2;
        a, b: ;
        b, c: ;
        c, b: ;
        c, d: ;
        a, d: ;
        d, end: x == 1;",
        "begin, a: ;
        a, b: ;
        b, c: ;
        c, b: ;
        c, d: ;
        a, d: ;
        d, end: 2 == 1;"
    );

    test_transform!(
        inline_assignment,
        assign_in_fork,
        "begin, a: ;
        a, b: x = 2;
        b, c: ;
        a, c: ;
        c, end: x == 1;"
    );

    test_transform!(
        inline_assignment,
        assign_in_fork_2,
        "begin, a: ;
        a, b1: x = 2;
        b1, c: ;
        a, b2: x = 2;
        b2, c: ;
        a, c: ;
        c, end: x == 1;"
    );

    // TODO: Add tests with forks
}
