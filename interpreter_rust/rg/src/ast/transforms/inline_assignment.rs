use crate::ast::analyses::{Analysis, ReachingDefinitions};
use crate::ast::{Edge, Error, Expression, Game, Label, Node, Type};
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
        let reaching_definitions = self.analyse::<ReachingDefinitions>(true);
        let next_edges = self.next_edges();
        let is_reachable = self.make_is_reachable();
        let mut to_inline = BTreeSet::new();
        let mut to_skip = BTreeSet::new();
        let mut modified_edges = BTreeSet::new();

        for edge in &self.edges {
            if let Some((identifier, rhs)) = edge.label.as_var_assignment() {
                if edge.label.is_player_assignment()
                    || modified_edges.contains(edge)
                    || (edge.label.is_goals_assignment()
                        && is_reachable(&edge.rhs, &Node::new(Id::from("end"))))
                {
                    continue;
                }

                let Some(current_definitions) = reaching_definitions.get(&edge.lhs) else {
                    continue;
                };

                let vars_in_rhs = rhs.used_variables();
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
                let uses_binding = edge
                    .bindings()
                    .iter()
                    .any(|binding| vars_in_rhs.contains(binding.0));
                let usage_already_modified =
                    usages.iter().any(|usage| modified_edges.contains(usage));
                if usage_already_modified {
                    continue;
                }
                if (uses_binding || edge.label.is_map_assignment()) && !usages.is_empty() {
                    continue;
                }

                modified_edges.insert(edge.clone());
                modified_edges.extend(usages.iter().cloned());
                to_inline.insert((
                    (*identifier).clone(),
                    self.new_rhs(identifier, rhs, edge),
                    usages,
                ));
                to_skip.insert((*edge).clone());
            }
        }
        (to_inline, to_skip)
    }

    fn new_rhs(
        &self,
        variable: &Id,
        expr: &Arc<Expression<Id>>,
        edge: &Edge<Id>,
    ) -> Arc<Expression<Id>> {
        match expr.as_ref() {
            Expression::Reference { identifier } if self.is_symbol(identifier, edge) => {
                let type_ = self.resolve_variable(variable).unwrap().type_.clone();
                if let Type::TypeReference { .. } = type_.as_ref() {
                    Arc::new(Expression::new_cast(type_, expr.clone()))
                } else {
                    expr.clone()
                }
            }
            _ => expr.clone(),
        }
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
    let used: BTreeMap<_, _> = variables
        .iter()
        .chain(iter::once(&identifier))
        .map(|var| (*var, defs.get(*var)))
        .collect();

    used
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
    matches!(label.as_var_assignment(), Some((identifier, _)) if identifier == id)
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
        binding_no_usages,
        "begin, t1(z: Pos): x = y[z];
        t1, end: y == z;",
        "begin, t1(z: Pos): ;
        t1, end: y == z;"
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
        binding,
        "begin, t1(p: Pos): x = y[p];
        t1(p: Pos), t1: ;
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

    // TODO: Add tests with forks
}
