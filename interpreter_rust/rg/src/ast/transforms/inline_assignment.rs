use crate::ast::analysis::{Analysis, ReachingDefinitions};
use crate::ast::{Edge, Error, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;

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
        let reaching_definitions = self.analyse::<ReachingDefinitions>();
        let next_edges = self.next_edges();
        let mut to_inline = BTreeSet::new();
        let mut modified_edges = BTreeSet::new();
        for edge in &self.edges {
            if let Some((identifier, rhs)) = edge.label.as_var_assignment() {
                if edge.label.is_player_assignment() || modified_edges.contains(edge) {
                    continue;
                }

                let Some(current_definitions) = reaching_definitions.get(&edge.lhs) else {
                    continue;
                };

                let vars_in_rhs = rhs.used_variables();
                let defs_on_assignment = used_definitions(current_definitions, &vars_in_rhs);
                let Some(usages) = maybe_inline_assignment(
                    &next_edges,
                    &reaching_definitions,
                    edge,
                    identifier,
                    &defs_on_assignment,
                ) else {
                    continue;
                };
                let uses_binding = edge
                    .bindings()
                    .iter()
                    .any(|binding| vars_in_rhs.contains(binding.0));
                if (uses_binding || edge.label.is_map_assignment()) && !usages.is_empty() {
                    continue;
                }

                modified_edges.extend(usages.iter().cloned());
                to_inline.insert((
                    (*identifier).clone(),
                    (*rhs).clone(),
                    (*edge).clone(),
                    usages,
                ));
            }
        }
        for (to_replace, new_expr, to_skip, usages) in to_inline {
            for edge in &mut self.edges {
                if *edge == to_skip {
                    edge.skip();
                } else if usages.contains(edge) {
                    edge.label = edge
                        .label
                        .substitute_variable_readonly(&to_replace, &new_expr);
                }
            }
        }

        Ok(())
    }
}

fn maybe_inline_assignment(
    next_edges: &BTreeMap<&Node<Id>, BTreeSet<&Edge<Id>>>,
    reaching_definitions: &BTreeMap<Node<Id>, <ReachingDefinitions as Analysis>::Domain>,
    def_edge: &Edge<Id>,
    id: &Id,
    defs_on_assignment: &BTreeMap<Id, BTreeSet<Option<Edge<Id>>>>,
) -> Option<BTreeSet<Edge<Id>>> {
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
                let defs_on_usage = used_definitions(defs_on_usage, &vars_in_label);
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

fn used_definitions(
    defs: &<ReachingDefinitions as Analysis>::Domain,
    variables: &BTreeSet<&Id>,
) -> BTreeMap<Id, BTreeSet<Option<Edge<Id>>>> {
    defs.iter().filter(|(var, _)| variables.contains(var)).fold(
        BTreeMap::new(),
        |mut grouped_defs, (var, edge)| {
            grouped_defs
                .entry(var.clone())
                .or_default()
                .insert((*edge).clone());
            grouped_defs
        },
    )
}

fn can_replace_usage(
    to_replace: &Id,
    def_edge: &Edge<Id>,
    defs_on_assignment: &BTreeMap<Id, BTreeSet<Option<Edge<Id>>>>,
    defs_on_usage: &BTreeMap<Id, BTreeSet<Option<Edge<Id>>>>,
) -> bool {
    defs_on_usage.get(to_replace).is_some_and(|defs| {
        defs.iter()
            .all(|def| def.as_ref().is_some_and(|def| def == def_edge))
    }) && defs_on_assignment.iter().all(|(var, on_def)| {
        var == to_replace
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
        "begin, t2: x = y;
        t2, t3: z = d;
        t3, t5: d = x;
        t5, t6: a2 = z;
        t6, end: a2 == x;",
        "begin, t2: ;
        t2, t3: ;
        t3, t5: d = y;
        t5, t6: a2 = d;
        t6, end: a2 == y;"
    );

    test_transform!(
        inline_assignment,
        only_skip,
        "begin, t1: x = y[z];
        t1, t2: y = z;
        t2, end: y == z;",
        "begin, t1: ;
        t1, t2: ;
        t2, end: z == z;"
    );

    test_transform!(
        inline_assignment,
        in_lhs,
        "begin, t1: x = y[z];
        t1, t2: a[x] = x;
        t2, end: x = z;",
        "begin, t1: ;
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
        "begin, t1: x[y] = z;
        t1, end: ;",
        "begin, t1: ;
        t1, end: ;"
    );

    test_transform!(
        inline_assignment,
        dont_inline_map_assignment,
        "begin, t1: x[y] = z;
        t1, end: x[z] == 2;"
    );

    // TODO: Add tests with forks
}
