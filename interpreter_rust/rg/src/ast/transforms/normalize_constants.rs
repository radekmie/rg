use crate::ast::{Constant, Error, Game, Type, Value, ValueEntry};
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;

impl Game<Id> {
    pub fn normalize_constants(&mut self) -> Result<(), Error<Id>> {
        // Partial clone used to accumulate hoisted constants and resolve types.
        let mut game = Self {
            constants: self.constants.clone(),
            typedefs: self.typedefs.clone(),
            ..Self::default()
        };

        for x in &mut self.constants {
            hoist(&mut game, &mut x.value, &x.type_, true)?;
        }

        for x in &mut self.variables {
            hoist(&mut game, &mut x.default_value, &x.type_, false)?;
        }

        // `self.constants` is now mutated and uses the hoisted constants. Here
        // we only need to copy the new ones. And as constants can only use the
        // previously defined ones, we have to prepend them.
        self.constants
            .splice(0..0, game.constants.drain(self.constants.len()..));

        Ok(())
    }
}

fn hoist(
    game: &mut Game<Id>,
    value: &mut Arc<Value<Id>>,
    type_: &Arc<Type<Id>>,
    is_toplevel_constant: bool,
) -> Result<(), Error<Id>> {
    if let Value::Map { entries, .. } = Arc::make_mut(value) {
        if let Type::Arrow { rhs, .. } = type_.resolve(game)?.clone() {
            for ValueEntry { value, .. } in entries {
                hoist(game, value, &rhs, false)?;
            }

            // If it's not a top-level constant, hoist it.
            if !is_toplevel_constant {
                let identifier = game
                    .constants
                    .iter()
                    // We cannot use `find` + `map_or_else` as we want to mutate
                    // `game.constants` inside of the `*_else` function.
                    .find_map(|x| {
                        (x.value == *value && x.type_ == *type_).then(|| x.identifier.clone())
                    })
                    .unwrap_or_else(|| {
                        // First unused one.
                        let identifier = (1..)
                            .map(|x| Id::from(format!("Hoisted_{x}")))
                            .find(|x| game.constants.iter().all(|y| y.identifier != *x))
                            .unwrap();
                        game.constants.push(Constant {
                            span: Span::none(),
                            identifier: identifier.clone(),
                            type_: type_.clone(),
                            value: value.clone(),
                        });
                        identifier
                    });

                *value = Arc::new(Value::Element { identifier });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::test_transform;

    test_transform!(
        normalize_constants,
        ignore_toplevel_constants,
        "const X: Bool -> Bool = { :0, 1: 1 };"
    );

    test_transform!(
        normalize_constants,
        reuse_existing,
        "const X: Bool -> Bool = { :0 };
        const Y: Bool -> Bool -> Bool = { :{ :0 } };",
        "const X: Bool -> Bool = { :0 };
        const Y: Bool -> Bool -> Bool = { :X };"
    );

    test_transform!(
        normalize_constants,
        dont_reuse_different_type,
        "type A = { a, b, c };
        const X: A -> Bool = { :0 };
        const Y: Bool -> Bool -> Bool = { :{ :0 } };",
        "type A = { a, b, c };
        const Hoisted_1: Bool -> Bool = { :0 };
        const X: A -> Bool = { :0 };
        const Y: Bool -> Bool -> Bool = { :Hoisted_1 };"
    );

    test_transform!(
        normalize_constants,
        name_collision,
        "const Hoisted_1: Bool -> Bool = { :1 };
        const Y: Bool -> Bool -> Bool = { :{ :0 } };",
        "const Hoisted_2: Bool -> Bool = { :0 };
        const Hoisted_1: Bool -> Bool = { :1 };
        const Y: Bool -> Bool -> Bool = { :Hoisted_2 };"
    );

    test_transform!(
        normalize_constants,
        default_only,
        "const X: Bool -> Bool -> Bool = { :{ :0 } };",
        "const Hoisted_1: Bool -> Bool = { :0 };
        const X: Bool -> Bool -> Bool = { :Hoisted_1 };"
    );

    test_transform!(
        normalize_constants,
        multiple_different,
        "const X: Bool -> Bool -> Bool = { :{ :0 }, 1: { :1 } };",
        "const Hoisted_1: Bool -> Bool = { :0 };
        const Hoisted_2: Bool -> Bool = { :1 };
        const X: Bool -> Bool -> Bool = { :Hoisted_1, 1: Hoisted_2 };"
    );

    test_transform!(
        normalize_constants,
        multiple_reuse,
        "const X: Bool -> Bool -> Bool = { :{ :0 }, 1: { :0 } };",
        "const Hoisted_1: Bool -> Bool = { :0 };
        const X: Bool -> Bool -> Bool = { :Hoisted_1, 1: Hoisted_1 };"
    );

    test_transform!(
        normalize_constants,
        nested,
        "const X: Bool -> Bool -> Bool -> Bool -> Bool = { :{ :{ :{ :0 } } } };",
        "const Hoisted_1: Bool -> Bool = { :0 };
        const Hoisted_2: Bool -> Bool -> Bool = { :Hoisted_1 };
        const Hoisted_3: Bool -> Bool -> Bool -> Bool = { :Hoisted_2 };
        const X: Bool -> Bool -> Bool -> Bool -> Bool = { :Hoisted_3 };"
    );

    test_transform!(
        normalize_constants,
        variable,
        "var X: Bool -> Bool -> Bool = { :{ :0 } };",
        "const Hoisted_1: Bool -> Bool = { :0 };
        const Hoisted_2: Bool -> Bool -> Bool = { :Hoisted_1 };
        var X: Bool -> Bool -> Bool = Hoisted_2;"
    );
}
