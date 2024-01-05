use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

type Id = Arc<str>;

pub fn gdl_to_rg(gdl: &gdl::ast::Game<Id>) -> rg::ast::Game<Id> {
    let mut rg = rg::ast::Game {
        constants: vec![],
        edges: vec![],
        pragmas: vec![],
        typedefs: vec![],
        variables: vec![],
    };

    add_common_typedefs(&mut rg, gdl);
    rg.add_builtins().unwrap();
    add_does_variables(&mut rg, gdl);
    add_fact_variables(&mut rg, gdl);
    add_loop_edges(&mut rg, gdl);
    add_next_edges(&mut rg, gdl);
    add_terminal_edges(&mut rg, gdl);
    add_goal_edges(&mut rg, gdl);

    rg
}

fn add_common_typedefs(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::{AtomOrVariable, Rule, Term};
    use rg::ast::{Type, Typedef};
    use rg::position::Span;

    let roles = gdl
        .0
        .iter()
        .flat_map(|Rule { term, .. }| match term.as_ref() {
            Term::Legal(AtomOrVariable::Atom(role), _) => Some(role),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    rg.typedefs.push(Typedef {
        span: Span::none(),
        identifier: Id::from("Player"),
        type_: Arc::from(Type::Set {
            span: Span::none(),
            identifiers: roles.into_iter().cloned().collect(),
        }),
    });

    let goals = gdl
        .subterms()
        .flat_map(|term| match term {
            Term::Goal(_, AtomOrVariable::Atom(goal)) => Some(goal),
            _ => None,
        })
        .collect::<BTreeSet<_>>();

    rg.typedefs.push(Typedef {
        span: Span::none(),
        identifier: Id::from("Score"),
        type_: Arc::from(Type::Set {
            span: Span::none(),
            identifiers: goals.into_iter().cloned().collect(),
        }),
    });
}

fn add_does_variables(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::{AtomOrVariable, Term};
    use rg::ast::{Type, Typedef, Value, Variable};
    use rg::position::Span;

    let mut role_actions: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for term in gdl.subterms() {
        if let Term::Legal(AtomOrVariable::Atom(role), action) = term {
            if let Term::Custom(AtomOrVariable::Atom(id), arguments) = action.as_ref() {
                if arguments.is_empty() {
                    role_actions.entry(role).or_default().insert(id.clone());
                }
            }
        }
    }

    for (role, actions) in role_actions {
        rg.variables.push(Variable {
            span: Span::none(),
            default_value: Arc::from(Value::new(actions.first().unwrap().clone())),
            identifier: Id::from(format!("does_{role}")),
            type_: Arc::from(Type::new(Id::from(format!("does_{role}_type")))),
        });

        rg.typedefs.push(Typedef {
            span: Span::none(),
            identifier: Id::from(format!("does_{role}_type")),
            type_: Arc::from(Type::Set {
                span: Span::none(),
                identifiers: actions.into_iter().collect(),
            }),
        });
    }
}

fn add_fact_variables(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::{AtomOrVariable, Term};
    use rg::ast::{Type, Value, Variable};
    use rg::position::Span;

    let mut inits = BTreeSet::new();
    for term in gdl.subterms() {
        if let Term::Init(term) = term {
            if let Term::Custom(AtomOrVariable::Atom(id), arguments) = term.as_ref() {
                if arguments.is_empty() {
                    inits.insert(id);
                }
            }
        }
    }

    let mut variables = BTreeSet::new();
    for term in gdl.subterms() {
        if let Term::Base(term) | Term::Next(term) | Term::True(term) = term {
            if let Term::Custom(AtomOrVariable::Atom(id), arguments) = term.as_ref() {
                if arguments.is_empty() && variables.insert(id) {
                    rg.variables.push(Variable {
                        span: Span::none(),
                        default_value: Arc::from(Value::new(Id::from(if inits.contains(id) {
                            "1"
                        } else {
                            "0"
                        }))),
                        identifier: Id::from(format!("{id}_prev")),
                        type_: Arc::from(Type::new(Id::from("Bool"))),
                    });
                    rg.variables.push(Variable {
                        span: Span::none(),
                        default_value: Arc::from(Value::new(Id::from("0"))),
                        identifier: Id::from(format!("{id}_next")),
                        type_: Arc::from(Type::new(Id::from("Bool"))),
                    });
                }
            }
        }
    }
}

fn add_loop_edges(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::{AtomOrVariable, Rule, Term};
    use rg::ast::{Edge, EdgeLabel, EdgeName, Expression};
    use rg::position::Span;

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from("begin")),
        rhs: EdgeName::new(Id::from("loop_begin")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    let mut legals: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for term in gdl.subterms() {
        if let Term::Legal(AtomOrVariable::Atom(role), term) = term {
            if let Term::Custom(AtomOrVariable::Atom(action), arguments) = term.as_ref() {
                if arguments.is_empty() {
                    legals.entry(role).or_default().insert(action);
                }
            }
        }
    }

    let roles = gdl
        .0
        .iter()
        .flat_map(|Rule { term, .. }| match term.as_ref() {
            Term::Legal(AtomOrVariable::Atom(role), _) => Some(role),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    for (role_index, role) in roles.iter().enumerate() {
        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(if role_index == 0 {
                Id::from("loop_begin")
            } else {
                Id::from(format!("loop_{}_end", roles[role_index - 1]))
            }),
            rhs: EdgeName::new(Id::from(format!("loop_{role}_visibility_0"))),
            label: EdgeLabel::Skip { span: Span::none() },
        });

        for (opponent_index, opponent) in roles.iter().enumerate() {
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!(
                    "loop_{role}_visibility_{}",
                    opponent_index
                ))),
                rhs: EdgeName::new(Id::from(format!(
                    "loop_{role}_visibility_{}",
                    opponent_index + 1
                ))),
                label: EdgeLabel::Assignment {
                    lhs: Arc::from(Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(Expression::new(Id::from("visible"))),
                        rhs: Arc::from(Expression::new((*opponent).clone())),
                    }),
                    rhs: Arc::from(Expression::new(Id::from(if role_index == opponent_index {
                        "1"
                    } else {
                        "0"
                    }))),
                },
            });
        }

        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(Id::from(format!("loop_{role}_visibility_{}", roles.len()))),
            rhs: EdgeName::new(Id::from(format!("loop_{role}_begin"))),
            label: EdgeLabel::Assignment {
                lhs: Arc::from(Expression::new(Id::from("player"))),
                rhs: Arc::from(Expression::new((*role).clone())),
            },
        });

        for action in legals.get(role).unwrap().iter() {
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("loop_{role}_begin"))),
                rhs: EdgeName::new(Id::from(format!("loop_{role}_{action}_move"))),
                label: EdgeLabel::Reachability {
                    span: Span::none(),
                    negated: false,
                    lhs: EdgeName::new(Id::from(format!("check_{role}_{action}_begin"))),
                    rhs: EdgeName::new(Id::from(format!("check_{role}_{action}_end"))),
                },
            });
        }

        for action in legals.get(role).unwrap().iter() {
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("loop_{role}_{action}_move"))),
                rhs: EdgeName::new(Id::from(format!("loop_{role}_{action}_tag"))),
                label: EdgeLabel::Assignment {
                    lhs: Arc::from(Expression::new(Id::from(format!("does_{role}")))),
                    rhs: Arc::from(Expression::new((*action).clone())),
                },
            });

            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("loop_{role}_{action}_tag"))),
                rhs: EdgeName::new(Id::from(format!("loop_{role}_switch"))),
                label: EdgeLabel::Tag {
                    symbol: (*action).clone(),
                },
            });
        }

        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(Id::from(format!("loop_{role}_switch"))),
            rhs: EdgeName::new(Id::from(format!("loop_{role}_end"))),
            label: EdgeLabel::Assignment {
                lhs: Arc::from(Expression::new(Id::from("player"))),
                rhs: Arc::from(Expression::new(Id::from("keeper"))),
            },
        });
    }

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from(format!("loop_{}_end", roles.last().unwrap()))),
        rhs: EdgeName::new(Id::from("loop_end")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    for (role, actions) in legals {
        for action in actions {
            connect(
                rg,
                gdl,
                &Term::Legal(
                    AtomOrVariable::Atom(role.clone()),
                    Rc::new(Term::Custom(AtomOrVariable::Atom(action.clone()), vec![])),
                ),
                Id::from(format!("check_{role}_{action}_begin")),
                Id::from(format!("check_{role}_{action}_end")),
            );
        }
    }
}

