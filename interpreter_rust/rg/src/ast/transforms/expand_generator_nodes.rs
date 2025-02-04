use crate::ast::{Error, Game};
use std::sync::Arc;

type Id = Arc<str>;

impl Game<Id> {
    pub fn expand_generator_nodes(&mut self) -> Result<(), Error<Id>> {
        todo!()
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

    test_transform!(
        expand_generator_nodes,
        pragma3,
        "type T = { a, b }; @simpleApply x y(t: T) [t: T];",
        "type T = { a, b }; @simpleApply x y__bind__a [a]; @simpleApply x y__bind__b [b];"
    );

    test_transform!(
        expand_generator_nodes,
        pragma4,
        "type T = { a, b }; var v: T = a; @simpleApply x y(t: T) [t: T] v = t;",
        "type T = { a, b }; var v: T = a; @simpleApply x y__bind__a [a] v = T(a); @simpleApply x y__bind__b [b] v = T(b);"
    );

    test_transform!(
        expand_generator_nodes,
        pragma5,
        "type T1 = { a, b }; type T2 = { a, b }; @unique x1(t1: T1) x2(t2: T2);",
        "type T1 = { a, b }; type T2 = { a, b }; @unique x1__bind__a x1__bind__b; @unique x2__bind__a x2__bind__b;"
    );
}
