use crate::ast::{Error, Game};
use std::sync::Arc;

impl Game<Arc<str>> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Arc<str>>> {
        for index in (0..self.edges.len()).rev() {
            let bindings = self.edges[index].bindings();
            if bindings.is_empty() {
                continue;
            }

            let mappings = self.create_mappings(bindings.into_iter())?;
            let item = self.edges.remove(index);
            self.edges.splice(
                index..index,
                mappings
                    .into_iter()
                    .map(|mapping| item.substitute_bindings(&mapping)),
            );
        }

        for index in 0..self.pragmas.len() {
            let bindings = self.pragmas[index].bindings();
            if bindings.is_empty() {
                continue;
            }

            let mappings = self.create_mappings(bindings.into_iter())?;
            self.pragmas[index].substitute_bindings_mut(&mappings);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        expand_generator_nodes,
        edge,
        "type T = { a, b }; x(t: T), y: ;",
        "type T = { a, b }; x__bind__a, y: ; x__bind__b, y: ;"
    );

    test_transform!(
        expand_generator_nodes,
        pragma1,
        "type T = { a, b }; @unique x(t: T);",
        "type T = { a, b }; @unique x__bind__a x__bind__b;"
    );

    test_transform!(expand_generator_nodes,
        pragma2,
        "type T = { a, b }; @unique y(t1: T)(t2: T);",
        "type T = { a, b }; @unique y__bind__a__bind__a y__bind__a__bind__b y__bind__b__bind__a y__bind__b__bind__b;"
    );
}
