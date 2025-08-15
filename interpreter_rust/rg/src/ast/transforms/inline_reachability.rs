use super::{gen_fresh_node, max_node_id};
use crate::ast::analyses::ReachableNodes;
use crate::ast::{Edge, Error, Game, Label, Node};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;
type Subgraph = BTreeSet<Arc<Edge<Id>>>;

impl Game<Id> {
    pub fn inline_reachability(&mut self) -> Result<(), Error<Id>> {
        let begin = Node::new(Arc::from("begin"));
        let in_main_automaton = self.analyse(&ReachableNodes::new());
        for edge in self.edges.clone() {
            if let Label::Reachability {
                lhs, rhs, negated, ..
            } = &edge.label
            {
                let is_main_automaton = lhs == &begin
                    || in_main_automaton
                        .get(&edge.lhs)
                        .is_some_and(|reachable| *reachable);
                if let Some((subgraph, defined_vars)) =
                    self.find_rechability_paths(lhs, rhs, *negated, is_main_automaton)
                {
                    self.substitute_reachability(edge.clone(), subgraph, defined_vars);
                }
            }
        }
        Ok(())
    }

    /// Return a subautomaton of [edges] that:
    /// 1. contains [start] and [target]
    /// 2. for any node except [target] contains all outgoing nodes
    /// 3. contains no edges from [target]
    /// 4. Assignments are allowed only if the variable is reassigned before being used after inlining
    /// 5. If the reachability is negated, the path consists of one edge
    fn find_rechability_paths(
        &self,
        start: &Node<Id>,
        target: &Node<Id>,
        negated: bool,
        is_main_automaton: bool,
    ) -> Option<(Subgraph, Option<BTreeSet<Id>>)> {
        if negated {
            let edge = self
                .outgoing_edge(start)
                .filter(|edge| edge.rhs == *target)?;
            match &edge.label {
                // Do not inline negated assignments.
                Label::Assignment { .. } | Label::AssignmentAny { .. } => None,
                // Copy (and negate) the comparison or reachability.
                Label::Comparison { .. } | Label::Reachability { .. } => {
                    Some((BTreeSet::from([edge.clone()]), None))
                }
                // Do not inline tags into the main automaton.
                Label::Tag { .. } | Label::TagVariable { .. } => {
                    if is_main_automaton {
                        None
                    } else {
                        Some((BTreeSet::new(), None))
                    }
                }
                // Skips are always passable, so a negated reachability
                // should never pass them - the edge should be removed.
                Label::Skip { .. } => Some((BTreeSet::new(), None)),
            }
        } else {
            let negated_label = Label::Reachability {
                span: Span::none(),
                lhs: start.clone(),
                rhs: target.clone(),
                negated: true,
            };
            // Do not inline `? a -> b` if `! a -> b` exists and cannot be inlined
            if self.edges.iter().any(|edge| edge.label == negated_label)
                && self
                    .find_rechability_paths(start, target, true, is_main_automaton)
                    .is_none()
            {
                return None;
            }
            self.find_acceptable_paths(start, target, is_main_automaton)
        }
    }

    fn find_acceptable_paths(
        &self,
        start: &Node<Id>,
        target: &Node<Id>,
        is_main_automaton: bool,
    ) -> Option<(Subgraph, Option<BTreeSet<Id>>)> {
        let next_edges = self.next_edges();
        let mut defined_vars = BTreeSet::new();
        let mut previous = BTreeSet::new();
        let mut queue = vec![start];
        let mut result = BTreeSet::new();
        while let Some(lhs) = queue.pop() {
            previous.insert(lhs);
            if let Some(edges) = next_edges.get(&lhs) {
                if edges.len() != 1 {
                    return None;
                }
                let edge = edges.iter().next().unwrap();
                if is_main_automaton
                    && (edge.label.is_assignment_any()
                        || edge.label.is_tag()
                        || edge.label.is_tag_variable())
                {
                    return None;
                }
                if let Some(id) = edge.label.as_var_assignment() {
                    defined_vars.insert(id.clone());
                }
                result.insert((*edge).clone());
                if edge.rhs != *target && !previous.contains(&edge.rhs) {
                    queue.push(&edge.rhs);
                }
            }
        }
        let defined_vars = Some(defined_vars).filter(|set| !set.is_empty());
        Some((result, defined_vars))
    }