fn add_next_edges(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::{AtomOrVariable, Term};
    use rg::ast::{Edge, EdgeLabel, EdgeName, Expression};
    use rg::position::Span;

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from("loop_end")),
        rhs: EdgeName::new(Id::from("next_0")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    let mut variables = BTreeSet::new();
    for term in gdl.subterms() {
        if let Term::Next(term) = term {
            if let Term::Custom(AtomOrVariable::Atom(id), arguments) = term.as_ref() {
                if arguments.is_empty() && variables.insert(id) {
                    rg.edges.push(Edge {
                        span: Span::none(),
                        lhs: EdgeName::new(Id::from(format!("next_{}", variables.len() - 1))),
                        rhs: EdgeName::new(Id::from(format!("next_{}_0", variables.len()))),
                        label: EdgeLabel::Reachability {
                            span: Span::none(),
                            negated: true,
                            lhs: EdgeName::new(Id::from(format!("next_{id}_check_begin",))),
                            rhs: EdgeName::new(Id::from(format!("next_{id}_check_end",))),
                        },
                    });

                    rg.edges.push(Edge {
                        span: Span::none(),
                        lhs: EdgeName::new(Id::from(format!("next_{}", variables.len() - 1))),
                        rhs: EdgeName::new(Id::from(format!("next_{}_1", variables.len()))),
                        label: EdgeLabel::Reachability {
                            span: Span::none(),
                            negated: false,
                            lhs: EdgeName::new(Id::from(format!("next_{id}_check_begin",))),
                            rhs: EdgeName::new(Id::from(format!("next_{id}_check_end",))),
                        },
                    });

                    rg.edges.push(Edge {
                        span: Span::none(),
                        lhs: EdgeName::new(Id::from(format!("next_{}_0", variables.len()))),
                        rhs: EdgeName::new(Id::from(format!("next_{}", variables.len()))),
                        label: EdgeLabel::Assignment {
                            lhs: Arc::from(Expression::new(Id::from(format!("{id}_next")))),
                            rhs: Arc::from(Expression::new(Id::from("0"))),
                        },
                    });

                    rg.edges.push(Edge {
                        span: Span::none(),
                        lhs: EdgeName::new(Id::from(format!("next_{}_1", variables.len()))),
                        rhs: EdgeName::new(Id::from(format!("next_{}", variables.len()))),
                        label: EdgeLabel::Assignment {
                            lhs: Arc::from(Expression::new(Id::from(format!("{id}_next")))),
                            rhs: Arc::from(Expression::new(Id::from("1"))),
                        },
                    });
                }
            }
        }
    }

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from(format!("next_{}", variables.len()))),
        rhs: EdgeName::new(Id::from("next_0_set")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    for (index, variable) in variables.iter().enumerate() {
        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(Id::from(format!("next_{}_set", index))),
            rhs: EdgeName::new(Id::from(format!("next_{}_set", index + 1))),
            label: EdgeLabel::Assignment {
                lhs: Arc::from(Expression::new(Id::from(format!("{variable}_prev")))),
                rhs: Arc::from(Expression::new(Id::from(format!("{variable}_next")))),
            },
        });
    }

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from(format!("next_{}_set", variables.len()))),
        rhs: EdgeName::new(Id::from("next_0_clear")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    for (index, variable) in variables.iter().enumerate() {
        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(Id::from(format!("next_{}_clear", index))),
            rhs: EdgeName::new(Id::from(format!("next_{}_clear", index + 1))),
            label: EdgeLabel::Assignment {
                lhs: Arc::from(Expression::new(Id::from(format!("{variable}_next")))),
                rhs: Arc::from(Expression::new(Id::from("0"))),
            },
        });
    }

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from(format!("next_{}_clear", variables.len()))),
        rhs: EdgeName::new(Id::from("next_end")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    for variable in variables {
        connect(
            rg,
            gdl,
            &Term::Next(Rc::new(Term::Custom(
                AtomOrVariable::Atom(variable.clone()),
                vec![],
            ))),
            Id::from(format!("next_{variable}_check_begin")),
            Id::from(format!("next_{variable}_check_end")),
        );
    }
}

