use hrg::ast as hrg;
use natord::compare;
use rg::ast as rg;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;

struct Context {
    counters: BTreeMap<Id, usize>,
    hrg: hrg::Game<Id>,
    reuse_functions: bool,
    rg: rg::Game<Id>,
    translated_functions: BTreeSet<Id>,
    type_values: BTreeMap<Id, Vec<hrg::Value<Id>>>,
}

impl Context {
    fn connect(
        &mut self,
        mut lhs: rg::Node<Id>,
        mut rhs: rg::Node<Id>,
        label: rg::Label<Id>,
        bindings: &[rg::Binding<Id>],
    ) {
        let parts: Vec<_> = bindings
            .iter()
            .map(|(identifier, type_)| rg::NodePart::Binding {
                span: Span::none(),
                identifier: (*identifier).clone(),
                type_: (*type_).clone(),
            })
            .collect();

        lhs.parts.extend(parts.clone());
        rhs.parts.extend(parts);
        self.rg.edges.push(rg::Edge::new(lhs, rhs, label));
    }

    fn random(&mut self, prefix: &Id) -> Id {
        let index = self.counters.entry(prefix.clone()).or_default();
        *index += 1;

        Id::from(format!("{prefix}_{index}"))
    }

    fn random_node(&mut self, prefix: &Id) -> rg::Node<Id> {
        rg::Node::new(self.random(prefix))
    }
}

pub fn hrg_to_rg(hrg: hrg::Game<Id>, reuse_functions: bool) -> rg::Game<Id> {
    let mut context = Context {
        counters: BTreeMap::new(),
        hrg,
        reuse_functions,
        rg: rg::Game::default(),
        translated_functions: BTreeSet::new(),
        type_values: BTreeMap::new(),
    };

    translate_domains(&mut context);
    translate_functions(&mut context);
    translate_variables(&mut context);

    context.rg.edges.push(rg::Edge {
        span: Span::none(),
        label: rg::Label::new_skip(),
        lhs: rg::Node::new(Id::from("begin")),
        rhs: rg::Node::new(Id::from("rules_begin")),
    });

    let rules = context
        .hrg
        .automaton
        .iter()
        .find(|automaton_function| automaton_function.name.as_ref() == "rules")
        .expect("No `rules` automation function found.")
        .clone();

    translate_automaton_function(
        &mut context,
        &rules,
        Some(&rg::Node::new(Id::from("end"))),
        Some(&rg::Node::new(Id::from("end"))),
        &Id::from(""),
    );

    context.rg
}

#[expect(clippy::needless_pass_by_value)]
fn cartesian<T: Clone>(xss: Vec<Vec<T>>, ys: Vec<T>) -> Vec<Vec<T>> {
    xss.into_iter()
        .flat_map(|xs| {
            ys.iter().cloned().map(move |y| {
                let mut xs = xs.clone();
                xs.push(y);
                xs
            })
        })
        .collect()
}

fn compare_identifiers(lhs: &Id, rhs: &Id) -> Ordering {
    compare(lhs, rhs)
}

fn compare_values(lhs: &hrg::Value<Id>, rhs: &hrg::Value<Id>) -> Ordering {
    match (lhs, rhs) {
        (
            hrg::Value::Constructor {
                identifier: li,
                args: la,
            },
            hrg::Value::Constructor {
                identifier: ri,
                args: ra,
            },
        ) => compare_identifiers(li, ri)
            .then_with(|| la.len().cmp(&ra.len()))
            .then_with(|| {
                la.iter().zip(ra).fold(Ordering::Equal, |ord, (lhs, rhs)| {
                    ord.then_with(|| compare_values(lhs, rhs))
                })
            }),
        (hrg::Value::Element { identifier: li }, hrg::Value::Element { identifier: ri }) => {
            compare_identifiers(li, ri)
        }
        (hrg::Value::Map { .. }, hrg::Value::Map { .. }) => unimplemented!(),
        _ => panic!("Incomparable values."),
    }
}

fn construct_map(mut entries: Vec<rg::ValueEntry<Id>>) -> rg::Value<Id> {
    assert!(!entries.is_empty(), "At least one entry is required.");

    let default_value = entries
        .iter()
        .fold(vec![], |mut counts: Vec<(_, _)>, entry| {
            match counts.iter_mut().find(|count| count.0 == entry.value) {
                Some(count) => count.1 += 1,
                None => counts.push((entry.value.clone(), 1)),
            }
            counts
        })
        .into_iter()
        .reduce(|x, y| if x.1 >= y.1 { x } else { y })
        .unwrap()
        .0;

    entries.retain(|entry| !evaluate_equality(&entry.value, &default_value));
    entries.insert(
        0,
        rg::ValueEntry {
            span: Span::none(),
            identifier: None,
            value: default_value,
        },
    );

    rg::Value::Map {
        span: Span::none(),
        entries,
    }
}

fn evaluate_binding(
    pattern: &hrg::Pattern<Id>,
    value: &hrg::Value<Id>,
) -> Option<BTreeMap<Id, hrg::Value<Id>>> {
    match (pattern, value) {
        (
            hrg::Pattern::Constructor {
                identifier: pi,
                args: pa,
            },
            hrg::Value::Constructor {
                identifier: vi,
                args: va,
            },
        ) if pi == vi && pa.len() == va.len() => {
            pa.iter()
                .enumerate()
                .try_fold(BTreeMap::new(), |mut binding, (index, pattern)| {
                    evaluate_binding(pattern, &va[index]).map(|subbinding| {
                        binding.extend(subbinding);
                        binding
                    })
                })
        }
        (hrg::Pattern::Literal { identifier: pi }, hrg::Value::Element { identifier: vi })
            if pi == vi =>
        {
            Some(BTreeMap::new())
        }
        (hrg::Pattern::Variable { identifier }, _) => {
            Some(BTreeMap::from([(identifier.clone(), value.clone())]))
        }
        (hrg::Pattern::Wildcard, _) => Some(BTreeMap::new()),
        _ => None,
    }
}

