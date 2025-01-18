use crate::ast::{Error, Expression, Game, Label};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

const IGNORED_TYPES: [&str; 2] = ["Player", "Score"];

impl Game<Arc<str>> {
    pub fn prune_singleton_types(&mut self) -> Result<(), Error<Arc<str>>> {
        let used_types: BTreeSet<_> = IGNORED_TYPES
            .into_iter()
            .chain(
                None.into_iter()
                    .chain(self.constants.iter().map(|constant| &constant.type_))
                    .chain(self.typedefs.iter().map(|typedef| &typedef.type_))
                    .filter_map(|type_| type_.type_references())
                    .flatten()
                    .map(Arc::as_ref),
            )
            .collect();

        let types: BTreeMap<_, _> = self
            .typedefs
            .iter()
            .filter(|typedef| !used_types.contains(&typedef.identifier.as_ref()))
            .filter_map(|typedef| {
                typedef.type_.as_singleton().map(|id| {
                    let pair = (typedef.identifier.clone(), id.clone());
                    (typedef.type_.clone(), pair)
                })
            })
            .collect();

        let variables: BTreeMap<_, _> = self
            .variables
            .iter()
            .filter_map(|variable| {
                variable
                    .type_
                    .resolve(self)
                    .ok()
                    .and_then(|type_| types.get(type_))
                    .map(|(_, id)| (variable.identifier.clone(), Expression::new((*id).clone())))
            })
            .collect();

        for edge in &mut self.edges {
            for (id, _) in types.values() {
                Arc::make_mut(edge).label = edge.label.remove_casts(id);
            }

            for (id, expression) in &variables {
                if edge.label.has_variable(id) && !edge.has_binding(id) {
                    // If we'd substitute the assigned variable, then the lhs
                    // would become constant (symbol), and the assignment would
                    // be illegal. To prevent that, we skip the whole assignment
                    // instead. It's legal, as the assignment are always legal.
                    if let Label::Assignment { lhs, .. } = &edge.label {
                        if lhs.uncast().is_reference_and(|lhs| lhs == id) {
                            Arc::make_mut(edge).skip();
                            continue;
                        }
                    }

                    Arc::make_mut(edge).label = edge.label.substitute_variable(id, expression);
                }
            }
        }

        self.typedefs
            .retain(|typedef| !types.contains_key(&*typedef.type_));
        self.variables
            .retain(|variable| !variables.contains_key(&variable.identifier));

        // TODO: Inline singleton types in bindings.

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        prune_singleton_types,
        remove_typedef,
        "type T = { 1 }; begin, end: ;",
        "begin, end: ;"
    );

    test_transform!(
        prune_singleton_types,
        remove_variable,
        "type T = { 1 }; var t: T = 1; begin, end: ;",
        "begin, end: ;"
    );

    test_transform!(
        prune_singleton_types,
        remove_casts,
        "type T = { 1 }; begin, end: T(1) == T(1);",
        "begin, end: 1 == 1;"
    );

    test_transform!(
        prune_singleton_types,
        referenced_in_constant,
        "type T = { 1 }; type U = { 2, 3 }; const C: U -> T = { :1 }; begin, end: ;"
    );

    test_transform!(
        prune_singleton_types,
        referenced_in_typedef,
        "type T = { 1 }; type U = T -> T; begin, end: ;"
    );

    test_transform!(
        prune_singleton_types,
        inline_values,
        "type T = { 1 }; var t: T = 1; begin, end: t == t;",
        "begin, end: 1 == 1;"
    );

    test_transform!(
        prune_singleton_types,
        skip_assignments,
        "type T = { 1 }; var t: T = 1; begin, end: t = 1;",
        "begin, end: ;"
    );

    test_transform!(
        prune_singleton_types,
        ignore_player,
        "type Player = { 1 }; begin, end: ;"
    );
}