    fn substitute_reachability(
        &mut self,
        mut edge: Arc<Edge<Id>>,
        subgraph: Subgraph,
        defined_vars: Option<BTreeSet<Id>>,
    ) {
        if subgraph.is_empty() {
            self.remove_edge(&edge);
        } else if defined_vars
            .into_iter()
            .all(|defined_vars| self.check_used_variables(&edge.rhs, defined_vars))
        {
            if let Label::Reachability {
                lhs: start,
                rhs: target,
                negated,
                ..
            } = edge.label.clone()
            {
                let mut max_id = max_node_id(&self.nodes());

                self.remove_edge(&edge);
                let new_start = gen_fresh_node(&mut max_id);

                let mut mapping = BTreeMap::new();
                mapping.insert(start, new_start.clone());
                mapping.insert(target.clone(), edge.rhs.clone());

                Arc::make_mut(&mut edge).rhs = new_start;
                Arc::make_mut(&mut edge).skip();
                self.add_edge(edge);

                for mut edge in subgraph {
                    if let Some(lhs) = mapping.get(&edge.lhs) {
                        Arc::make_mut(&mut edge).lhs = lhs.clone();
                    } else {
                        let lhs = gen_fresh_node(&mut max_id);
                        mapping.insert(edge.lhs.clone(), lhs.clone());
                        Arc::make_mut(&mut edge).lhs = lhs;
                    }

                    if let Some(rhs) = mapping.get(&edge.rhs) {
                        Arc::make_mut(&mut edge).rhs = rhs.clone();
                    } else {
                        let rhs = gen_fresh_node(&mut max_id);
                        mapping.insert(edge.rhs.clone(), rhs.clone());
                        Arc::make_mut(&mut edge).rhs = rhs;
                    }
                    if negated {
                        Arc::make_mut(&mut edge).label.negate();
                    }
                    self.add_edge(edge);
                }
            }
        }
    }

