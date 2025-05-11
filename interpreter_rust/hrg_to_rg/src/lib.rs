use hrg::ast as hrg;
use natord::compare;
use rg::ast as rg;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::cartesian::cartesian;
use utils::position::Span;

type Id = Arc<str>;

struct Context {
    counters: BTreeMap<Id, usize>,
    hrg: hrg::Game<Id>,
    rg: rg::Game<Id>,
    translated_functions: BTreeSet<Id>,
    type_values: BTreeMap<Id, Vec<hrg::Value<Id>>>,
}

impl Context {
    fn connect(&mut self, lhs: rg::Node<Id>, rhs: rg::Node<Id>, label: rg::Label<Id>) {
        self.rg
            .edges
            .push(Arc::from(rg::Edge::new(lhs, rhs, label)));
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

pub fn hrg_to_rg(hrg: hrg::Game<Id>) -> Result<rg::Game<Id>, hrg::Error<Id>> {
    let mut context = Context {
        counters: BTreeMap::new(),
        hrg,
        rg: rg::Game::default(),
        translated_functions: BTreeSet::new(),
        type_values: BTreeMap::new(),
    };

    translate_domains(&mut context);
    translate_functions(&mut context)?;
    translate_variables(&mut context)?;

    context.rg.edges.push(Arc::from(rg::Edge {
        span: Span::none(),
        label: rg::Label::new_skip(),
        lhs: rg::Node::new(Id::from("begin")),
        rhs: rg::Node::new(Id::from("rules_begin")),
    }));

    let rules = context
        .hrg
        .automaton
        .iter()
        .find(|automaton_function| automaton_function.name.as_ref() == "rules")
        .ok_or_else(|| hrg::Error::UnknownAutomatonFunction {
            identifier: Arc::from("rules"),
        })?
        .clone();

    translate_automaton_function(
        &mut context,
        &rules,
        Some(&rg::Node::new(Id::from("end"))),
        Some(&rg::Node::new(Id::from("end"))),
        &Id::from(""),
    )?;

    Ok(context.rg)
}

fn check_arguments_length(
    identifier: Id,
    expected: usize,
    got: usize,
) -> Result<(), hrg::Error<Id>> {
    (expected == got)
        .then_some(())
        .ok_or(hrg::Error::IncorrectNumberOfArguments {
            identifier,
            expected,
            got,
        })
}

fn compare_identifiers(lhs: &Id, rhs: &Id) -> Ordering {
    compare(lhs, rhs)
}

fn compare_values(lhs: &hrg::Value<Id>, rhs: &hrg::Value<Id>) -> Result<Ordering, hrg::Error<Id>> {
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
        ) => Ok(compare_identifiers(li, ri)
            .then(la.len().cmp(&ra.len()))
            .then({
                la.iter()
                    .zip(ra)
                    .try_fold(Ordering::Equal, |ord, (lhs, rhs)| {
                        Ok(ord.then(compare_values(lhs, rhs)?))
                    })?
            })),
        (hrg::Value::Element { identifier: li }, hrg::Value::Element { identifier: ri }) => {
            Ok(compare_identifiers(li, ri))
        }
        (hrg::Value::Map { .. }, hrg::Value::Map { .. }) => Err(hrg::Error::NotImplemented {
            message: "compare_values for Value::Map",
        }),
        _ => Err(hrg::Error::IncomparableValues {
            lhs: lhs.clone(),
            rhs: rhs.clone(),
        }),
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
    context: &Context,
    expression: &hrg::Expression<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> Result<bool, hrg::Error<Id>> {
    let hrg::Expression::BinExpr { lhs, op, rhs } = expression else {
        return Err(hrg::Error::InvalidCondition {
            expression: expression.clone(),
        });
    };

    Ok(match op {
        hrg::Binop::And => {
            evaluate_condition(context, lhs, binding)? && evaluate_condition(context, rhs, binding)?
        }
        hrg::Binop::Or => {
            evaluate_condition(context, lhs, binding)? || evaluate_condition(context, rhs, binding)?
        }
        hrg::Binop::Eq
        | hrg::Binop::Gt
        | hrg::Binop::Gte
        | hrg::Binop::Lt
        | hrg::Binop::Lte
        | hrg::Binop::Ne => {
            let lhs = evaluate_expression(context, lhs, binding)?;
            let rhs = evaluate_expression(context, rhs, binding)?;
            let ord = compare_values(&lhs, &rhs)?;
            match op {
                hrg::Binop::Eq => ord.is_eq(),
                hrg::Binop::Gt => ord.is_gt(),
                hrg::Binop::Gte => ord.is_gt() || ord.is_eq(),
                hrg::Binop::Lt => ord.is_lt(),
                hrg::Binop::Lte => ord.is_lt() || ord.is_eq(),
                hrg::Binop::Ne => !ord.is_eq(),
                _ => {
                    return Err(hrg::Error::InvalidCondition {
                        expression: expression.clone(),
                    })
                }
            }
        }
        _ => {
            return Err(hrg::Error::InvalidCondition {
                expression: expression.clone(),
            })
        }
    })
}

fn evaluate_default_value(
    context: &Context,
    type_: &rg::Type<Id>,
) -> Result<hrg::Value<Id>, hrg::Error<Id>> {
    Ok(match type_ {
        // NOTE: Is this even correct?
        rg::Type::Arrow { lhs, rhs } => hrg::Value::Map {
            default_value: None,
            entries: evaluate_type_values(context, lhs)?
                .iter()
                .map(|value| {
                    Ok(hrg::ValueMapEntry {
                        key: Arc::from(value.clone()),
                        value: Arc::from(evaluate_default_value(context, rhs)?),
                    })
                })
                .collect::<Result<_, _>>()?,
        },
        rg::Type::Set { .. } => unimplemented!(),
        rg::Type::TypeReference { .. } => evaluate_type_values(context, type_)?
            .first()
            .unwrap()
            .clone(),
    })
}

fn evaluate_domain_values(
    domain_values: &[hrg::DomainValue<Id>],
) -> Result<Vec<BTreeMap<Id, hrg::Value<Id>>>, hrg::Error<Id>> {
    // No `where ...` part is present, so no bindings are generated.
    if domain_values.is_empty() {
        return Ok(vec![BTreeMap::default()]);
    }

    for domain_value in &domain_values[1..] {
        if domain_value.identifier() == domain_values.first().unwrap().identifier() {
            return Err(hrg::Error::DuplicatedDomainValue {
                identifier: domain_value.identifier().clone(),
            });
        }
    }

    Ok(domain_values
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
        .collect())
}

fn evaluate_expression(
    context: &Context,
    expression: &hrg::Expression<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> Result<hrg::Value<Id>, hrg::Error<Id>> {
    Ok(match expression {
        hrg::Expression::BinExpr { lhs, op, rhs }
            if matches!(op, hrg::Binop::Add | hrg::Binop::Mod | hrg::Binop::Sub) =>
        {
            let lhs: i32 = evaluate_expression_identifier(context, lhs, binding)?
                .parse()
                .unwrap();
            let rhs: i32 = evaluate_expression_identifier(context, rhs, binding)?
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
        hrg::Expression::Call { expression, args }
            if matches!(expression.as_ref(), hrg::Expression::Literal { .. }) =>
        {
            let hrg::Expression::Literal { identifier } = expression.as_ref() else {
                unreachable!();
            };

            let function_declaration = context
                .hrg
                .functions
                .iter()
                .find(|function| function.identifier == *identifier)
                .ok_or_else(|| hrg::Error::UnknownFunction {
                    identifier: Arc::from("rules"),
                })?;

            let args = args
                .iter()
                .map(|arg| evaluate_expression(context, arg, binding))
                .collect::<Result<Vec<_>, _>>()?;

            evaluate_expression_call(context, function_declaration, &args[..])?
        }
        hrg::Expression::Constructor { identifier, args } => hrg::Value::Constructor {
            identifier: identifier.clone(),
            args: args
                .iter()
                .map(|arg| Ok(Arc::from(evaluate_expression(context, arg, binding)?)))
                .collect::<Result<_, _>>()?,
        },
        hrg::Expression::If { cond, then, else_ } => {
            if evaluate_condition(context, cond, binding)? {
                evaluate_expression(context, then, binding)?
            } else {
                evaluate_expression(context, else_, binding)?
            }
        }
        hrg::Expression::Literal { identifier } => binding.get(identifier).cloned().map_or_else(
            || {
                let identifier = identifier.clone();
                if hrg::Pattern::is_literal(&identifier) {
                    Ok(hrg::Value::Element { identifier })
                } else {
                    Err(hrg::Error::UnknownVariable { identifier })
                }
            },
            Ok,
        )?,
        hrg::Expression::Map {
            default_value,
            parts,
        } => hrg::Value::Map {
            default_value: match default_value {
                None => None,
                Some(default_value) => Some(Arc::from(evaluate_expression(
                    context,
                    default_value,
                    binding,
                )?)),
            },
            entries: parts
                .iter()
                .map(|part| evaluate_expression_map_part(context, part, binding))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .try_fold(vec![], |mut xs: Vec<hrg::ValueMapEntry<_>>, ys| {
                    for y in ys {
                        if let Err(index) = xs.binary_search_by(|x| x.key.cmp(&y.key)) {
                            xs.insert(index, y);
                        } else {
                            let key = Arc::unwrap_or_clone(y.key);
                            return Err(hrg::Error::DuplicatedMapKey { key });
                        }
                    }

                    Ok(xs)
                })?,
        },
        _ => unimplemented!(),
    })
}

fn evaluate_expression_call(
    context: &Context,
    function_declaration: &hrg::FunctionDeclaration<Id>,
    values: &[hrg::Value<Id>],
) -> Result<hrg::Value<Id>, hrg::Error<Id>> {
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
        .ok_or_else(|| hrg::Error::FunctionCaseNotCovered {
            identifier: function_declaration.identifier.clone(),
            args: values.to_owned(),
        })?;

    evaluate_expression(context, &arm.1.body, &arm.0)
}

fn evaluate_expression_identifier(
    context: &Context,
    expression: &hrg::Expression<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> Result<Id, hrg::Error<Id>> {
    match evaluate_expression(context, expression, binding)? {
        hrg::Value::Element { identifier } => Ok(identifier.clone()),
        _ => panic!("Expected ValueElement."),
    }
}

fn evaluate_expression_map_part(
    context: &Context,
    part: &hrg::ExpressionMapPart<Id>,
    binding: &BTreeMap<Id, hrg::Value<Id>>,
) -> Result<Vec<hrg::ValueMapEntry<Id>>, hrg::Error<Id>> {
    evaluate_domain_values(&part.domains)?
        .into_iter()
        .map(|subbinding| {
            let mut binding = binding.clone();
            binding.extend(subbinding);
            binding
        })
        .map(|binding| {
            Ok(hrg::ValueMapEntry {
                key: Arc::from(evaluate_pattern(&part.pattern, &binding)),
                value: Arc::from(evaluate_expression(context, &part.expression, &binding)?),
            })
        })
        .collect()
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

fn evaluate_type_values<'a>(
    context: &'a Context,
    type_: &rg::Type<Id>,
) -> Result<&'a [hrg::Value<Id>], hrg::Error<Id>> {
    let rg::Type::TypeReference { identifier } = type_ else {
        panic!("Expected TypeReference, got \"{type_}\".");
    };

    let values = context
        .type_values
        .get(identifier)
        .ok_or_else(|| hrg::Error::UnknownType {
            identifier: identifier.clone(),
        })?;
    assert!(!values.is_empty(), "Expected at least one identifier.");

    Ok(values)
}

fn serialize_value(value: &hrg::Value<Id>) -> Id {
    match value {
        hrg::Value::Constructor { identifier, args } => Arc::from(format!(
            "{identifier}__{}",
            args.iter()
                .map(|arg| serialize_value(arg))
                .collect::<Vec<_>>()
                .join("_")
        )),
        hrg::Value::Element { identifier } => identifier.clone(),
        hrg::Value::Map { .. } => unimplemented!(),
    }
}

fn translate_automaton_function(
    context: &mut Context,
    automaton_function: &hrg::Function<Id>,
    end_node: Option<&rg::Node<Id>>,
    return_node: Option<&rg::Node<Id>>,
    prefix: &Id,
) -> Result<(), hrg::Error<Id>> {
    for (index, arg) in automaton_function.args.iter().enumerate() {
        // Function arguments are hoisted into global variables, shadowing them
        // if needed. For the sake of easier implementation, the type of a
        // global variable has to match the type of the argument.
        let type_ = translate_type(&arg.type_);
        let identifier = automaton_function.nth_arg_variable(index);
        match context.rg.variables.iter().find(|x| x.identifier == identifier) {
            Some(variable) =>
                assert_eq!(
                    type_,
                    variable.type_,
                    "Argument \"{}\" of function \"{}\" (stored in \"{identifier}\") has a different type than an already existing variable ({type_} != {})",
                    arg.identifier,
                    automaton_function.name,
                    variable.type_
                ),
            None => context.rg.variables.push(rg::Variable {
                span: Span::none(),
                default_value: translate_value(&evaluate_default_value(context, &type_)?)?,
                identifier,
                type_
            })
        }
    }

    let next_node = rg::Node::new(Id::from(if automaton_function.reusable {
        format!("{}_end", automaton_function.name)
    } else {
        format!("{prefix}{}_end", automaton_function.name)
    }));

    let returns = translate_automaton_statements(
        context,
        &automaton_function.body,
        None,
        None,
        end_node,
        rg::Node::new(Id::from(if automaton_function.reusable {
            format!("{}_begin", automaton_function.name)
        } else {
            format!("{prefix}{}_begin", automaton_function.name)
        })),
        Some(&next_node),
        &Id::from(format!("{prefix}{}", automaton_function.name)),
        Some(&next_node),
        Some(automaton_function),
    )?;

    if returns {
        if let Some(return_node) = return_node {
            context.connect(next_node, return_node.clone(), rg::Label::new_skip());
        }
    }

    Ok(())
}

#[expect(clippy::too_many_arguments)]
fn translate_automaton_statements(
    context: &mut Context,
    automaton_statements: &[hrg::Statement<Id>],
    break_node: Option<&rg::Node<Id>>,
    continue_node: Option<&rg::Node<Id>>,
    end_node: Option<&rg::Node<Id>>,
    entry_node: rg::Node<Id>,
    next_node: Option<&rg::Node<Id>>,
    prefix: &Id,
    return_node: Option<&rg::Node<Id>>,
    automaton_function: Option<&hrg::Function<Id>>,
) -> Result<bool, hrg::Error<Id>> {
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
                        lhs: Arc::from(accessors.iter().try_fold(
                            rg::Expression::new(identifier.clone()),
                            |expression, accessor| {
                                Ok(rg::Expression::Access {
                                    span: Span::none(),
                                    lhs: Arc::from(expression),
                                    rhs: translate_expression(
                                        context,
                                        accessor,
                                        automaton_function,
                                    )?,
                                })
                            },
                        )?),
                        rhs: translate_expression(context, expression, automaton_function)?,
                    },
                );
                current_node = local_node;
            }
            hrg::Statement::AssignmentAny {
                identifier,
                accessors,
                type_,
            } => {
                let local_node = context.random_node(prefix);
                context.connect(
                    current_node,
                    local_node.clone(),
                    rg::Label::AssignmentAny {
                        lhs: Arc::from(accessors.iter().try_fold(
                            rg::Expression::new(identifier.clone()),
                            |expression, accessor| {
                                Ok(rg::Expression::Access {
                                    span: Span::none(),
                                    lhs: Arc::from(expression),
                                    rhs: translate_expression(
                                        context,
                                        accessor,
                                        automaton_function,
                                    )?,
                                })
                            },
                        )?),
                        rhs: translate_type(type_),
                    },
                );
                current_node = local_node;
            }
            hrg::Statement::Branch { arms } => {
                let local_node = context.random_node(prefix);
                for arm in arms {
                    translate_automaton_statements(
                        context,
                        arm,
                        break_node,
                        continue_node,
                        end_node,
                        current_node.clone(),
                        Some(&local_node),
                        prefix,
                        return_node,
                        automaton_function,
                    )?;
                }
                current_node = local_node;
            }
            hrg::Statement::BranchVar {
                identifier,
                type_,
                body,
            } => {
                let local_node = context.random_node(prefix);
                for value in translate_type(type_).values(&context.rg).unwrap() {
                    let mut body = body.clone();
                    for statement in &mut body {
                        statement.substitute_var(identifier, &value)?;
                    }

                    translate_automaton_statements(
                        context,
                        &body,
                        break_node,
                        continue_node,
                        end_node,
                        current_node.clone(),
                        Some(&local_node),
                        prefix,
                        return_node,
                        automaton_function,
                    )?;
                }
                current_node = local_node;
            }
            hrg::Statement::Call { identifier, args } => match identifier.as_ref() {
                "break" => {
                    check_arguments_length(identifier.clone(), 0, args.len())?;
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "break() has to be the last statement."
                    );
                    let Some(break_node) = break_node else {
                        panic!("break() requires break_node.");
                    };

                    context.connect(current_node, break_node.clone(), rg::Label::new_skip());
                    return Ok(true);
                }
                "check" => {
                    check_arguments_length(identifier.clone(), 1, args.len())?;
                    let local_node = context.random_node(prefix);
                    translate_condition(
                        context,
                        args[0].as_ref(),
                        &current_node,
                        Some(&local_node),
                        None,
                        prefix,
                        automaton_function,
                    )?;
                    current_node = local_node;
                }
                "continue" => {
                    check_arguments_length(identifier.clone(), 0, args.len())?;
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "continue() has to be the last statement."
                    );
                    let Some(continue_node) = continue_node else {
                        panic!("continue() requires continue_node.");
                    };

                    context.connect(current_node, continue_node.clone(), rg::Label::new_skip());
                    return Ok(true);
                }
                "end" => {
                    check_arguments_length(identifier.clone(), 0, args.len())?;
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "end() has to be the last statement."
                    );
                    let Some(end_node) = end_node else {
                        panic!("end() requires end_node.");
                    };

