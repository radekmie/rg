use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use crate::ast::{Edge, Error, Expression, Game, Label, Node};

impl Game<Arc<str>> {
    pub fn inline_assignment(&mut self) -> Result<(), Error<Arc<str>>> {
        let reaching_definitions = self.reaching_definitions();
        let next_edges = self.next_edges();
        let mut to_inline = BTreeSet::new();
        let mut modified_edges = BTreeSet::new();
        for edge in &self.edges {
            if let Label::Assignment { lhs, rhs } = &edge.label {
                if let Expression::Reference { identifier } = lhs.as_ref() {
                    if identifier.as_ref() != "player" {
                        if modified_edges.contains(edge) {
                            continue;
                        }
                        if let Some(current_definitions) = reaching_definitions.get(&edge.lhs) {
                            let vars_in_rhs = vars_in_expression(rhs);
                            let defs_on_assignment: BTreeMap<_, _> =
                                group_definitions(current_definitions)
                                    .into_iter()
                                    .filter(|(var, _)| vars_in_rhs.contains(var))
                                    .collect();
                            if let Some(usages) = can_be_inlined(
                                &next_edges,
                                &reaching_definitions,
                                edge,
                                identifier,
                                &defs_on_assignment,
                            ) {
                                if usages.is_empty()
                                    || !edge
                                        .bindings()
                                        .iter()
                                        .any(|binding| vars_in_rhs.contains(binding.0))
                                {
                                    modified_edges.extend(usages.iter().cloned());
                                    to_inline.insert((
                                        (*identifier).clone(),
                                        (*rhs).clone(),
                                        (*edge).clone(),
                                        usages,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (to_replace, new_expr, to_skip, usages) in to_inline.iter() {
            for edge in &mut self.edges {
                if edge == to_skip {
                    edge.skip();
                } else if usages.contains(edge) {
                    edge.label = substiture_variable(&edge.label, to_replace, new_expr);
                }
            }
        }

        Ok(())
    }
}

fn substiture_variable(
    label: &Label<Arc<str>>,
    to_replace: &Arc<str>,
    new_expr: &Expression<Arc<str>>,
) -> Label<Arc<str>> {
    if let Label::Assignment { lhs, rhs } = label {
        let lhs = match lhs.as_ref() {
            Expression::Reference { identifier } if identifier == to_replace => (*lhs).clone(),
            _ => Arc::new(lhs.substitute_variable(to_replace, new_expr)),
        };
        let rhs = Arc::new(rhs.substitute_variable(to_replace, new_expr));
        Label::Assignment { lhs, rhs }
    } else {
        label.substitute_variable(to_replace, new_expr)
    }
}

fn can_be_inlined(
    next_edges: &BTreeMap<&Node<Arc<str>>, BTreeSet<&Edge<Arc<str>>>>,
    reaching_definitions: &BTreeMap<Node<Arc<str>>, BTreeSet<(Arc<str>, Option<Edge<Arc<str>>>)>>,
    def_edge: &Edge<Arc<str>>,
    id: &Arc<str>,
    defs_on_assignment: &BTreeMap<Arc<str>, BTreeSet<Option<Edge<Arc<str>>>>>,
) -> Option<BTreeSet<Edge<Arc<str>>>> {
    let mut queue = vec![&def_edge.rhs];
    let mut seen = BTreeSet::new();
    let mut to_inline = BTreeSet::new();
    while let Some(lhs) = queue.pop() {
        let maybe_edges = next_edges.get(&lhs);
        if seen.insert(lhs) {
            if let Some(edges) = maybe_edges {
                for edge in edges {
                    let vars_in_label = vars_in_label(&edge.label);
                    if vars_in_label.contains(id) {
                        let defs_on_usage = reaching_definitions.get(lhs).unwrap();
                        let defs_on_usage: BTreeMap<_, _> = group_definitions(defs_on_usage)
                            .into_iter()
                            .filter(|(var, _)| vars_in_label.contains(var))
                            .collect();
                        if !can_replace_usage(id, def_edge, defs_on_assignment, &defs_on_usage) {
                            return None;
                        }
                        dbg!("inserting");
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
        }
    }
    Some(to_inline)
}

fn group_definitions(
    defs: &BTreeSet<(Arc<str>, Option<Edge<Arc<str>>>)>,
) -> BTreeMap<Arc<str>, BTreeSet<Option<Edge<Arc<str>>>>> {
    let mut grouped = BTreeMap::new();
    for (var, edge) in defs {
        grouped
            .entry(var.clone())
            .or_insert_with(BTreeSet::new)
            .insert((*edge).clone());
    }
    grouped
}

fn can_replace_usage(
    id: &Arc<str>,
    def_edge: &Edge<Arc<str>>,
    defs_on_assignment: &BTreeMap<Arc<str>, BTreeSet<Option<Edge<Arc<str>>>>>,
    defs_on_usage: &BTreeMap<Arc<str>, BTreeSet<Option<Edge<Arc<str>>>>>,
) -> bool {
    defs_on_usage
        .get(id)
        .expect(id)
        .iter()
        .all(|def| def.as_ref().is_some_and(|def| def == def_edge))
        && defs_on_assignment.iter().all(|(var, on_def)| {
            if var == id {
                true
            } else {
                defs_on_usage
                    .get(var)
                    .is_some_and(|on_use| on_def == on_use)
            }
        })
}

fn is_reassigned(label: &Label<Arc<str>>, id: &Arc<str>) -> bool {
    match label {
        Label::Assignment { lhs, .. } => {
            if let Expression::Reference { identifier } = lhs.as_ref() {
                identifier == id
            } else {
                false
            }
        }
        _ => false,
    }
}

fn vars_in_label(label: &Label<Arc<str>>) -> BTreeSet<Arc<str>> {
    match label {
        Label::Assignment { lhs, rhs } => {
            let mut vars = vars_in_expression(lhs);
            vars.extend(vars_in_expression(rhs));
            vars
        }
        Label::Comparison { lhs, rhs, .. } => {
            let mut vars = vars_in_expression(lhs);
            vars.extend(vars_in_expression(rhs));
            vars
        }
        _ => BTreeSet::new(),
    }
}

fn vars_in_expression(expression: &Expression<Arc<str>>) -> BTreeSet<Arc<str>> {
    let mut vars = BTreeSet::new();
    match expression {
        Expression::Access { lhs, rhs, .. } => {
            vars.extend(vars_in_expression(lhs));
            vars.extend(vars_in_expression(rhs));
        }
        Expression::Cast { rhs, .. } => vars.extend(vars_in_expression(rhs)),
        Expression::Reference { identifier } => {
            vars.insert(identifier.clone());
        }
    }
    vars
}

#[cfg(test)]
mod test {
    use crate::ast::Game;
    use crate::parsing::parser::parse_with_errors;
    use map_id::MapId;
    use std::sync::Arc;

    fn parse(input: &str) -> Game<Arc<str>> {
        let (game, errors) = parse_with_errors(input);
        assert!(errors.is_empty(), "Parse errors: {errors:?}");
        game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
    }

    macro_rules! test {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let mut actual = parse($actual);
                let expect = parse($expect);
                actual.inline_assignment().unwrap();

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    test!(
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
}