fn evaluate_condition(
    expression: &hrg::Expression<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> bool {
    let hrg::Expression::BinExpr { lhs, op, rhs } = expression else {
        panic!("Expression \"{expression}\" is not a valid condition.");
    };

    match op {
        hrg::Binop::And => evaluate_condition(lhs, binding) && evaluate_condition(rhs, binding),
        hrg::Binop::Or => evaluate_condition(lhs, binding) || evaluate_condition(rhs, binding),
        hrg::Binop::Eq
        | hrg::Binop::Gt
        | hrg::Binop::Gte
        | hrg::Binop::Lt
        | hrg::Binop::Lte
        | hrg::Binop::Ne => {
            let lhs = evaluate_expression(lhs, binding);
            let rhs = evaluate_expression(rhs, binding);
            let ord = compare_values(&lhs, &rhs);
            match op {
                hrg::Binop::Eq => ord.is_eq(),
                hrg::Binop::Gt => ord.is_gt(),
                hrg::Binop::Gte => ord.is_gt() || ord.is_eq(),
                hrg::Binop::Lt => ord.is_lt(),
                hrg::Binop::Lte => ord.is_lt() || ord.is_eq(),
                hrg::Binop::Ne => !ord.is_eq(),
                _ => panic!("Expression \"{expression}\" is not a valid condition."),
            }
        }
        _ => panic!("Expression \"{expression}\" is not a valid condition."),
    }
}

fn evaluate_default_value(context: &Context, type_: &rg::Type<Id>) -> hrg::Value<Id> {
    match type_ {
        // NOTE: Is this even correct?
        rg::Type::Arrow { lhs, rhs } => hrg::Value::Map {
            entries: evaluate_type_values(context, lhs)
                .iter()
                .map(|value| hrg::ValueMapEntry {
                    key: Arc::from(value.clone()),
                    value: Arc::from(evaluate_default_value(context, rhs)),
                })
                .collect(),
        },
        rg::Type::Set { .. } => unimplemented!(),
        rg::Type::TypeReference { .. } => evaluate_type_values(context, type_)
            .first()
            .unwrap()
            .clone(),
    }
}

fn evaluate_domain_values(
    domain_values: &[hrg::DomainValue<Id>],
) -> Vec<BTreeMap<Id, hrg::Value<Id>>> {
    for domain_value in &domain_values[1..] {
        assert_ne!(
            domain_value.identifier(),
            domain_values.first().unwrap().identifier(),
            "Duplicated identifier."
        );
    }

    domain_values
        .iter()
        .map(|domain_value| match domain_value {
            hrg::DomainValue::Range {
                identifier,
                min,
                max,
            } => (*min..=*max)
                .map(|index| {
                    (
                        identifier.clone(),
                        hrg::Value::Element {
                            identifier: Id::from(format!("{index}")),
                        },
                    )
                })
                .collect(),
            hrg::DomainValue::Set {
                identifier,
                elements,
            } => elements
                .iter()
                .map(|element| {
                    (
                        identifier.clone(),
                        hrg::Value::Element {
                            identifier: element.clone(),
                        },
                    )
                })
                .collect(),
        })
        .fold(vec![vec![]], cartesian)
        .into_iter()
        .map(|bindings| {
            bindings
                .into_iter()
                .fold(BTreeMap::new(), |mut binding, (key, value)| {
                    binding.insert(key, value);
                    binding
                })
        })
        .collect()
}

fn evaluate_equality(lhs: &rg::Value<Id>, rhs: &rg::Value<Id>) -> bool {
    match (lhs, rhs) {
        (rg::Value::Element { identifier: lhs }, rg::Value::Element { identifier: rhs }) => {
            lhs == rhs
        }
        (rg::Value::Map { entries: lhss, .. }, rg::Value::Map { entries: rhss, .. }) => {
            lhss.len() == rhss.len()
                && lhss.iter().all(|lhs| {
                    rhss.iter()
                        .find(|rhs| rhs.identifier == lhs.identifier)
                        .is_some_and(|rhs| evaluate_equality(&lhs.value, &rhs.value))
                })
        }
        _ => panic!("Equality for different kinds."),
    }
}

fn evaluate_expression(
    expression: &hrg::Expression<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> hrg::Value<Id> {
    match expression {
        hrg::Expression::BinExpr { lhs, op, rhs }
            if matches!(op, hrg::Binop::Add | hrg::Binop::Mod | hrg::Binop::Sub) =>
        {
            let lhs: i32 = evaluate_expression_identifier(lhs, binding)
                .parse()
                .unwrap();
            let rhs: i32 = evaluate_expression_identifier(rhs, binding)
                .parse()
                .unwrap();
            let value = match op {
                hrg::Binop::Add => lhs + rhs,
                hrg::Binop::Mod => (lhs + rhs) % rhs,
                hrg::Binop::Sub => lhs - rhs,
                _ => unreachable!(),
            };
            hrg::Value::Element {
                identifier: Arc::from(format!("{value}")),
            }
        }
        hrg::Expression::Call { .. } => unimplemented!(),
        hrg::Expression::Constructor { identifier, args } => hrg::Value::Constructor {
            identifier: identifier.clone(),
            args: args
                .iter()
                .map(|expression| Arc::from(evaluate_expression(expression, binding)))
                .collect(),
        },
        hrg::Expression::If { cond, then, else_ } => {
            if evaluate_condition(cond, binding) {
                evaluate_expression(then, binding)
            } else {
                evaluate_expression(else_, binding)
            }
        }
        hrg::Expression::Literal { identifier } => {
            binding
                .get(identifier)
                .cloned()
                .unwrap_or_else(|| hrg::Value::Element {
                    identifier: identifier.clone(),
                })
        }
        hrg::Expression::Map {
            pattern,
            expression,
            domains,
        } => hrg::Value::Map {
            entries: evaluate_domain_values(domains)
                .into_iter()
                .map(|subbinding| {
                    let mut binding = binding.clone();
                    binding.extend(subbinding);
                    binding
                })
                .map(|binding| hrg::ValueMapEntry {
                    key: Arc::from(evaluate_pattern(pattern, &binding)),
                    value: Arc::from(evaluate_expression(expression, &binding)),
                })
                .collect(),
        },
        _ => unimplemented!(),
    }
}

fn evaluate_expression_identifier(
    expression: &hrg::Expression<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> Id {
    match evaluate_expression(expression, binding) {
        hrg::Value::Element { identifier } => identifier.clone(),
        _ => panic!("Expected ValueElement."),
    }
}

fn evaluate_pattern(
    pattern: &hrg::Pattern<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> hrg::Value<Id> {
    match pattern {
        hrg::Pattern::Constructor { identifier, args } => hrg::Value::Constructor {
            identifier: identifier.clone(),
            args: args
                .iter()
                .map(|pattern| Arc::from(evaluate_pattern(pattern, binding)))
                .collect(),
        },
        hrg::Pattern::Literal { identifier } => hrg::Value::Element {
            identifier: identifier.clone(),
        },
        hrg::Pattern::Variable { identifier } => binding
            .get(identifier)
            .cloned()
            .unwrap_or_else(|| panic!("Unknown variable \"{identifier}\".")),
        hrg::Pattern::Wildcard => panic!("Wildcard is not evaluable."),
    }
}

fn evaluate_type_values<'a>(context: &'a Context, type_: &rg::Type<Id>) -> &'a [hrg::Value<Id>] {
    let rg::Type::TypeReference { identifier } = type_ else {
        panic!("Expected TypeReference, got \"{type_}\".");
    };

    let values = context
        .type_values
        .get(identifier)
        .unwrap_or_else(|| panic!("Unresolved TypeReference \"{identifier}\"."));
    assert!(!values.is_empty(), "Expected at least one identifier.");

    values
}

fn serialize_value(value: &hrg::Value<Id>) -> Id {
    match value {
        hrg::Value::Constructor { identifier, args } => Arc::from(format!(
            "{}__{}",
            identifier.to_lowercase(),
            args.iter()
                .map(|arg| serialize_value(arg))
                .collect::<Vec<_>>()
                .join("_")
        )),
        hrg::Value::Element { identifier } => Arc::from(identifier.to_lowercase()),
        hrg::Value::Map { .. } => unimplemented!(),
    }
}

fn translate_automaton_function(
    context: &mut Context,
    automaton_function: &hrg::Function<Id>,
    end_node: Option<&rg::Node<Id>>,
    return_node: Option<&rg::Node<Id>>,
    prefix: &Id,
) {
    for arg in &automaton_function.args {
        // Function arguments are hoisted into global variables, shadowing them
        // if needed. For the sake of easier implementation, the type of a
        // global variable has to match the type of the argument.
        let type_ = translate_type(&arg.type_);
        match context.rg.variables.iter().find(|x| x.identifier == arg.identifier) {
            Some(variable) =>
                assert_eq!(type_, variable.type_, "Argument \"{}\" of function \"{}\" has a different type than an already existing variable ({} != {})", arg.identifier,
          automaton_function.name,
        type_,
        variable.type_
        )
            ,
            None => context.rg.variables.push(rg::Variable {
                span: Span::none(),
                default_value: Arc::from(translate_value(&evaluate_default_value(context, &type_))),
                identifier: arg.identifier.clone(),
                type_
            })
        }
    }

    let next_node = rg::Node::new(Id::from(if context.reuse_functions {
        format!("{}_end", automaton_function.name)
    } else {
        format!("{prefix}{}_end", automaton_function.name)
    }));

    let returns = translate_automaton_statements(
        context,
        &automaton_function.body,
        &[],
        None,
        None,
        end_node,
        rg::Node::new(Id::from(if context.reuse_functions {
            format!("{}_begin", automaton_function.name)
        } else {
            format!("{prefix}{}_begin", automaton_function.name)
        })),
        Some(&next_node),
        &Id::from(format!("{prefix}{}", automaton_function.name)),
        Some(&next_node),
    );

    if returns {
        if let Some(return_node) = return_node {
            context.connect(next_node, return_node.clone(), rg::Label::new_skip(), &[]);
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn translate_automaton_statements(
    context: &mut Context,
    automaton_statements: &[hrg::Statement<Id>],
    bindings: &[rg::Binding<Id>],
    break_node: Option<&rg::Node<Id>>,
    continue_node: Option<&rg::Node<Id>>,
    end_node: Option<&rg::Node<Id>>,
    entry_node: rg::Node<Id>,
    next_node: Option<&rg::Node<Id>>,
    prefix: &Id,
    return_node: Option<&rg::Node<Id>>,
) -> bool {
    let mut current_node = entry_node;
    for automaton_statement in automaton_statements {
        match automaton_statement {
            hrg::Statement::Assignment {
                identifier,
                accessors,
                expression,
            } => {
                let local_node = context.random_node(prefix);
                context.connect(
                    current_node,
                    local_node.clone(),
                    rg::Label::Assignment {
                        lhs: Arc::from(accessors.iter().fold(
                            rg::Expression::new(identifier.clone()),
                            |expression, accessor| rg::Expression::Access {
                                span: Span::none(),
                                lhs: Arc::from(expression),
                                rhs: translate_expression(accessor),
                            },
                        )),
                        rhs: translate_expression(expression),
                    },
                    bindings,
                );
                current_node = local_node;
            }
            hrg::Statement::Branch { arms } => {
                let local_node = context.random_node(prefix);
                for arm in arms {
                    translate_automaton_statements(
                        context,
                        arm,
                        bindings,
                        break_node,
                        continue_node,
                        end_node,
                        current_node.clone(),
                        Some(&local_node),
                        prefix,
                        return_node,
                    );
                }
                current_node = local_node;
            }
            hrg::Statement::Call { identifier, args } => match identifier.as_ref() {
                "break" => {
                    assert!(args.is_empty(), "break() expects no arguments.");
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "break() has to be the last statement."
                    );
                    let Some(break_node) = break_node else {
                        panic!("break() requires break_node.");
                    };

                    current_node
                        .parts
                        .extend(
                            bindings
                                .iter()
                                .map(|(identifier, type_)| rg::NodePart::Binding {
                                    span: Span::none(),
                                    identifier: (*identifier).clone(),
                                    type_: (*type_).clone(),
                                }),
                        );
                    context.connect(current_node, break_node.clone(), rg::Label::new_skip(), &[]);
                    return true;
                }
                "check" => {
                    assert_eq!(args.len(), 1, "check() expects exactly 1 argument.");
                    let local_node = context.random_node(prefix);
                    translate_condition(
                        context,
                        args[0].as_ref(),
                        &current_node,
                        Some(&local_node),
                        None,
                        prefix,
                        bindings,
                    );
                    current_node = local_node;
                }
                "continue" => {
                    assert!(args.is_empty(), "continue() expects no arguments.");
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "continue() has to be the last statement."
                    );
                    let Some(continue_node) = continue_node else {
                        panic!("continue() requires continue_node.");
                    };

                    current_node
                        .parts
                        .extend(
                            bindings
                                .iter()
                                .map(|(identifier, type_)| rg::NodePart::Binding {
                                    span: Span::none(),
                                    identifier: (*identifier).clone(),
                                    type_: (*type_).clone(),
                                }),
                        );
                    context.connect(
                        current_node,
                        continue_node.clone(),
                        rg::Label::new_skip(),
                        &[],
                    );
                    return true;
                }
                "end" => {
                    assert!(args.is_empty(), "end() expects no arguments.");
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "end() has to be the last statement."
                    );
                    let Some(end_node) = end_node else {
                        panic!("end() requires end_node.");
                    };

                    current_node
                        .parts
                        .extend(
                            bindings
                                .iter()
                                .map(|(identifier, type_)| rg::NodePart::Binding {
                                    span: Span::none(),
                                    identifier: (*identifier).clone(),
                                    type_: (*type_).clone(),
                                }),
                        );
                    context.connect(
                        current_node,
                        end_node.clone(),
                        rg::Label::Assignment {
                            lhs: Arc::from(rg::Expression::new(Id::from("player"))),
                            rhs: Arc::from(rg::Expression::new(Id::from("keeper"))),
                        },
                        &[],
                    );
                    return true;
                }
                "return" => {
                    assert!(args.is_empty(), "return() expects no arguments.");
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "return() has to be the last statement."
                    );
                    let Some(return_node) = return_node else {
                        panic!("return() requires return_node.");
                    };

                    current_node
                        .parts
                        .extend(
                            bindings
                                .iter()
                                .map(|(identifier, type_)| rg::NodePart::Binding {
                                    span: Span::none(),
                                    identifier: (*identifier).clone(),
                                    type_: (*type_).clone(),
                                }),
                        );
                    context.connect(
                        current_node,
                        return_node.clone(),
                        rg::Label::new_skip(),
                        &[],
                    );
                    return true;
                }
                _ => {
                    let automaton_function = context
                        .hrg
                        .automaton
                        .iter()
                        .find(|automaton_function| automaton_function.name == *identifier)
                        .unwrap_or_else(|| panic!("Unknown automaton function \"{identifier}\"."))
                        .clone();

                    if context.reuse_functions {
                        let call_id =
                            context.random(&Id::from(format!("{}_call", &automaton_function.name)));
                        let variable = Id::from(format!("{}_return", &automaton_function.name));
                        let type_ = Id::from(format!("{variable}_type"));

                        if context
                            .rg
                            .variables
                            .iter()
                            .any(|x| x.identifier == variable)
                        {
                            let type_declaration = context
                                .rg
                                .typedefs
                                .iter_mut()
                                .find(|x| x.identifier == type_)
                                .unwrap_or_else(|| panic!("Type \"{type_}\" not found."));
                            let rg::Type::Set {
                                ref mut identifiers,
                                ..
                            } = Arc::make_mut(&mut type_declaration.type_)
                            else {
                                panic!("Type \"{variable}\" has invalid type.");
                            };
                            identifiers.push(call_id.clone());
                        } else {
                            context.rg.typedefs.push(rg::Typedef {
                                span: Span::none(),
                                identifier: type_.clone(),
                                type_: Arc::from(rg::Type::Set {
                                    span: Span::none(),
                                    identifiers: vec![call_id.clone()],
                                }),
                            });
                            context.rg.variables.push(rg::Variable {
                                span: Span::none(),
                                default_value: Arc::from(rg::Value::Element {
                                    identifier: call_id.clone(),
                                }),
                                identifier: variable.clone(),
                                type_: Arc::from(rg::Type::new(type_)),
                            });
                        }

                        let call_node = rg::Node::new(call_id.clone());
                        context.connect(
                            current_node,
                            call_node.clone(),
                            rg::Label::new_skip(),
                            bindings,
                        );

                        let set_node = context.random_node(prefix);
                        context.connect(
                            call_node,
                            set_node.clone(),
                            rg::Label::Assignment {
                                lhs: Arc::from(rg::Expression::new(variable.clone())),
                                rhs: Arc::from(rg::Expression::new(call_id.clone())),
                            },
                            bindings,
                        );

                        current_node = set_node;
                        for (index, arg) in args.iter().enumerate() {
                            let arg_node = context.random_node(prefix);
                            context.connect(
                                current_node,
                                arg_node.clone(),
                                rg::Label::Assignment {
                                    lhs: Arc::from(rg::Expression::new(
                                        automaton_function.args[index].identifier.clone(),
                                    )),
                                    rhs: translate_expression(arg),
                                },
                                bindings,
                            );
                            current_node = arg_node;
                        }

                        context.connect(
                            current_node,
                            rg::Node::new(Id::from(format!("{}_begin", &automaton_function.name))),
                            rg::Label::new_skip(),
                            bindings,
                        );

                        let local_node =
                            rg::Node::new(Id::from(format!("{}_return", &automaton_function.name)));
                        if context
                            .translated_functions
                            .insert(automaton_function.name.clone())
                        {
                            translate_automaton_function(
                                context,
                                &automaton_function,
                                end_node,
                                Some(&local_node),
                                &Id::from(""),
                            );
                        } else {
                            context.connect(
                                rg::Node::new(Id::from(format!(
                                    "{}_end",
                                    &automaton_function.name
                                ))),
                                local_node.clone(),
                                rg::Label::new_skip(),
                                bindings,
                            );
                        }

                        current_node = local_node;
                        let intermediate_node = context.random_node(prefix);
                        context.connect(
                            current_node,
                            intermediate_node.clone(),
                            rg::Label::Comparison {
                                lhs: Arc::from(rg::Expression::new(variable)),
                                rhs: Arc::from(rg::Expression::new(call_id)),
                                negated: false,
                            },
                            bindings,
                        );

                        current_node = intermediate_node;
                    } else {
                        let call_id = Id::from(format!("{}_", context.random(prefix)));
                        for (index, arg) in args.iter().enumerate() {
                            let arg_node = context.random_node(&call_id);
                            context.connect(
                                current_node,
                                arg_node.clone(),
                                rg::Label::Assignment {
                                    lhs: Arc::from(rg::Expression::new(
                                        automaton_function.args[index].identifier.clone(),
                                    )),
                                    rhs: translate_expression(arg),
                                },
                                bindings,
                            );
                            current_node = arg_node;
                        }

                        context.connect(
                            current_node,
                            rg::Node::new(Id::from(format!(
                                "{call_id}{}_begin",
                                &automaton_function.name
                            ))),
                            rg::Label::new_skip(),
                            bindings,
                        );

                        let local_node = context.random_node(prefix);
                        translate_automaton_function(
                            context,
                            &automaton_function,
                            end_node,
                            Some(&local_node),
                            &call_id,
                        );
                        current_node = local_node;
                    }
                }
            },
            hrg::Statement::Forall {
                identifier,
                type_,
                body,
            } => {
                let binding = (identifier, &translate_type(type_));

                let mut local_node = context.random_node(prefix);
                local_node.parts.push(rg::NodePart::Binding {
                    span: Span::none(),
                    identifier: binding.0.clone(),
                    type_: binding.1.clone(),
                });
                context.connect(
                    current_node.clone(),
                    local_node.clone(),
                    rg::Label::new_skip(),
                    bindings,
                );
                local_node.parts.pop();

                let mut middle_node = context.random_node(prefix);
                translate_automaton_statements(
                    context,
                    body,
                    bindings
                        .iter()
                        .cloned()
                        .chain(Some(binding))
                        .collect::<Vec<_>>()
                        .as_slice(),
                    break_node,
                    continue_node,
                    end_node,
                    local_node,
                    Some(&middle_node),
                    prefix,
                    return_node,
                );

                let after_node = context.random_node(prefix);
                middle_node.parts.push(rg::NodePart::Binding {
                    span: Span::none(),
                    identifier: binding.0.clone(),
                    type_: binding.1.clone(),
                });
                context.connect(
                    middle_node.clone(),
                    after_node.clone(),
                    rg::Label::new_skip(),
                    bindings,
                );
                middle_node.parts.pop();

                current_node = after_node;
            }
            hrg::Statement::If { expression, body } => {
                let then_node = context.random_node(prefix);
                let else_node = context.random_node(prefix);
                translate_condition(
                    context,
                    expression,
                    &current_node,
                    Some(&then_node),
                    Some(&else_node),
                    prefix,
                    bindings,
                );
                translate_automaton_statements(
                    context,
                    body,
                    bindings,
                    break_node,
                    continue_node,
                    end_node,
                    then_node,
                    Some(&else_node),
                    prefix,
                    return_node,
                );
                current_node = else_node;
            }
            hrg::Statement::Loop { body } => {
                let local_node = context.random_node(prefix);
                translate_automaton_statements(
                    context,
                    body,
                    bindings,
                    Some(&local_node),
                    Some(&current_node.clone()),
                    end_node,
                    current_node.clone(),
                    Some(&current_node),
                    prefix,
                    return_node,
                );
                current_node = local_node;
            }
            hrg::Statement::Tag { symbol } => {
                let local_node = context.random_node(prefix);
                context.connect(
                    current_node,
                    local_node.clone(),
                    rg::Label::Tag {
                        symbol: symbol.clone(),
                    },
                    bindings,
                );
                current_node = local_node;
            }
            hrg::Statement::While { expression, body } => {
                let then_node = context.random_node(prefix);
                let else_node = context.random_node(prefix);
                translate_condition(
                    context,
                    expression,
                    &current_node,
                    Some(&then_node),
                    Some(&else_node),
                    prefix,
                    bindings,
                );
                translate_automaton_statements(
                    context,
                    body,
                    bindings,
                    Some(&else_node),
                    Some(&current_node),
                    end_node,
                    then_node,
                    Some(&current_node),
                    prefix,
                    return_node,
                );
                current_node = else_node;
            }
        }
    }

    if let Some(next_node) = next_node {
        context.connect(
            current_node,
            next_node.clone(),
            rg::Label::new_skip(),
            bindings,
        );
    }

    true
}

fn translate_condition(
    context: &mut Context,
    expression: &hrg::Expression<Id>,
    entry_node: &rg::Node<Id>,
    then_node: Option<&rg::Node<Id>>,
    else_node: Option<&rg::Node<Id>>,
    prefix: &Id,
    bindings: &[rg::Binding<Id>],
) {
    match expression {
        hrg::Expression::BinExpr {
            lhs,
            op: hrg::Binop::And,
            rhs,
        } => {
            let true_node = context.random_node(prefix);
            translate_condition(
                context,
                lhs,
                entry_node,
                Some(&true_node),
                else_node,
                prefix,
                bindings,
            );
            translate_condition(
                context, rhs, &true_node, then_node, else_node, prefix, bindings,
            );
        }
        hrg::Expression::BinExpr {
            lhs,
            op: hrg::Binop::Eq,
            rhs,
        } => {
            if let Some(then_node) = then_node {
                context.connect(
                    entry_node.clone(),
                    then_node.clone(),
                    rg::Label::Comparison {
                        lhs: translate_expression(lhs),
                        rhs: translate_expression(rhs),
                        negated: false,
                    },
                    bindings,
                );
            }
            if let Some(else_node) = else_node {
                context.connect(
                    entry_node.clone(),
                    else_node.clone(),
                    rg::Label::Comparison {
                        lhs: translate_expression(lhs),
                        rhs: translate_expression(rhs),
                        negated: true,
                    },
                    bindings,
                );
            }
        }
        hrg::Expression::BinExpr {
            lhs,
            op: hrg::Binop::Or,
            rhs,
        } => {
            let false_node = context.random_node(prefix);
            translate_condition(
                context,
                lhs,
                entry_node,
                then_node,
                Some(&false_node),
                prefix,
                bindings,
            );
            translate_condition(
                context,
                rhs,
                &false_node,
                then_node,
                else_node,
                prefix,
                bindings,
            );
        }
        hrg::Expression::BinExpr {
            lhs,
            op: hrg::Binop::Ne,
            rhs,
        } => {
            if let Some(then_node) = then_node {
                context.connect(
                    entry_node.clone(),
                    then_node.clone(),
                    rg::Label::Comparison {
                        lhs: translate_expression(lhs),
                        rhs: translate_expression(rhs),
                        negated: true,
                    },
                    bindings,
                );
            }
            if let Some(else_node) = else_node {
                context.connect(
                    entry_node.clone(),
                    else_node.clone(),
                    rg::Label::Comparison {
                        lhs: translate_expression(lhs),
                        rhs: translate_expression(rhs),
                        negated: false,
                    },
                    bindings,
                );
            }
        }
        hrg::Expression::Call { expression, args } => {
            let hrg::Expression::Literal { identifier } = expression.as_ref() else {
                panic!("Call expects a literal.");
            };

            match identifier.as_ref() {
                "not" => {
                    assert_eq!(args.len(), 1, "not() expects exactly 1 argument.");
                    translate_condition(
                        context,
                        args[0].as_ref(),
                        entry_node,
                        else_node,
                        then_node,
                        prefix,
                        bindings,
                    );
                }
                "reachable" => {
                    assert_eq!(args.len(), 1, "reachable() expects exactly 1 argument.");
                    let hrg::Expression::Call { expression, args } = args[0].as_ref() else {
                        panic!("reachable() expects an automaton call.");
                    };
                    let hrg::Expression::Literal { identifier } = expression.as_ref() else {
                        panic!("reachable() expects an automaton call.");
                    };

                    let automaton_name = identifier;
                    let automaton_prefix = context.random(prefix);
                    let automaton_function = context
                        .hrg
                        .automaton
                        .iter()
                        .find(|automaton_function| automaton_function.name == *automaton_name)
                        .unwrap_or_else(|| panic!("Unknown automaton function \"{identifier}\"."))
                        .clone();

                    let call_id =
                        context.random(&Id::from(format!("{}_call", automaton_function.name)));
                    let automaton_start_node = if context.reuse_functions {
                        rg::Node::new(call_id)
                    } else {
                        context.random_node(&automaton_prefix)
                    };

                    let mut automaton_current_node = automaton_start_node.clone();
                    for (index, arg) in args.iter().enumerate() {
                        let arg_node = context.random_node(&automaton_prefix);
                        context.connect(
                            automaton_current_node,
                            arg_node.clone(),
                            rg::Label::Assignment {
                                lhs: Arc::from(rg::Expression::new(
                                    automaton_function.args[index].identifier.clone(),
                                )),
                                rhs: translate_expression(arg),
                            },
                            bindings,
                        );
                        automaton_current_node = arg_node;
                    }

                    context.connect(
                        automaton_current_node,
                        rg::Node::new(Id::from(if context.reuse_functions {
                            format!("{automaton_name}_begin")
                        } else {
                            format!("{automaton_prefix}_{automaton_name}_begin")
                        })),
                        rg::Label::new_skip(),
                        bindings,
                    );

                    let automaton_end_node = rg::Node::new(Id::from(if context.reuse_functions {
                        format!("{automaton_name}_end")
                    } else {
                        format!("{automaton_prefix}_{automaton_name}_end")
                    }));

                    if context.reuse_functions {
                        if context
                            .translated_functions
                            .insert(automaton_function.name.clone())
                        {
                            translate_automaton_function(
                                context,
                                &automaton_function,
                                Some(&automaton_end_node),
                                None,
                                &Id::from(""),
                            );
                        }
                    } else {
                        translate_automaton_function(
                            context,
                            &automaton_function,
                            Some(&automaton_end_node),
                            None,
                            &Id::from(format!("{automaton_prefix}_")),
                        );
                    }

                    if let Some(then_node) = then_node {
                        context.connect(
                            entry_node.clone(),
                            then_node.clone(),
                            rg::Label::Reachability {
                                span: Span::none(),
                                lhs: automaton_start_node.clone(),
                                rhs: automaton_end_node.clone(),
                                negated: false,
                            },
                            bindings,
                        );
                    }

                    if let Some(else_node) = else_node {
                        context.connect(
                            entry_node.clone(),
                            else_node.clone(),
                            rg::Label::Reachability {
                                span: Span::none(),
                                lhs: automaton_start_node,
                                rhs: automaton_end_node,
                                negated: true,
                            },
                            bindings,
                        );
                    }
                }
                _ => unimplemented!("Unknown condition function \"{identifier}\"."),
            }
        }
        _ => unimplemented!(),
    }
}

fn translate_domains(context: &mut Context) {
    for domain in &context.hrg.domains {
        if context.type_values.contains_key(&domain.identifier) {
            // TODO: Errors!
            panic!("Duplicated domain \"{}\".", domain.identifier);
        }

        let domain_elements = translate_domain_elements(context, &domain.elements);
        context.rg.typedefs.push(rg::Typedef {
            span: Span::none(),
            identifier: domain.identifier.clone(),
            type_: Arc::from(rg::Type::Set {
                span: Span::none(),
                identifiers: domain_elements.iter().map(serialize_value).collect(),
            }),
        });
        context
            .type_values
            .insert(domain.identifier.clone(), domain_elements);
    }
}

fn translate_domain_element(
    context: &Context,
    domain_element: &hrg::DomainElement<Id>,
) -> Vec<hrg::Value<Id>> {
    match domain_element {
        hrg::DomainElement::Generator {
            identifier,
            args,
            values,
        } => args
            .iter()
            .map(|identifier| {
                let domain_value = values
                    .iter()
                    .find(|x| x.identifier() == identifier)
                    .unwrap_or_else(|| panic!("Missing values for \"{identifier}\"."));
                let identifiers = match domain_value {
                    hrg::DomainValue::Range { min, max, .. } => (*min..=*max)
                        .map(|index: usize| Id::from(index.to_string()))
                        .collect(),
                    hrg::DomainValue::Set { elements, .. } => elements.clone(),
                };

                identifiers
                    .into_iter()
                    .map(|identifier| Arc::from(hrg::Value::Element { identifier }))
                    .collect()
            })
            .fold(vec![vec![]], cartesian)
            .into_iter()
            .map(|args| hrg::Value::Constructor {
                identifier: identifier.clone(),
                args,
            })
            .collect(),
        hrg::DomainElement::Literal { identifier } => context
            .hrg
            .domains
            .iter()
            .find(|x| x.identifier == *identifier)
            .map_or_else(
                || {
                    vec![hrg::Value::Element {
                        identifier: identifier.clone(),
                    }]
                },
                |referenced_domain| translate_domain_elements(context, &referenced_domain.elements),
            ),
    }
}

fn translate_domain_elements(
    context: &Context,
    domain_elements: &[hrg::DomainElement<Id>],
) -> Vec<hrg::Value<Id>> {
    domain_elements
        .iter()
        .flat_map(|domain_element| translate_domain_element(context, domain_element))
        .collect()
}

fn translate_expression(expression: &hrg::Expression<Id>) -> Arc<rg::Expression<Id>> {
    match expression {
        hrg::Expression::Access { lhs, rhs } => Arc::from(rg::Expression::Access {
            span: Span::none(),
            lhs: translate_expression(lhs),
            rhs: translate_expression(rhs),
        }),
        hrg::Expression::Call { expression, args } => {
            args.iter()
                .fold(translate_expression(expression), |expression, arg| {
                    Arc::from(rg::Expression::Access {
                        span: Span::none(),
                        lhs: expression,
                        rhs: translate_expression(arg),
                    })
                })
        }
        hrg::Expression::Constructor { .. } => Arc::from(rg::Expression::new(serialize_value(
            &evaluate_expression(expression, &BTreeMap::new()),
        ))),
        hrg::Expression::Literal { identifier } => {
            Arc::from(rg::Expression::new(identifier.clone()))
        }
        _ => unimplemented!(),
    }
}

fn translate_function(
    context: &Context,
    function_declaration: &hrg::FunctionDeclaration<Id>,
) -> rg::Constant<Id> {
    let first_case = function_declaration
        .cases
        .first()
        .expect("All functions require at least one case.");
    function_declaration.cases.iter().for_each(|function_case| {
        assert_eq!(
            function_declaration.identifier, function_case.identifier,
            "All function cases should have the same identifier as function declaration."
        );
        assert_eq!(
            first_case.args.len(),
            function_case.args.len(),
            "All function cases should have the same number of arguments."
        );
    });

    let type_ = translate_type(&function_declaration.type_);
    assert!(type_.is_arrow(), "Function is expected to have Arrow type.");

    let value = Arc::from(translate_function_layer(
        context,
        function_declaration,
        first_case,
        &type_,
        &[],
    ));
    rg::Constant {
        span: Span::none(),
        identifier: function_declaration.identifier.clone(),
        type_,
        value,
    }
}

fn translate_function_layer(
    context: &Context,
    function_declaration: &hrg::FunctionDeclaration<Id>,
    first_case: &hrg::FunctionCase<Id>,
    type_: &rg::Type<Id>,
    values: &[hrg::Value<Id>],
) -> rg::Value<Id> {
    if first_case.args.len() > values.len() {
        if let rg::Type::Arrow { lhs, rhs } = type_ {
            return construct_map(
                evaluate_type_values(context, lhs)
                    .iter()
                    .map(|value| {
                        let mut values = values.to_owned();
                        values.push(value.clone());
                        rg::ValueEntry {
                            span: Span::none(),
                            identifier: Some(serialize_value(value)),
                            value: Arc::from(translate_function_layer(
                                context,
                                function_declaration,
                                first_case,
                                rhs,
                                &values,
                            )),
                        }
                    })
                    .collect(),
            );
        }
    }

    let arm = function_declaration
        .cases
        .iter()
        .find_map(|function_case| {
            values
                .iter()
                .enumerate()
                .try_fold(BTreeMap::new(), |mut binding, (index, value)| {
                    evaluate_binding(&function_case.args[index], value).map(|subbinding| {
                        binding.extend(subbinding);
                        binding
                    })
                })
                .map(|binding| (binding, function_case))
        })
        .unwrap_or_else(|| {
            panic!(
                "No case for {}({}).",
                function_declaration.identifier,
                values
                    .iter()
                    .map(serialize_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        });

    let value = evaluate_expression(&arm.1.body, &arm.0);
    rg::Value::Element {
        identifier: serialize_value(&value),
    }
}

fn translate_functions(context: &mut Context) {
    for function_declaration in &context.hrg.functions {
        context
            .rg
            .constants
            .push(translate_function(context, function_declaration));
    }
}

fn translate_type(type_: &hrg::Type<Id>) -> Arc<rg::Type<Id>> {
    Arc::from(match type_ {
        hrg::Type::Function { lhs, rhs } => rg::Type::Arrow {
            lhs: translate_type(lhs),
            rhs: translate_type(rhs),
        },
        hrg::Type::Name { identifier } => rg::Type::TypeReference {
            identifier: identifier.clone(),
        },
    })
}

fn translate_value(value: &hrg::Value<Id>) -> rg::Value<Id> {
    match value {
        hrg::Value::Constructor { .. } => rg::Value::Element {
            identifier: serialize_value(value),
        },
        hrg::Value::Element { identifier } => rg::Value::Element {
            identifier: identifier.clone(),
        },
        hrg::Value::Map { entries } => {
            assert!(!entries.is_empty(), "At least one entry is required.");
            construct_map(
                entries
                    .iter()
                    .map(|entry| rg::ValueEntry {
                        span: Span::none(),
                        identifier: Some(serialize_value(&entry.key)),
                        value: Arc::from(translate_value(&entry.value)),
                    })
                    .collect(),
            )
        }
    }
}

fn translate_variable(
    context: &Context,
    variable_declaration: &hrg::VariableDeclaration<Id>,
) -> rg::Variable<Id> {
    let type_ = translate_type(&variable_declaration.type_);
    rg::Variable {
        span: Span::none(),
        default_value: Arc::from(translate_variable_default_value(
            context,
            &variable_declaration.default_value,
            &type_,
        )),
        identifier: variable_declaration.identifier.clone(),
        type_,
    }
}

fn translate_variable_default_value(
    context: &Context,
    default_value: &Option<Arc<hrg::Expression<Id>>>,
    type_: &rg::Type<Id>,
) -> rg::Value<Id> {
    match default_value.as_deref() {
        // If there's none, build one from the type.
        None => translate_value(&evaluate_default_value(context, type_)),

        // If it's a map with a wildcard pattern, set it as the default value.
        Some(hrg::Expression::Map {
            pattern,
            expression,
            ..
        }) if **pattern == hrg::Pattern::Wildcard => rg::Value::Map {
            span: Span::none(),
            entries: vec![rg::ValueEntry {
                span: Span::none(),
                identifier: None,
                value: Arc::from(translate_value(&evaluate_expression(
                    expression,
                    &BTreeMap::new(),
                ))),
            }],
        },

        // Build the map.
        // TODO: Check whether map domains match type domains.
        Some(default_value) => {
            translate_value(&evaluate_expression(default_value, &BTreeMap::new()))
        }
    }
}

fn translate_variables(context: &mut Context) {
    for variable_declaration in &context.hrg.variables {
        context
            .rg
            .variables
            .push(translate_variable(context, variable_declaration));
    }
}