                    context.connect(
                        current_node,
                        end_node.clone(),
                        rg::Label::Assignment {
                            lhs: Arc::from(rg::Expression::new(Id::from("player"))),
                            rhs: Arc::from(rg::Expression::new(Id::from("keeper"))),
                        },
                    );
                    return Ok(true);
                }
                "return" => {
                    check_arguments_length(identifier.clone(), 0, args.len())?;
                    assert_eq!(
                        Some(automaton_statement),
                        automaton_statements.last(),
                        "return() has to be the last statement."
                    );
                    let Some(return_node) = return_node else {
                        panic!("return() requires return_node.");
                    };

                    context.connect(current_node, return_node.clone(), rg::Label::new_skip());
                    return Ok(true);
                }
                _ => {
                    let called_automaton_function = context
                        .hrg
                        .automaton
                        .iter()
                        .find(|automaton_function| automaton_function.name == *identifier)
                        .unwrap_or_else(|| panic!("Unknown automaton function \"{identifier}\"."))
                        .clone();

                    check_arguments_length(
                        called_automaton_function.name.clone(),
                        called_automaton_function.args.len(),
                        args.len(),
                    )?;

                    if called_automaton_function.reusable {
                        let call_id = context.random(&Id::from(format!(
                            "{}_call",
                            &called_automaton_function.name
                        )));
                        let variable =
                            Id::from(format!("{}_return", &called_automaton_function.name));
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
                        context.connect(current_node, call_node.clone(), rg::Label::new_skip());

                        let set_node = context.random_node(prefix);
                        context.connect(
                            call_node,
                            set_node.clone(),
                            rg::Label::Assignment {
                                lhs: Arc::from(rg::Expression::new(variable.clone())),
                                rhs: Arc::from(rg::Expression::new(call_id.clone())),
                            },
                        );

                        current_node = set_node;
                        for (index, arg) in args.iter().enumerate() {
                            let arg_node = context.random_node(prefix);
                            context.connect(
                                current_node,
                                arg_node.clone(),
                                rg::Label::Assignment {
                                    lhs: Arc::from(rg::Expression::new(
                                        called_automaton_function.nth_arg_variable(index),
                                    )),
                                    rhs: translate_expression(context, arg, automaton_function)?,
                                },
                            );
                            current_node = arg_node;
                        }

                        context.connect(
                            current_node,
                            rg::Node::new(Id::from(format!(
                                "{}_begin",
                                &called_automaton_function.name
                            ))),
                            rg::Label::new_skip(),
                        );

                        let local_node = rg::Node::new(Id::from(format!(
                            "{}_return",
                            &called_automaton_function.name
                        )));
                        if context
                            .translated_functions
                            .insert(called_automaton_function.name.clone())
                        {
                            translate_automaton_function(
                                context,
                                &called_automaton_function,
                                end_node,
                                Some(&local_node),
                                &Id::from(""),
                            )?;
                        } else {
                            context.connect(
                                rg::Node::new(Id::from(format!(
                                    "{}_end",
                                    &called_automaton_function.name
                                ))),
                                local_node.clone(),
                                rg::Label::new_skip(),
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
                                        called_automaton_function.nth_arg_variable(index),
                                    )),
                                    rhs: translate_expression(context, arg, automaton_function)?,
                                },
                            );
                            current_node = arg_node;
                        }

                        context.connect(
                            current_node,
                            rg::Node::new(Id::from(format!(
                                "{call_id}{}_begin",
                                &called_automaton_function.name
                            ))),
                            rg::Label::new_skip(),
                        );

                        let local_node = context.random_node(prefix);
                        translate_automaton_function(
                            context,
                            &called_automaton_function,
                            end_node,
                            Some(&local_node),
                            &call_id,
                        )?;
                        current_node = local_node;
                    }
                }
            },
            hrg::Statement::If {
                expression,
                then,
                else_: None,
            } => {
                let then_node = context.random_node(prefix);
                let else_node = context.random_node(prefix);
                translate_condition(
                    context,
                    expression,
                    &current_node,
                    Some(&then_node),
                    Some(&else_node),
                    prefix,
                    automaton_function,
                )?;
                translate_automaton_statements(
                    context,
                    then,
                    break_node,
                    continue_node,
                    end_node,
                    then_node,
                    Some(&else_node),
                    prefix,
                    return_node,
                    automaton_function,
                )?;
                current_node = else_node;
            }
            hrg::Statement::If {
                expression,
                then: body,
                else_: Some(else_),
            } => {
                let then_node = context.random_node(prefix);
                let else_node = context.random_node(prefix);
                let next_node = context.random_node(prefix);
                translate_condition(
                    context,
                    expression,
                    &current_node,
                    Some(&then_node),
                    Some(&else_node),
                    prefix,
                    automaton_function,
                )?;
                translate_automaton_statements(
                    context,
                    body,
                    break_node,
                    continue_node,
                    end_node,
                    then_node,
                    Some(&next_node),
                    prefix,
                    return_node,
                    automaton_function,
                )?;
                translate_automaton_statements(
                    context,
                    else_,
                    break_node,
                    continue_node,
                    end_node,
                    else_node,
                    Some(&next_node),
                    prefix,
                    return_node,
                    automaton_function,
                )?;
                current_node = next_node;
            }
            hrg::Statement::Loop { body } => {
                let loop_init = context.random_node(prefix);
                context.connect(current_node, loop_init.clone(), rg::Label::new_skip());

                let loop_end = context.random_node(prefix);
                translate_automaton_statements(
                    context,
                    body,
                    Some(&loop_end),
                    Some(&loop_init.clone()),
                    end_node,
                    loop_init.clone(),
                    Some(&loop_init),
                    prefix,
                    return_node,
                    automaton_function,
                )?;
                current_node = loop_end;
            }
            hrg::Statement::Repeat { count, body } => {
                let repeat_end = context.random_node(prefix);
                for _ in 0..*count {
                    let local_node = context.random_node(prefix);
                    translate_automaton_statements(
                        context,
                        body,
                        Some(&repeat_end),
                        Some(&local_node),
                        end_node,
                        current_node.clone(),
                        Some(&local_node),
                        prefix,
                        return_node,
                        automaton_function,
                    )?;
                    current_node = local_node;
                }
                context.connect(current_node, repeat_end.clone(), rg::Label::new_skip());
                current_node = repeat_end;
            }
            hrg::Statement::Tag { symbol } => {
                // By convention, all tags starting with `_` are artificial.
                if symbol.starts_with("_") {
                    context.rg.add_pragma(rg::Pragma::ArtificialTag {
                        span: Span::none(),
                        tags: vec![symbol.clone()],
                    });
                }

                let local_node = context.random_node(prefix);
                context.connect(
                    current_node,
                    local_node.clone(),
                    rg::Label::Tag {
                        symbol: symbol.clone(),
                    },
                );
                current_node = local_node;
            }
            hrg::Statement::TagVariable { identifier } => {
                let local_node = context.random_node(prefix);
                context.connect(
                    current_node,
                    local_node.clone(),
                    rg::Label::TagVariable {
                        identifier: identifier.clone(),
                    },
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
                    automaton_function,
                )?;
                translate_automaton_statements(
                    context,
                    body,
                    Some(&else_node),
                    Some(&current_node),
                    end_node,
                    then_node,
                    Some(&current_node),
                    prefix,
                    return_node,
                    automaton_function,
                )?;
                current_node = else_node;
            }
        }
    }

    if let Some(next_node) = next_node {
        context.connect(current_node, next_node.clone(), rg::Label::new_skip());
    }

    Ok(true)
}

