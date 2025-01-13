use crate::ast::analyses::ReachingBindingAssignments;
use crate::ast::{Edge, Error, Game, Label, Node, Type};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type BindingAssignments<Id> = BTreeMap<Id, (Arc<Edge<Id>>, Id)>;
type EdgesWithIdx<'a, Id> = BTreeSet<(usize, &'a Arc<Edge<Id>>)>;
type Id = Arc<str>;
type MappingEntry<Id> = (Id, Arc<Type<Id>>, Id);

impl Game<Id> {
    pub fn merge_bindings(&mut self) -> Result<(), Error<Arc<str>>> {
        while self.merge_bindings_step() {}
        Ok(())
    }

    fn merge_bindings_step(&mut self) -> bool {
        let mut changed = false;
        while let Some((simple_path, to_rename, (bind, type_, orig_bind))) = self.find_to_join() {
            changed = true;
            let mapping = BTreeMap::from([(bind.clone(), (orig_bind.clone(), type_.clone()))]);
            for idx in to_rename {
                self.edges[idx] = Arc::from(self.edges[idx].rename_variables(&mapping));
            }
            for idx in simple_path {
                if !self.edges[idx].lhs.has_binding(&orig_bind) {
                    Arc::make_mut(&mut self.edges[idx])
                        .lhs
                        .add_binding(orig_bind.clone(), type_.clone());
                }
                if !self.edges[idx].rhs.has_binding(&orig_bind) {
                    Arc::make_mut(&mut self.edges[idx])
                        .rhs
                        .add_binding(orig_bind.clone(), type_.clone());
                }
            }
        }
        changed
    }

    fn find_to_join(&self) -> Option<(Vec<usize>, Vec<usize>, MappingEntry<Id>)> {
        let prev_edges = self.prev_edges();
        let next_edges = self.next_edges_with_idx();
        let binding_assignments = self.analyse::<ReachingBindingAssignments>(true);
        let reachability_targets = self.reachability_targets();
        for (edge_idx, edge) in self.edges.iter().enumerate() {
            if edge.lhs.has_bindings() {
                continue;
            }
            for (bind, type_) in edge.rhs.bindings() {
                let Some((simple_path, mut to_rename, orig_bind)) = self.try_this(
                    edge,
                    (bind, type_),
                    &prev_edges,
                    &next_edges,
                    &binding_assignments,
                    &reachability_targets,
                ) else {
                    continue;
                };

                to_rename.push(edge_idx);

                return Some((
                    simple_path,
                    to_rename,
                    ((*bind).clone(), (*type_).clone(), orig_bind),
                ));
            }
        }

        None
    }

    fn try_this<'a>(
        &'a self,
        edge: &'a Arc<Edge<Id>>,
        (bind, type_): (&'a Id, &'a Arc<Type<Id>>),
        prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
        next_edges: &BTreeMap<&Node<Id>, EdgesWithIdx<Id>>,
        binding_assignments: &'a BTreeMap<Node<Id>, BindingAssignments<Id>>,
        reachability_targets: &BTreeSet<&Node<Id>>,
    ) -> Option<(Vec<usize>, Vec<usize>, Id)> {
        let Label::Comparison {
            lhs,
            rhs,
            negated: false,
        } = &edge.label
        else {
            return None;
        };
        let variable = rhs.uncast().as_reference()?;
        if !(lhs.uncast().is_reference_and(|id| id == bind)
            || self.variables.iter().any(|var| var.identifier == *variable))
        {
            return None;
        }
        let (def_edge, orig_bind) = binding_assignments
            .get(&edge.lhs)
            .and_then(|x| x.get(variable))
            .filter(|(def_edge, orig_bind)| {
                def_edge
                    .get_binding(orig_bind)
                    .is_some_and(|(_, orig_type)| orig_type == type_ && orig_bind != bind)
            })?;
        // Types of bindings match, we can merge them if there is simple path and no other bindings
        let simple_path = find_simple_path(
            &def_edge.lhs,
            &edge.rhs,
            orig_bind,
            prev_edges,
            next_edges,
            reachability_targets,
        )?;

        let edges_using_binding = edges_using_binding(&edge.rhs, bind, orig_bind, next_edges)?;

        Some((simple_path, edges_using_binding, (*orig_bind).clone()))
    }
}

fn find_simple_path(
    start: &Node<Id>,
    end: &Node<Id>,
    orig_bind: &Id,
    prev_edges: &BTreeMap<&Node<Id>, BTreeSet<&Arc<Edge<Id>>>>,
    next_edges: &BTreeMap<&Node<Id>, EdgesWithIdx<Id>>,
    reachability_targets: &BTreeSet<&Node<Id>>,
) -> Option<Vec<usize>> {
    let mut path = vec![];
    let mut curr = start;
    while let Some((idx, next)) = get_singleton(next_edges, curr) {
        // Must have one predecessor
        let _ = get_singleton(prev_edges, &next.rhs)?;

        path.push(*idx);

        if &next.rhs == end {
            return Some(path);
        }
        // Can only have orig_bind as binding
        if next.lhs.bindings().any(|(bind, _)| bind != orig_bind)
            || reachability_targets.contains(curr)
            || next.label.is_player_assignment()
        {
            return None;
        }

        curr = &next.rhs;
    }

    None
}

fn edges_using_binding(
    start: &Node<Id>,
    bind: &Id,
    new_bind: &Id,
    next_edges: &BTreeMap<&Node<Id>, EdgesWithIdx<Id>>,
) -> Option<Vec<usize>> {
    let mut queue = vec![start];
    let mut seen = BTreeSet::new();
    let mut to_rename = vec![];
    while let Some(lhs) = queue.pop() {
        if lhs.has_binding(new_bind) {
            return None;
        }
        if !seen.insert(lhs) || !lhs.has_binding(bind) {
            continue;
        }

        let Some(edges) = next_edges.get(&lhs) else {
            continue;
        };

        for (idx, edge) in edges {
            queue.push(&edge.rhs);
            to_rename.push(*idx);
        }
    }

    Some(to_rename)
}

fn get_singleton<K: Ord, V: Ord>(map: &BTreeMap<K, BTreeSet<V>>, key: K) -> Option<&V> {
    if let Some(set) = map.get(&key) {
        if set.len() == 1 {
            return set.iter().next();
        }
    }
    None
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        merge_bindings,
        small,
        "type Coord = {0,1,2};
        var posX: Coord = 0;
        begin, a: ; // 0
        a, b(bind: Coord): bind != 1; // 1
        b(bind: Coord), c: posX = bind; // 2
        c, d: ; // 3
        d, e(bind2: Coord): bind2 == posX; // 4
        e(bind2: Coord), f: $ bind2; // 5
        f, end: ; // 6 ",
        "type Coord = { 0, 1, 2 };
        var posX: Coord = 0;
        begin, a: ;
        a, b(bind: Coord): bind != 1;
        b(bind: Coord), c(bind: Coord): posX = bind;
        c(bind: Coord), d(bind: Coord): ;
        d(bind: Coord), e(bind: Coord): Coord(bind) == posX;
        e(bind: Coord), f: $ bind;
        f, end: ;"
    );
}
