use crate::ast::{Error, Expression, Game};
use std::collections::BTreeMap;
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn prune_singleton_types(&mut self) -> Result<(), Error<Arc<str>>> {
        let types: BTreeMap<_, _> = self
            .typedefs
            .iter()
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
                edge.label = edge.label.remove_casts(id);
            }

            for (id, expression) in &variables {
                if edge.label.has_variable(id) && !edge.has_binding(id) {
                    edge.label = edge.label.substitute_variable(id, expression);
                }
            }
        }

        self.typedefs
            .retain(|typedef| !types.contains_key(&*typedef.type_));
        self.variables
            .retain(|variable| !variables.contains_key(&variable.identifier));

        // TODO: Inline singleton types in bindings.
        // TODO: Inline singleton types in other types.

        Ok(())
    }
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
                actual.prune_singleton_types().unwrap();

                assert_eq!(
                    actual, expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    test!(
        remove_typedef,
        "type T = { 1 }; begin, end: ;",
        "begin, end: ;"
    );

    test!(
        remove_variable,
        "type T = { 1 }; var t: T = 1; begin, end: ;",
        "begin, end: ;"
    );

    test!(
        remove_casts,
        "type T = { 1 }; begin, end: T(1) == T(1);",
        "begin, end: 1 == 1;"
    );

    test!(
        inline_values,
        "type T = { 1 }; var t: T = 1; begin, end: t == t;",
        "begin, end: 1 == 1;"
    );
}