fn translate_condition(
    context: &mut Context,
    expression: &hrg::Expression<Id>,
    entry_node: &rg::Node<Id>,
    then_node: Option<&rg::Node<Id>>,
    else_node: Option<&rg::Node<Id>>,
    prefix: &Id,
    automaton_function: Option<&hrg::Function<Id>>,
) -> Result<(), hrg::Error<Id>> {
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
                automaton_function,
            )?;
            translate_condition(
                context,
                rhs,
                &true_node,
                then_node,
                else_node,
                prefix,
                automaton_function,
            )?;
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
                        lhs: translate_expression(context, lhs, automaton_function)?,
                        rhs: translate_expression(context, rhs, automaton_function)?,
                        negated: false,
                    },
                );
            }
            if let Some(else_node) = else_node {
                context.connect(
                    entry_node.clone(),
                    else_node.clone(),
                    rg::Label::Comparison {
                        lhs: translate_expression(context, lhs, automaton_function)?,
                        rhs: translate_expression(context, rhs, automaton_function)?,
                        negated: true,
                    },
                );
            }
        }
        hrg::Expression::BinExpr {
            lhs,
            op: hrg::Binop::Or,
            rhs,
        } if else_node.is_none() => {
            translate_condition(
                context,
                lhs,
                entry_node,
                then_node,
                else_node,
                prefix,
                automaton_function,
            )?;
            translate_condition(
                context,
                rhs,
                entry_node,
                then_node,
                else_node,
                prefix,
                automaton_function,
            )?;
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
                automaton_function,
            )?;
            translate_condition(
                context,
                rhs,
                &false_node,
                then_node,
                else_node,
                prefix,
                automaton_function,
            )?;
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
                        lhs: translate_expression(context, lhs, automaton_function)?,
                        rhs: translate_expression(context, rhs, automaton_function)?,
                        negated: true,
                    },
                );
            }
            if let Some(else_node) = else_node {
                context.connect(
                    entry_node.clone(),
                    else_node.clone(),
                    rg::Label::Comparison {
                        lhs: translate_expression(context, lhs, automaton_function)?,
                        rhs: translate_expression(context, rhs, automaton_function)?,
                        negated: false,
                    },
                );
            }
        }
        hrg::Expression::Call { expression, args } => {
            let hrg::Expression::Literal { identifier } = expression.as_ref() else {
                panic!("Call expects a literal.");
            };

            match identifier.as_ref() {
                "false" => {
                    if let Some(else_node) = else_node {
                        context.connect(
                            entry_node.clone(),
                            else_node.clone(),
                            rg::Label::new_skip(),
                        );
                    }
                }
                "not" => {
                    check_arguments_length(identifier.clone(), 1, args.len())?;
                    translate_condition(
                        context,
                        args[0].as_ref(),
                        entry_node,
                        else_node,
                        then_node,
                        prefix,
                        automaton_function,
                    )?;
                }
                "reachable" => {
                    check_arguments_length(identifier.clone(), 1, args.len())?;
                    let hrg::Expression::Call { expression, args } = args[0].as_ref() else {
                        panic!("reachable() expects an automaton call.");
                    };
                    let hrg::Expression::Literal { identifier } = expression.as_ref() else {
                        panic!("reachable() expects an automaton call.");
                    };

                    let prefix = context.random(prefix);
                    let called_automaton_function = context
                        .hrg
                        .automaton
                        .iter()
                        .find(|automaton_function| automaton_function.name == *identifier)
                        .unwrap_or_else(|| panic!("Unknown automaton function \"{identifier}\"."))
                        .clone();

                    check_arguments_length(
                        called_automaton_function.name.clone(),
                        called_automaton_function.args.len(),
                        args.len(),
                    )?;

                    let call_id = context.random(&Id::from(format!(
                        "{}_call",
                        called_automaton_function.name
                    )));
                    let automaton_start_node = if called_automaton_function.reusable {
                        rg::Node::new(call_id)
                    } else {
                        context.random_node(&prefix)
                    };

                    let mut automaton_current_node = automaton_start_node.clone();
                    for (index, arg) in args.iter().enumerate() {
                        let arg_node = context.random_node(&prefix);
                        context.connect(
                            automaton_current_node,
                            arg_node.clone(),
                            rg::Label::Assignment {
                                lhs: Arc::from(rg::Expression::new(
                                    called_automaton_function.nth_arg_variable(index),
                                )),
                                rhs: translate_expression(context, arg, automaton_function)?,
                            },
                        );
                        automaton_current_node = arg_node;
                    }

                    context.connect(
                        automaton_current_node,
                        rg::Node::new(Id::from(if called_automaton_function.reusable {
                            format!("{identifier}_begin")
                        } else {
                            format!("{prefix}_{identifier}_begin")
                        })),
                        rg::Label::new_skip(),
                    );

                    let automaton_end_node =
                        rg::Node::new(Id::from(if called_automaton_function.reusable {
                            format!("{identifier}_end")
                        } else {
                            format!("{prefix}_{identifier}_end")
                        }));

                    if called_automaton_function.reusable {
                        if context
                            .translated_functions
                            .insert(called_automaton_function.name.clone())
                        {
                            translate_automaton_function(
                                context,
                                &called_automaton_function,
                                Some(&automaton_end_node),
                                None,
                                &Id::from(""),
                            )?;
                        }
                    } else {
                        translate_automaton_function(
                            context,
                            &called_automaton_function,
                            Some(&automaton_end_node),
                            None,
                            &Id::from(format!("{prefix}_")),
                        )?;
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
                        );
                    }
                }
                "true" => {
                    if let Some(then_node) = then_node {
                        context.connect(
                            entry_node.clone(),
                            then_node.clone(),
                            rg::Label::new_skip(),
                        );
                    }
                }
                _ => unimplemented!("Unknown condition function \"{identifier}\"."),
            }
        }
        _ => unimplemented!(),
    }

    Ok(())
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
            .map(|pattern| match pattern {
                hrg::DomainElementPattern::Literal { identifier } => {
                    vec![Arc::from(hrg::Value::Element {
                        identifier: identifier.clone(),
                    })]
                }
                hrg::DomainElementPattern::Variable { identifier } => {
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
                }
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

fn translate_expression(
    context: &Context,
    expression: &hrg::Expression<Id>,
    automaton_function: Option<&hrg::Function<Id>>,
) -> Result<Arc<rg::Expression<Id>>, hrg::Error<Id>> {
    Ok(match expression {
        hrg::Expression::Access { lhs, rhs } => Arc::from(rg::Expression::Access {
            span: Span::none(),
            lhs: translate_expression(context, lhs, automaton_function)?,
            rhs: translate_expression(context, rhs, automaton_function)?,
        }),
        hrg::Expression::Call { expression, args } => args.iter().try_fold(
            translate_expression(context, expression, automaton_function)?,
            |expression, arg| {
                Ok(Arc::from(rg::Expression::Access {
                    span: Span::none(),
                    lhs: expression,
                    rhs: translate_expression(context, arg, automaton_function)?,
                }))
            },
        )?,
        hrg::Expression::Constructor { .. } => Arc::from(rg::Expression::new(serialize_value(
            &evaluate_expression(context, expression, &BTreeMap::new())?,
        ))),
        hrg::Expression::Literal { identifier } => {
            // If it's function's argument...
            if let Some(automaton_function) = automaton_function {
                if let Some(index) = automaton_function.arg_index(identifier) {
                    // ...then rename it.
                    return Ok(Arc::from(rg::Expression::new(
                        automaton_function.nth_arg_variable(index),
                    )));
                }
            }
            Arc::from(rg::Expression::new(identifier.clone()))
        }
        _ => unimplemented!(),
    })
}

fn translate_function(
    context: &Context,
    function_declaration: &hrg::FunctionDeclaration<Id>,
) -> Result<rg::Constant<Id>, hrg::Error<Id>> {
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
    )?);

    Ok(rg::Constant {
        span: Span::none(),
        identifier: function_declaration.identifier.clone(),
        type_,
        value,
    })
}

