use map_id::MapId;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::Interner;

type Id = Arc<str>;

pub fn gdl_to_rg(gdl: &gdl::ast::Game<&str>) -> rg::ast::Game<Id> {
    let mut interner: Interner<&str, u8> = Interner::default();
    let gdl = gdl
        .map_id(&mut |id| interner.intern(id))
        .ground()
        .expand_ors(&interner.intern(&"or"))
        .eval_distinct(&interner.intern(&"distinct"))
        .simplify()
        .map_id(&mut |id| Arc::from(*interner.recall(id).unwrap()))
        .symbolify();

    let mut rg = rg::ast::Game {
        constants: vec![],
        edges: vec![],
        pragmas: vec![],
        typedefs: vec![],
        variables: vec![],
    };

    add_common_typedefs(&mut rg, &gdl);
    rg.add_builtins().unwrap();
    add_does_variables(&mut rg, &gdl);
    add_fact_variables(&mut rg, &gdl);
    add_loop_edges(&mut rg, &gdl);
    add_next_edges(&mut rg, &gdl);
    add_terminal_edges(&mut rg, &gdl);
    add_goal_edges(&mut rg, &gdl);

    rg
}

fn add_common_typedefs(rg: &mut rg::ast::Game<Id>, gdl: &gdl::ast::Game<Id>) {
    use gdl::ast::{AtomOrVariable, Term};
    use rg::ast::{Type, Typedef};
    use rg::position::Span;

    let roles = gdl
        .0
        .iter()
        .filter_map(|rule| match rule.term.as_ref() {
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
        .filter_map(|term| match term {
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
                    let default_value = if inits.contains(id) { "1" } else { "0" };
                    rg.variables.push(Variable {
                        span: Span::none(),
                        default_value: Arc::from(Value::new(Id::from(default_value))),
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
    use gdl::ast::{AtomOrVariable, Term};
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
        .filter_map(|rule| match rule.term.as_ref() {
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

        for (op_index, op) in roles.iter().enumerate() {
            let is_visible = if role_index == op_index { "1" } else { "0" };
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("loop_{role}_visibility_{op_index}"))),
                rhs: EdgeName::new(Id::from(format!("loop_{role}_visibility_{}", op_index + 1))),
                label: EdgeLabel::Assignment {
                    lhs: Arc::from(Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(Expression::new(Id::from("visible"))),
                        rhs: Arc::from(Expression::new((*op).clone())),
                    }),
                    rhs: Arc::from(Expression::new(Id::from(is_visible))),
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

        for action in legals.get(role).unwrap() {
            connect(
                rg,
                gdl,
                &Term::Legal(
                    AtomOrVariable::Atom((*role).clone()),
                    Arc::new(Term::Custom(
                        AtomOrVariable::Atom((*action).clone()),
                        vec![],
                    )),
                ),
                Id::from(format!("loop_{role}_begin")),
                Id::from(format!("loop_{role}_{action}_move")),
                false,
            );
        }

        for action in legals.get(role).unwrap() {
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
                    connect(
                        rg,
                        gdl,
                        &Term::Next(Arc::new(Term::Custom(
                            AtomOrVariable::Atom(id.clone()),
                            vec![],
                        ))),
                        Id::from(format!("next_{}", variables.len() - 1)),
                        Id::from(format!("next_{}_0", variables.len())),
                        true,
                    );

                    connect(
                        rg,
                        gdl,
                        &Term::Next(Arc::new(Term::Custom(
                            AtomOrVariable::Atom(id.clone()),
                            vec![],
                        ))),
                        Id::from(format!("next_{}", variables.len() - 1)),
                        Id::from(format!("next_{}_1", variables.len())),
                        false,
                    );

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
            lhs: EdgeName::new(Id::from(format!("next_{index}_set"))),
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
            lhs: EdgeName::new(Id::from(format!("next_{index}_clear"))),
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

    connect(
        rg,
        gdl,
        &Term::Terminal,
        Id::from("terminal"),
        Id::from("loop_begin"),
        true,
    );

    connect(
        rg,
        gdl,
        &Term::Terminal,
        Id::from("terminal"),
        Id::from("terminal_end"),
        false,
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
            connect(
                rg,
                gdl,
                &Term::Goal(
                    AtomOrVariable::Atom((*role).clone()),
                    AtomOrVariable::Atom((*goal).clone()),
                ),
                Id::from(format!("goals_{index}_set")),
                Id::from(format!("goals_{}_check_{goal}", index + 1)),
                false,
            );
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
}

fn hash_term(term: &gdl::ast::Term<Id>) -> String {
    use gdl::ast::{AtomOrVariable, Term};
    match term {
        Term::Custom(AtomOrVariable::Atom(id), arguments) if arguments.is_empty() => id.to_string(),
        Term::Goal(AtomOrVariable::Atom(role), AtomOrVariable::Atom(goal)) => {
            format!("goal_{role}_{goal}")
        }
        Term::Legal(AtomOrVariable::Atom(role), action) => {
            format!("legal_{role}_{}", hash_term(action))
        }
        Term::Next(term) => format!("next_{}", hash_term(term)),
        Term::Terminal => "terminal".to_string(),
        _ => unimplemented!("{term:?}"),
    }
}

fn connect(
    rg: &mut rg::ast::Game<Id>,
    gdl: &gdl::ast::Game<Id>,
    goal: &gdl::ast::Term<Id>,
    begin: Id,
    end: Id,
    negated: bool,
) {
    use rg::ast::{Edge, EdgeLabel, EdgeName, Expression};
    use rg::position::Span;

    let hash = hash_term(goal);
    let lhs = EdgeName::new(Id::from(format!("__{hash}_begin")));
    let rhs = EdgeName::new(Id::from(format!("__{hash}_end")));

    let start_present = rg.edges.iter().any(|edge| edge.lhs == lhs);
    if !start_present {
        let mut edge_added = false;
        for (index, rule) in gdl.0.iter().enumerate() {
            if rule.term.as_ref() != goal {
                continue;
            }

            edge_added = true;

            let prefix = format!("__{hash}_{index}");
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: lhs.clone(),
                rhs: EdgeName::new(Id::from(format!("{prefix}_0"))),
                label: EdgeLabel::Skip { span: Span::none() },
            });

            for (step, predicate) in rule.predicates.iter().enumerate() {
                let label = connect_one(rg, gdl, predicate, &format!("{prefix}_{}", step + 1));
                rg.edges.push(Edge {
                    span: Span::none(),
                    lhs: EdgeName::new(Id::from(format!("{prefix}_{step}"))),
                    rhs: EdgeName::new(Id::from(format!("{prefix}_{}", step + 1))),
                    label,
                });
            }

            rg.edges.push(Edge {
                span: Span::none(),
                lhs: EdgeName::new(Id::from(format!("{prefix}_{}", rule.predicates.len()))),
                rhs: rhs.clone(),
                label: EdgeLabel::Skip { span: Span::none() },
            });
        }

        // If no edges were added, add an always-false one.
        if !edge_added {
            rg.edges.push(Edge {
                span: Span::none(),
                lhs: lhs.clone(),
                rhs: rhs.clone(),
                label: EdgeLabel::Comparison {
                    lhs: Arc::from(Expression::new(begin.clone())),
                    rhs: Arc::from(Expression::new(begin.clone())),
                    negated: true,
                },
            });
        }
    }

    rg.edges.push(Edge {
        span: Span::none(),
        lhs: EdgeName::new(begin),
        rhs: EdgeName::new(end),
        label: EdgeLabel::Reachability {
            span: Span::none(),
            lhs,
            rhs,
            negated,
        },
    });
}

fn connect_one(
    rg: &mut rg::ast::Game<Id>,
    gdl: &gdl::ast::Game<Id>,
    predicate: &gdl::ast::Predicate<Id>,
    prefix: &str,
) -> rg::ast::EdgeLabel<Id> {
    use gdl::ast::{AtomOrVariable, Term};
    use rg::ast::{EdgeLabel, EdgeName, Expression};
    use rg::position::Span;

    match predicate.term.as_ref() {
        Term::Custom(_, _) => {
            let lhs = Id::from(format!("{prefix}_begin"));
            let rhs = Id::from(format!("{prefix}_end"));
            connect(rg, gdl, &predicate.term, lhs.clone(), rhs.clone(), false);
            EdgeLabel::Reachability {
                span: Span::none(),
                lhs: EdgeName::new(lhs),
                rhs: EdgeName::new(rhs),
                negated: predicate.is_negated,
            }
        }
        Term::Does(AtomOrVariable::Atom(role), action) => match action.as_ref() {
            Term::Custom(AtomOrVariable::Atom(id), arguments) if arguments.is_empty() => {
                EdgeLabel::Comparison {
                    lhs: Arc::from(Expression::new(Id::from(format!("does_{role}")))),
                    rhs: Arc::from(Expression::new(id.clone())),
                    negated: predicate.is_negated,
                }
            }
            _ => unreachable!(),
        },
        Term::True(proposition) => match proposition.as_ref() {
            Term::Custom(AtomOrVariable::Atom(id), arguments) if arguments.is_empty() => {
                EdgeLabel::Comparison {
                    lhs: Arc::from(Expression::new(Id::from(format!("{id}_prev")))),
                    rhs: Arc::from(Expression::new(Id::from("1"))),
                    negated: predicate.is_negated,
                }
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