fn add_terminal_edges(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::Term;
    use rg::ast::{Edge, EdgeLabel, EdgeName};
    use rg::position::Span;

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from("next_end")),
        rhs: EdgeName::new(Id::from("terminal")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from("terminal")),
        rhs: EdgeName::new(Id::from("loop_begin")),
        label: EdgeLabel::Reachability {
            span: Span::none(),
            negated: true,
            lhs: EdgeName::new(Id::from("terminal_begin")),
            rhs: EdgeName::new(Id::from("terminal_end")),
        },
    });

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from("terminal")),
        rhs: EdgeName::new(Id::from("terminal_end")),
        label: EdgeLabel::Reachability {
            span: Span::none(),
            negated: false,
            lhs: EdgeName::new(Id::from("terminal_begin")),
            rhs: EdgeName::new(Id::from("terminal_end")),
        },
    });

    connect(
        rg,
        gdl,
        &Term::Terminal,
        Id::from("terminal_begin"),
        Id::from("terminal_end"),
    );
}

fn add_goal_edges(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::{AtomOrVariable, Rule, Term};
    use rg::ast::{Edge, EdgeLabel, EdgeName, Expression};
    use rg::position::Span;

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from("terminal_end")),
        rhs: EdgeName::new(Id::from("goals_0_set")),
        label: EdgeLabel::Skip { span: Span::none() },
    });

    let mut goals: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for Rule { term, .. } in &gdl.0 {
        if let Term::Goal(AtomOrVariable::Atom(role), AtomOrVariable::Atom(goal)) = term.as_ref() {
            goals.entry(role).or_default().insert(goal);
        }
    }

    for (index, (role, goals)) in goals.iter().enumerate() {
        for goal in goals {
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("goals_{}_set", index))),
                rhs: EdgeName::new(Id::from(format!("goals_{}_check_{goal}", index + 1))),
                label: EdgeLabel::Reachability {
                    span: Span::none(),
                    negated: false,
                    lhs: EdgeName::new(Id::from(format!("goals_{role}_check_{goal}_begin"))),
                    rhs: EdgeName::new(Id::from(format!("goals_{role}_check_{goal}_end"))),
                },
            });
        }

        for goal in goals {
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("goals_{}_check_{goal}", index + 1))),
                rhs: EdgeName::new(Id::from(format!("goals_{}_set", index + 1))),
                label: EdgeLabel::Assignment {
                    lhs: Arc::from(Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(Expression::new(Id::from("goals"))),
                        rhs: Arc::from(Expression::new((**role).clone())),
                    }),
                    rhs: Arc::from(Expression::new((**goal).clone())),
                },
            });
        }
    }

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(Id::from(format!("goals_{}_set", goals.len()))),
        rhs: EdgeName::new(Id::from("end")),
        label: EdgeLabel::Assignment {
            lhs: Arc::from(Expression::new(Id::from("player"))),
            rhs: Arc::from(Expression::new(Id::from("keeper"))),
        },
    });

    for (role, goals) in goals {
        for goal in goals {
            connect(
                rg,
                gdl,
                &Term::Goal(
                    AtomOrVariable::Atom(role.clone()),
                    AtomOrVariable::Atom(goal.clone()),
                ),
                Id::from(format!("goals_{role}_check_{goal}_begin")),
                Id::from(format!("goals_{role}_check_{goal}_end")),
            );
        }
    }
}