    /// If the subgraph contains assignments,
    /// they should not be used in the rest of the graph before beeing reassigned.
    fn check_used_variables(&self, start: &Node<Id>, defined_vars: BTreeSet<Id>) -> bool {
        let next_edges = self.next_edges();
        let mut queue = vec![(start, BTreeSet::new(), defined_vars)];
        while let Some((lhs, mut previous, defined_vars)) = queue.pop() {
            previous.insert(lhs);
            if let Some(edges) = next_edges.get(&lhs) {
                for edge in edges {
                    let mut defined_vars = defined_vars.clone();
                    if previous.contains(&edge.rhs) {
                        continue;
                    }
                    let used_variables = edge.label.used_variables();
                    if defined_vars.iter().any(|id| used_variables.contains(id)) {
                        return false;
                    }
                    match edge.label.as_var_assignment() {
                        Some(id) if !edge.label.is_map_assignment() => {
                            defined_vars.remove(id);
                        }
                        _ => (),
                    }
                    if let Label::Reachability { lhs, .. } = &edge.label {
                        if !previous.contains(lhs) {
                            queue.push((lhs, previous.clone(), defined_vars.clone()));
                        }
                    }
                    queue.push((&edge.rhs, previous.clone(), defined_vars));
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        inline_reachability,
        basic1,
        "a, b: ? x -> y;
        x, y: 1 == 1;",
        "x, y: 1 == 1;
        a, 1: ;
        1, b: 1 == 1;"
    );

    test_transform!(
        inline_reachability,
        basic2,
        "a, b: ? x -> z;
        x, y: 1 == 1;
        y, z: 2 == 2;",
        "x, y: 1 == 1;
        y, z: 2 == 2;
        a, 1: ;
        1, 2: 1 == 1;
        2, b: 2 == 2;"
    );

    test_transform!(
        inline_reachability,
        basic3,
        "a, b: ? x -> z;
        x, y: ;
        y, z: 2 == 2;",
        "x, y: ;
        y, z: 2 == 2;
        a, 1: ;
        2, b: 2 == 2;
        1, 2: ;"
    );

    test_transform!(
        inline_reachability,
        basic4,
        "a, b: ? x -> z;
        x, y: 1 == 1;
        y, z: ;",
        "x, y: 1 == 1;
        y, z: ;
        a, 1: ;
        1, 2: 1 == 1;
        2, b: ;"
    );

    test_transform!(
        inline_reachability,
        basic5,
        "a, b: ? x -> z;
        x, y: 1 == 1;
        y, x: 2 == 2;
        y, z: 3 == 3;"
    );

    test_transform!(
        inline_reachability,
        exclusive_comparison,
        "x, y: ? a -> d;
        a, b: 1 == 1;
        a, c: 1 != 1;
        b, d: ;
        c, d: ;"
    );

    test_transform!(
        inline_reachability,
        non_exclusive_comparison,
        "type T = {1, 2};
        var v: T = 1;
        x, y: ? a -> d;
        a, b: v == 1;
        a, c: v != 2;
        b, d: ;
        c, d: ;"
    );

    test_transform!(
        inline_reachability,
        exclusive_reachability_step,
        "x, y: ? a -> d;
        a, b: ? e -> f;
        a, c: ! e -> f;
        b, d: ;
        c, d: ;
        e, f: ;",
        "x, y: ? a -> d;
        b, d: ;
        c, d: ;
        e, f: ;
        a, 1: ;
        1, b: ;"
    );

    test_transform!(
        inline_reachability,
        exclusive_reachability_step2,
        "b, d: ;
        c, d: ;
        e, f: ;
        x, 1: ;
        1, 2: ? e -> f;
        1, 3: ! e -> f;
        2, y: ;
        3, y: ;
        a, 4: ;
        4, b: ;",
        "b, d: ;
        c, d: ;
        e, f: ;
        x, 1: ;
        2, y: ;
        3, y: ;
        a, 4: ;
        4, b: ;
        1, 5: ;
        5, 2: ;"
    );

    test_transform!(
        inline_reachability,
        non_exclusive_reachability,
        "x, y: ? a -> d;
        a, b: ? e -> f;
        a, c: ! e -> g;
        b, d: ;
        c, d: ;
        e, f: ;
        e, g: ;"
    );

    test_transform!(
        inline_reachability,
        dont_copy_trailing_edges,
        "x, y: ? a -> c;
        a, b: 0 == 0;
        b, c: 1 == 1;
        c, d: 2 == 2;",
        "a, b: 0 == 0;
        b, c: 1 == 1;
        c, d: 2 == 2;
        x, 1: ;
        1, 2: 0 == 0;
        2, y: 1 == 1;"
    );

    test_transform!(
        inline_reachability,
        reachability_with_assign_any_main,
        "type T = { null };
        begin, end: ? a -> b;
        a, b: t = T(*);"
    );

    test_transform!(
        inline_reachability,
        reachability_with_assign_any_nested,
        "type T = { null };
        begin, end: ? x -> y;
        x, y: ? a -> b;
        a, b: t = T(*);",
        "type T = { null };
        a, b: t = T(*);
        begin, 1: ;
        1, end: ? a -> b;
        x, 2: ;
        2, y: t = T(*);"
    );

    test_transform!(
        inline_reachability,
        negated,
        "a, b: ! x -> y;
        x, y: 1 == 1;",
        "x, y: 1 == 1;
        a, 1: ;
        1, b: 1 != 1;"
    );

    test_transform!(
        inline_reachability,
        negated_skip,
        "a, b: ! x -> y;
        x, y: ;",
        "x, y: ;"
    );

    test_transform!(
        inline_reachability,
        negated_tag,
        "a, b: ! x -> y;
        x, y: $ a;",
        "x, y: $ a;"
    );

    test_transform!(
        inline_reachability,
        tag_in_main,
        "begin, a: ;
        a, b: ? x -> y;
        x, y: $ a;"
    );

    test_transform!(
        inline_reachability,
        tag_var_in_main,
        "begin, a: ;
        a, b: ? x -> y;
        x, y: $$ a;"
    );

    test_transform!(
        inline_reachability,
        tag_in_sub,
        "a1, a: ;
        a, b: ? x -> y;
        x, y: $ a;",
        "a1, a: ;
        x, y: $ a;
        a, 1: ;
        1, b: $ a;"
    );

    test_transform!(
        inline_reachability,
        tag_var_in_sub,
        "a1, a: ;
        a, b: ? x -> y;
        x, y: $$ a;",
        "a1, a: ;
        x, y: $$ a;
        a, 1: ;
        1, b: $$ a;"
    );

    test_transform!(
        inline_reachability,
        negated_tag_begin,
        "begin, end: ! x -> y;
        x, y: $ a;"
    );

    test_transform!(
        inline_reachability,
        long_negated,
        "a, b: ! x -> y;
        x, x1: 1 == 1;
        x1, y: ;"
    );

    test_transform!(
        inline_reachability,
        option_direct1,
        "a, b: ? x -> y;
        c, d: ! x -> y;
        x, y: 1 == 1;",
        "x, y: 1 == 1;
        a, 1: ;
        1, b: 1 == 1;
        c, 2: ;
        2, d: 1 != 1;"
    );

    test_transform!(
        inline_reachability,
        option_direct2,
        "a, b: ? x -> y;
        c, d: ! x -> y;
        x, x1: 1 == 1;
        x1, y: ;"
    );

    test_transform!(
        inline_reachability,
        option_direct_fork,
        "a, b: 1 == 1;
        a, c: ;
        c, b: 2 == 2;
        x, y1: ? a -> b;
        x, y2: ! a -> b;"
    );

    test_transform!(
        inline_reachability,
        option_direct_assignment,
        "a, b: ? x -> y;
        a, c: ! x -> y;
        x, y: v = 1;"
    );

    test_transform!(
        inline_reachability,
        inline_assignment1,
        "begin, a: ? e -> f;
        a, end: ;
        e, f: v = 1;",
        "a, end: ;
        e, f: v = 1;
        begin, 1: ;
        1, a: v = 1;"
    );

    test_transform!(
        inline_reachability,
        inline_assignment2,
        "begin, a: ? e -> f;
        a, a1: v = 1;
        a1, end: v == 1;
        e, f: v = 1;",
        "a, a1: v = 1;
        a1, end: v == 1;
        e, f: v = 1;
        begin, 1: ;
        1, a: v = 1;"
    );

    test_transform!(
        inline_reachability,
        inline_assignment3,
        "begin, a: ? e -> f;
        a, end: v == 1;
        e, f: v = 1;"
    );

    test_transform!(
        inline_reachability,
        inline_assignment4,
        "begin, a: ? e -> f;
        a, end: v == 1;
        e, f1: v = 1;
        e, f2: v = 2;
        f1, f: ;
        f2, f: ;"
    );

    test_transform!(
        inline_reachability,
        inline_assignment5,
        "begin, a: ? e -> f;
        a, end: ? e1 -> f1;
        e, f: coord = 0;
        e1, f1: coord == 1;",
        "begin, a: ? e -> f;
        e, f: coord = 0;
        e1, f1: coord == 1;
        a, 1: ;
        1, end: coord == 1;"
    );

    test_transform!(
        inline_reachability,
        inline_assignment6,
        "begin, a: ? e -> f;
        e, f: coord = 0;
        a, end: coord = 1;
        a, end: coord == 0;"
    );
}
