use super::Analysis;
use crate::ast::{Edge, Expression, Game, Label, Type, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Id = Arc<str>;
type Ids = BTreeSet<Id>;
type ValueSetMap = BTreeMap<Id, ValueSet>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ValueSet {
    /// Set of possible symbols.
    Set(Ids),
    /// Something the analysis could not process.
    Unknown,
}

impl ValueSet {
    #[allow(dead_code)]
    fn merge_with<F: Fn(&mut Ids, &Ids)>(&mut self, other: &Self, fn_: &F) {
        match self {
            Self::Set(xs) => match other {
                Self::Set(ys) => fn_(xs, ys),
                Self::Unknown => *self = Self::Unknown,
            },
            Self::Unknown => {}
        }
    }

    #[allow(dead_code)]
    fn merge_with_at<F: Fn(&mut Ids, &Ids)>(xs: &mut ValueSetMap, key: Id, y: Self, fn_: &F) {
        xs.entry(key)
            .and_modify(|x| x.merge_with(&y, &fn_))
            .or_insert(y);
    }

    #[allow(dead_code)]
    fn new() -> Self {
        Self::from(BTreeSet::new())
    }
}

impl From<Ids> for ValueSet {
    fn from(ids: Ids) -> Self {
        Self::Set(ids)
    }
}

impl From<&Id> for ValueSet {
    fn from(id: &Id) -> Self {
        Self::Set(BTreeSet::from([id.clone()]))
    }
}

impl From<&Value<Id>> for ValueSet {
    fn from(value: &Value<Id>) -> Self {
        match value {
            Value::Element { identifier } => Self::from(identifier),
            Value::Map { .. } => Self::Unknown,
        }
    }
}

#[allow(dead_code)]
fn merge_sets(xs: &mut Ids, ys: &Ids) {
    xs.extend(ys.iter().cloned());
}

#[allow(dead_code)]
pub struct ValueSets {
    constants: BTreeMap<Id, Arc<Value<Id>>>,
    initial_by_variable: ValueSetMap,
    maximum_by_type: ValueSetMap,
    maximum_by_variable: ValueSetMap,
}

impl ValueSets {
    #[allow(dead_code)]
    fn resolve(&self, state: &ValueSetMap, identifier: &Id) -> ValueSet {
        self.constants.get(identifier).map_or_else(
            || {
                state
                    .get(identifier)
                    .map_or_else(|| ValueSet::from(identifier), Clone::clone)
            },
            |value| match value.as_ref() {
                Value::Element { identifier } => ValueSet::from(identifier),
                Value::Map { .. } => ValueSet::Unknown,
            },
        )
    }
}

impl Analysis for ValueSets {
    type Domain = ValueSetMap;

    fn bot(&self) -> Self::Domain {
        self.maximum_by_variable.clone()
    }

    fn extreme(&self, _game: &Game<Id>) -> Self::Domain {
        self.initial_by_variable.clone()
    }

    fn join(&self, mut xs: Self::Domain, ys: Self::Domain) -> Self::Domain {
        for (key, y) in ys {
            ValueSet::merge_with_at(&mut xs, key, y, &merge_sets);
        }
        xs
    }

    fn transfer(&self, mut input: Self::Domain, edge: &Arc<Edge<Id>>) -> Self::Domain {
        match &edge.label {
            Label::Assignment { lhs, rhs } => 'block: {
                let variable = match lhs.uncast() {
                    // We do not track maps (yet).
                    Expression::Access { .. } => break 'block,
                    Expression::Cast { .. } => unreachable!("Cast should be resolved."),
                    Expression::Reference { identifier } => identifier,
                };

                // Only variables can be refined.
                if !self.maximum_by_variable.contains_key(variable) {
                    break 'block;
                }

                let mut value_set = rhs.uncast().as_reference().map_or_else(
                    || {
                        let (identifier, accessors) = rhs.access_identifier_with_accessors();

                        // TODO: Handle nested accessers.
                        if accessors.len() > 1 {
                            return self.maximum_by_variable.get(variable).unwrap().clone();
                        }

                        accessors
                            .iter()
                            .fold(
                                self.constants.get(identifier).map_or_else(
                                    || {
                                        Ok(input
                                            .get(identifier)
                                            .unwrap_or_else(|| {
                                                panic!("Cannot resolve {identifier}.")
                                            })
                                            .clone())
                                    },
                                    |value| Err(value.clone()),
                                ),
                                |result, accessor| match result {
                                    Ok(ValueSet::Set(_)) => {
                                        println!("{lhs} {rhs} {result:?} {accessor:?}");
                                        unreachable!("Set cannot be accessed.")
                                    }
                                    Ok(ValueSet::Unknown) => Ok(ValueSet::Unknown),
                                    Err(value) => match value.as_ref() {
                                        Value::Element { .. } => {
                                            unreachable!("Element cannot be accessed.")
                                        }
                                        Value::Map { entries, .. } => match accessor.uncast() {
                                            Expression::Access { .. } => {
                                                unreachable!("Access should be resolved.")
                                            }
                                            Expression::Cast { .. } => {
                                                unreachable!("Cast should be resolved.")
                                            }
                                            Expression::Reference { identifier } => {
                                                let default = ValueSet::from(
                                                    entries
                                                        .iter()
                                                        .find(|entry| entry.identifier.is_none())
                                                        .expect("Map is missing default value.")
                                                        .value
                                                        .as_ref(),
                                                );

                                                Ok(match self.resolve(&input, identifier) {
                                                    ValueSet::Set(xs) => xs
                                                        .into_iter()
                                                        .map(|x| {
                                                            entries
                                                                .iter()
                                                                .find_map(|entry| {
                                                                    entry
                                                                        .identifier
                                                                        .as_ref()
                                                                        .filter(|y| x == **y)
                                                                        .map(|_| {
                                                                            ValueSet::from(
                                                                                entry
                                                                                    .value
                                                                                    .as_ref(),
                                                                            )
                                                                        })
                                                                })
                                                                .unwrap_or_else(|| default.clone())
                                                        })
                                                        .fold(ValueSet::new(), |mut x, y| {
                                                            x.merge_with(&y, &merge_sets);
                                                            x
                                                        }),
                                                    ValueSet::Unknown => ValueSet::Unknown,
                                                })
                                            }
                                        },
                                    },
                                },
                            )
                            .unwrap_or(ValueSet::Unknown)
                    },
                    |identifier| self.resolve(&input, identifier),
                );

                // Initial values have to be merged.
                if edge.rhs.is_begin() {
                    value_set
                        .merge_with(self.initial_by_variable.get(variable).unwrap(), &merge_sets);
                }

                input.insert(variable.clone(), value_set);
            }
            Label::AssignmentAny { lhs, rhs } => {
                let Some(variable) = lhs.uncast().as_reference() else {
                    todo!();
                };

                let mut value_set = match rhs.as_ref() {
                    Type::Arrow { .. } => unreachable!("Cannot AssignAny arrow type."),
                    Type::Set { .. } => todo!(),
                    Type::TypeReference { identifier } => {
                        self.maximum_by_type.get(identifier).unwrap().clone()
                    }
                };

                // Initial values have to be merged.
                if edge.rhs.is_begin() {
                    value_set
                        .merge_with(self.initial_by_variable.get(variable).unwrap(), &merge_sets);
                }

                input.insert(variable.clone(), value_set);
            }
            Label::Comparison { lhs, rhs, negated } => {
                // Comparisons refine in both directions.
                for (lhs, rhs) in [(lhs, rhs), (rhs, lhs)] {
                    let variable = match lhs.uncast() {
                        // We do not track maps (yet).
                        Expression::Access { .. } => continue,
                        Expression::Cast { .. } => unreachable!("Cast should be resolved."),
                        Expression::Reference { identifier } => identifier,
                    };

                    // Only variables can be refined.
                    if !self.maximum_by_variable.contains_key(variable) {
                        continue;
                    }

                    match rhs.uncast() {
                        Expression::Access { .. } => {
                            // TODO: If `rhs` has no map variables, we can
                            // iterate through all possible values.
                        }
                        Expression::Cast { .. } => unreachable!("Cast should be resolved."),
                        Expression::Reference { identifier } => {
                            let y = self.resolve(&input, identifier);
                            ValueSet::merge_with_at(&mut input, variable.clone(), y, &|xs, ys| {
                                xs.retain(|x| ys.contains(x) != *negated);
                            });
                        }
                    }
                }
            }
            Label::Reachability { .. } => {}
            Label::Skip { .. } => {}
            Label::Tag { .. } => {}
            Label::TagVariable { .. } => {}
        }

        input
    }

    fn with_reachability(&self) -> bool {
        true
    }
}