fn connect(
    rg: &mut rg::ast::Game<Id>,
    gdl: &gdl::ast::Game<Id>,
    goal: &gdl::ast::Term<Id>,
    begin: Id,
    end: Id,
) {
    use gdl::ast::{AtomOrVariable, Predicate, Rule, Term};
    use rg::ast::{Edge, EdgeLabel, EdgeName, Expression};
    use rg::position::Span;

    let edges_count = rg.edges.len();
    for (index, Rule { term, predicates }) in gdl.0.iter().enumerate() {
        if **term != *goal {
            continue;
        }

        let term_id = match term.as_ref() {
            Term::Custom(AtomOrVariable::Atom(id), arguments) if arguments.is_empty() => {
                format!("{edges_count}_{index}_{id}")
            }
            _ => format!("{edges_count}_{index}"),
        };

        if rg
            .edges
            .iter()
            .any(|edge| edge.lhs == EdgeName::new(Id::from(format!("connected_{term_id}_0"))))
        {
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(begin.clone()),
                rhs: EdgeName::new(end.clone()),
                label: EdgeLabel::Reachability {
                    span: Span::none(),
                    negated: false,
                    lhs: EdgeName::new(Id::from(format!("connected_{term_id}_0"))),
                    rhs: EdgeName::new(Id::from(format!("connected_{term_id}_end"))),
                },
            });
            continue;
        }

        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(begin.clone()),
            rhs: EdgeName::new(Id::from(format!("connected_{term_id}_0"))),
            label: EdgeLabel::Skip { span: Span::none() },
        });

        for (step, Predicate { is_negated, term }) in predicates.iter().enumerate() {
            let label = match term.as_ref() {
                Term::Custom(_, _) => {
                    connect(
                        rg,
                        gdl,
                        term,
                        Id::from(format!("connected_{term_id}_{}_begin", step + 1)),
                        Id::from(format!("connected_{term_id}_{}_end", step + 1)),
                    );
                    EdgeLabel::Reachability {
                        span: Span::none(),
                        lhs: EdgeName::new(Id::from(format!(
                            "connected_{term_id}_{}_begin",
                            step + 1
                        ))),
                        rhs: EdgeName::new(Id::from(format!(
                            "connected_{term_id}_{}_end",
                            step + 1
                        ))),
                        negated: *is_negated,
                    }
                }
                Term::Does(AtomOrVariable::Atom(role), action) => match action.as_ref() {
                    Term::Custom(AtomOrVariable::Atom(id), arguments) if arguments.is_empty() => {
                        EdgeLabel::Comparison {
                            lhs: Arc::from(Expression::new(Id::from(format!("does_{role}")))),
                            rhs: Arc::from(Expression::new(id.clone())),
                            negated: *is_negated,
                        }
                    }
                    _ => unreachable!(),
                },
                Term::True(proposition) => match proposition.as_ref() {
                    Term::Custom(AtomOrVariable::Atom(variable), arguments)
                        if arguments.is_empty() =>
                    {
                        EdgeLabel::Comparison {
                            lhs: Arc::from(Expression::new(Id::from(format!("{variable}_prev")))),
                            rhs: Arc::from(Expression::new(Id::from("1"))),
                            negated: *is_negated,
                        }
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };

            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("connected_{term_id}_{}", step))),
                rhs: EdgeName::new(Id::from(format!("connected_{term_id}_{}", step + 1))),
                label,
            });
        }

        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(Id::from(format!(
                "connected_{term_id}_{}",
                predicates.len()
            ))),
            rhs: EdgeName::new(Id::from(format!("connected_{term_id}_end"))),
            label: EdgeLabel::Skip { span: Span::none() },
        });

        rg.edges.push(Edge {
            span: Span::none(),
            lhs: EdgeName::new(Id::from(format!("connected_{term_id}_end"))),
            rhs: EdgeName::new(end.clone()),
            label: EdgeLabel::Skip { span: Span::none() },
        });
    }
}