fn translate_function_layer(
    context: &Context,
    function_declaration: &hrg::FunctionDeclaration<Id>,
    first_case: &hrg::FunctionCase<Id>,
    type_: &rg::Type<Id>,
    values: &[hrg::Value<Id>],
) -> Result<rg::Value<Id>, hrg::Error<Id>> {
    if first_case.args.len() > values.len() {
        if let rg::Type::Arrow { lhs, rhs } = type_ {
            return Ok(rg::Value::from_pairs(
                evaluate_type_values(context, lhs)?
                    .iter()
                    .map(|value| {
                        let key = serialize_value(value);
                        let value = Arc::from(translate_function_layer(
                            context,
                            function_declaration,
                            first_case,
                            rhs,
                            values
                                .iter()
                                .chain([value])
                                .cloned()
                                .collect::<Vec<_>>()
                                .as_slice(),
                        )?);

                        Ok((key, value))
                    })
                    .collect::<Result<_, _>>()?,
            ));
        }
    }

    let value = evaluate_expression_call(context, function_declaration, values)?;
    Ok(rg::Value::Element {
        identifier: serialize_value(&value),
    })
}

fn translate_functions(context: &mut Context) -> Result<(), hrg::Error<Id>> {
    for function_declaration in &context.hrg.functions {
        context
            .rg
            .constants
            .push(translate_function(context, function_declaration)?);
    }

    Ok(())
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

fn translate_value(value: &hrg::Value<Id>) -> Result<Arc<rg::Value<Id>>, hrg::Error<Id>> {
    Ok(Arc::from(match value {
        hrg::Value::Constructor { .. } => rg::Value::Element {
            identifier: serialize_value(value),
        },
        hrg::Value::Element { identifier } => rg::Value::Element {
            identifier: identifier.clone(),
        },
        hrg::Value::Map {
            default_value: None,
            entries,
        } => rg::Value::from_pairs(
            entries
                .iter()
                .map(|entry| {
                    let key = serialize_value(&entry.key);
                    let value = translate_value(&entry.value)?;
                    Ok((key, value))
                })
                .collect::<Result<_, _>>()?,
        ),
        hrg::Value::Map {
            default_value: Some(default_value),
            entries,
        } => rg::Value::Map {
            span: Span::none(),
            entries: Some(Ok(rg::ValueEntry::new_default(translate_value(
                default_value.as_ref(),
            )?)))
            .into_iter()
            .chain(entries.iter().map(|entry| {
                Ok(rg::ValueEntry {
                    span: Span::none(),
                    identifier: Some(serialize_value(&entry.key)),
                    value: translate_value(&entry.value)?,
                })
            }))
            .collect::<Result<Vec<_>, _>>()?,
        },
    }))
}

fn translate_variable(
    context: &Context,
    variable_declaration: &hrg::VariableDeclaration<Id>,
) -> Result<rg::Variable<Id>, hrg::Error<Id>> {
    let type_ = translate_type(&variable_declaration.type_);
    Ok(rg::Variable {
        span: Span::none(),
        default_value: translate_variable_default_value(
            context,
            &variable_declaration.default_value,
            &type_,
        )?,
        identifier: variable_declaration.identifier.clone(),
        type_,
    })
}

fn translate_variable_default_value(
    context: &Context,
    default_value: &Option<Arc<hrg::Expression<Id>>>,
    type_: &rg::Type<Id>,
) -> Result<Arc<rg::Value<Id>>, hrg::Error<Id>> {
    let Some(default_value) = default_value.as_deref() else {
        // If there's none, build one from the type.
        return translate_value(&evaluate_default_value(context, type_)?);
    };

    // If it's a map with a wildcard pattern, set it as the default.
    if let hrg::Expression::Map {
        default_value: None,
        parts,
    } = default_value
    {
        if parts.len() == 1 && *parts[0].pattern == hrg::Pattern::Wildcard {
            let value = evaluate_expression(context, &parts[0].expression, &BTreeMap::new())?;
            return Ok(Arc::from(rg::Value::new_empty(translate_value(&value)?)));
        }
    }

    // Build the map.
    // TODO: Check whether map domains match type domains.
    let value = evaluate_expression(context, default_value, &BTreeMap::new());
    translate_value(&value?)
}

fn translate_variables(context: &mut Context) -> Result<(), hrg::Error<Id>> {
    for variable_declaration in &context.hrg.variables {
        context
            .rg
            .variables
            .push(translate_variable(context, variable_declaration)?);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::hrg_to_rg;
    use hrg::parsing::parser::parse_with_errors as parse_hrg;
    use map_id::MapId;
    use rg::parsing::parser::parse_with_errors as parse_rg;
    use std::sync::Arc;

    macro_rules! test_translation {
        ($name:ident, $actual:expr, $expect:expr) => {
            #[test]
            fn $name() {
                let actual = hrg_to_rg({
                    let (game, errors) = parse_hrg($actual);
                    assert!(errors.is_empty(), "Parse errors: {errors:?}");
                    game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
                })
                .unwrap();
                let expect = {
                    let (game, errors) = parse_rg($expect);
                    assert!(errors.is_empty(), "Parse errors: {errors:?}");
                    game.map_id(&mut |id| Arc::from(id.identifier.as_str()))
                };

                // `assert_eq` prints the entire structs and it's not helpful.
                assert!(
                    actual == expect,
                    "\n\n>>> Actual: <<<\n{actual}\n>>> Expect: <<<\n{expect}\n"
                );
            }
        };
    }

    test_translation!(
        condition_false,
        "
            graph rules() {
                check(false())
            }
        ",
        "
            begin, rules_begin: ;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_true,
        "
            graph rules() {
                check(true())
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_1: ;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_and,
        "
            graph rules() {
                check(x == 1 && y == 2)
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_2: x == 1;
            rules_2, rules_1: y == 2;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_and_else,
        "
            graph rules() {
                if x == 1 && y == 2 { end() }
                x = y
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_3: x == 1;
            rules_begin, rules_2: x != 1;
            rules_3, rules_1: y == 2;
            rules_3, rules_2: y != 2;
            rules_1, end: player = keeper;
            rules_2, rules_4: x = y;
            rules_4, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_or,
        "
            graph rules() {
                check(x == 1 || y == 2)
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_1: x == 1;
            rules_begin, rules_1: y == 2;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_or_else,
        "
            graph rules() {
                if x == 1 || y == 2 { end() }
                x = y
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_1: x == 1;
            rules_begin, rules_3: x != 1;
            rules_3, rules_1: y == 2;
            rules_3, rules_2: y != 2;
            rules_1, end: player = keeper;
            rules_2, rules_4: x = y;
            rules_4, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_and_or,
        "
            graph rules() {
                check(x == 1 && (y == 2 || z == 3))
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_2: x == 1;
            rules_2, rules_1: y == 2;
            rules_2, rules_1: z == 3;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_or_and,
        "
            graph rules() {
                check((x == 1 && y == 2) || z == 3)
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_2: x == 1;
            rules_2, rules_1: y == 2;
            rules_begin, rules_1: z == 3;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_not_and,
        "
            graph rules() {
                check(not(x == 1 && y == 2))
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_2: x == 1;
            rules_begin, rules_1: x != 1;
            rules_2, rules_1: y != 2;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        condition_not_or,
        "
            graph rules() {
                check(not(x == 1 || y == 2))
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_2: x != 1;
            rules_2, rules_1: y != 2;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        empty_branch_or_loop,
        "
            graph rules() {
                branch {} or {
                    loop {
                        check(0 == 0)
                    }
                }
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_1: ;
            rules_begin, rules_2: ;
            rules_2, rules_4: 0 == 0;
            rules_4, rules_2: ;
            rules_3, rules_1: ;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        loop_with_branch_break,
        "
            graph rules() {
                loop {
                    branch {
                        break()
                    } or {
                        check(0 == 0)
                    }
                }
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_1: ;
            rules_1, rules_2: ;
            rules_1, rules_4: 0 == 0;
            rules_4, rules_3: ;
            rules_3, rules_1: ;
            rules_2, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        repeat_break,
        "
            graph rules() {
              repeat 3 {
                check(0 == 0)
                if 1 == 1 {
                  break()
                }
              }
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_3: 0 == 0;
            rules_3, rules_4: 1 == 1;
            rules_3, rules_5: 1 != 1;
            rules_4, rules_1: ;
            rules_5, rules_2: ;
            rules_2, rules_7: 0 == 0;
            rules_7, rules_8: 1 == 1;
            rules_7, rules_9: 1 != 1;
            rules_8, rules_1: ;
            rules_9, rules_6: ;
            rules_6, rules_11: 0 == 0;
            rules_11, rules_12: 1 == 1;
            rules_11, rules_13: 1 != 1;
            rules_12, rules_1: ;
            rules_13, rules_10: ;
            rules_10, rules_1: ;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        repeat_continue,
        "
            graph rules() {
              repeat 3 {
                if 1 == 1 {
                  continue()
                }
                check(0 == 0)
              }
            }
        ",
        "
            begin, rules_begin: ;
            rules_begin, rules_3: 1 == 1;
            rules_begin, rules_4: 1 != 1;
            rules_3, rules_2: ;
            rules_4, rules_5: 0 == 0;
            rules_5, rules_2: ;
            rules_2, rules_7: 1 == 1;
            rules_2, rules_8: 1 != 1;
            rules_7, rules_6: ;
            rules_8, rules_9: 0 == 0;
            rules_9, rules_6: ;
            rules_6, rules_11: 1 == 1;
            rules_6, rules_12: 1 != 1;
            rules_11, rules_10: ;
            rules_12, rules_13: 0 == 0;
            rules_13, rules_10: ;
            rules_10, rules_1: ;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );

    test_translation!(
        branch_var,
        "
            domain T = a | b | c
            graph rules() {
              branch x in T {
                check(x != c)
                if x == a {
                    branch y in T {
                        check(x == y)
                    }
                }
              }
            }
        ",
        "
            type T = { a, b, c };
            begin, rules_begin: ;
            rules_begin, rules_2: a != c;
            rules_2, rules_3: a == a;
            rules_2, rules_4: a != a;
            rules_3, rules_6: a == a;
            rules_6, rules_5: ;
            rules_3, rules_7: a == b;
            rules_7, rules_5: ;
            rules_3, rules_8: a == c;
            rules_8, rules_5: ;
            rules_5, rules_4: ;
            rules_4, rules_1: ;
            rules_begin, rules_9: b != c;
            rules_9, rules_10: b == a;
            rules_9, rules_11: b != a;
            rules_10, rules_13: b == a;
            rules_13, rules_12: ;
            rules_10, rules_14: b == b;
            rules_14, rules_12: ;
            rules_10, rules_15: b == c;
            rules_15, rules_12: ;
            rules_12, rules_11: ;
            rules_11, rules_1: ;
            rules_begin, rules_16: c != c;
            rules_16, rules_17: c == a;
            rules_16, rules_18: c != a;
            rules_17, rules_20: c == a;
            rules_20, rules_19: ;
            rules_17, rules_21: c == b;
            rules_21, rules_19: ;
            rules_17, rules_22: c == c;
            rules_22, rules_19: ;
            rules_19, rules_18: ;
            rules_18, rules_1: ;
            rules_1, rules_end: ;
            rules_end, end: ;
        "
    );
}