impl From<&Game<Id>> for ValueSets {
    fn from(game: &Game<Id>) -> Self {
        fn resolve_type(game: &Game<Id>, type_: &Arc<Type<Id>>) -> ValueSet {
            match type_.resolve(game).unwrap() {
                Type::Arrow { .. } => ValueSet::Unknown,
                Type::Set { identifiers, .. } => {
                    ValueSet::from(identifiers.iter().cloned().collect::<BTreeSet<_>>())
                }
                Type::TypeReference { .. } => unreachable!("TypeReference should be resolved."),
            }
        }

        let constants = game
            .constants
            .iter()
            .map(|constant| (constant.identifier.clone(), constant.value.clone()))
            .collect();

        let maximum_by_type = game
            .typedefs
            .iter()
            .map(|typedef| {
                let maximum = resolve_type(game, &typedef.type_);
                (typedef.identifier.clone(), maximum)
            })
            .collect();

        let (initial_by_variable, maximum_by_variable) = game
            .variables
            .iter()
            .map(|variable| {
                let maximum = resolve_type(game, &variable.type_);
                let initial = match variable.default_value.as_ref() {
                    Value::Element { identifier } => {
                        let identifier = game.resolve_constant(identifier).map_or_else(
                            || identifier,
                            |constant| match constant.value.as_ref() {
                                Value::Element { identifier } => identifier,
                                Value::Map { .. } => unreachable!("Incorrect value type."),
                            },
                        );

                        ValueSet::from(identifier)
                    }
                    Value::Map { .. } => ValueSet::Unknown,
                };

                let id = &variable.identifier;
                ((id.clone(), initial), (id.clone(), maximum))
            })
            .unzip();

        Self {
            constants,
            initial_by_variable,
            maximum_by_type,
            maximum_by_variable,
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Analysis, ValueSet, ValueSets};
    use crate::ast::{Game, Node};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn format_analysis(
        analysis: BTreeMap<Node<Arc<str>>, <ValueSets as Analysis>::Domain>,
    ) -> String {
        let mut result = String::new();
        result.push('\n');
        for (node, variables) in analysis {
            result.push_str(&format!("        {node}:\n"));
            for (identifier, possible_values) in variables {
                result.push_str(&format!("            {identifier}: "));
                match possible_values {
                    ValueSet::Set(ids) => {
                        result.push('{');
                        let mut iter = ids.iter();
                        if let Some(id) = iter.next() {
                            result.push_str(&format!("{id}"));
                            for id in iter {
                                result.push_str(&format!(", {id}"));
                            }
                        }
                        result.push_str("}\n");
                    }
                    ValueSet::Unknown => result.push_str("(unknown)\n"),
                }
            }
        }
        result
    }

    macro_rules! test {
        ($name:ident, $source:expr, $expect:expr) => {
            #[test]
            fn $name() {
                Game::test_analysis(
                    $source,
                    $expect,
                    Box::new(|game| ValueSets::from(game)),
                    Box::new(format_analysis),
                );
            }
        };
    }

    test!(
        initial_symbol,
        "type T = { a, b, c }; var x: T = a;",
        "begin:
            x: {a}"
    );

    test!(
        initial_constant,
        "type T = { a, b, c }; const x: T = a; var y: T = x;",
        "begin:
            y: {a}"
    );

    test!(
        assign,
        "type T = { a, b, c }; var x: T = a; begin, q1: x = b;",
        "begin:
            x: {a}
        q1:
            x: {b}"
    );

    test!(
        assign_any,
        "type T = { a, b, c }; var x: T = a; begin, q1: x = T(*);",
        "begin:
            x: {a}
        q1:
            x: {a, b, c}"
    );

    test!(
        compare_positive,
        "type T = { a, b, c }; var x: T = a; begin, q1: x = T(*); q1, q2: x == b;",
        "begin:
            x: {a}
        q1:
            x: {a, b, c}
        q2:
            x: {b}"
    );

    test!(
        compare_negative,
        "type T = { a, b, c }; var x: T = a; begin, q1: x = T(*); q1, q2: x != b;",
        "begin:
            x: {a}
        q1:
            x: {a, b, c}
        q2:
            x: {a, c}"
    );

    test!(
        sample_flow_1,
        "type T = { a, b, c };
        var x: T = a;
        begin, q1: x = a;
        begin, q1: x = b;
        q1, q2: x == b;
        q1, q2: x == c;",
        "begin:
            x: {a}
        q1:
            x: {a, b}
        q2:
            x: {b}"
    );

    test!(
        sample_flow_2,
        "type Int = { 0, 1, 2, 3 };
        const inc: Int -> Int = { :3, 0: 1, 1: 2 };
        var x: Int = 0;
        begin, q1: x != 3;
        q1, begin: x = inc[x];
        begin, q2: x == 3;",
        "begin:
            x: {0, 1, 2, 3}
        q1:
            x: {0, 1, 2}
        q2:
            x: {3}"
    );

    test!(
        sample_flow_3,
        "type Int = { 0, 1, 2, 3 };
        const inc: Int -> Int = { :3, 0: 1, 1: 2 };
        var x: Int = 0;
        begin, q0: ;
        q0, q1: 3 != x;
        q1, q0: x = inc[x];
        q0, q2: 3 == x;",
        "begin:
            x: {0}
        q0:
            x: {0, 1, 2, 3}
        q1:
            x: {0, 1, 2}
        q2:
            x: {3}"
    );
}
