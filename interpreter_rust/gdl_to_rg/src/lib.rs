use gdl::ast as gdl;
use map_id::MapId;
use rg::ast as rg;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::interner::Interner;

type Id = Arc<str>;

pub fn gdl_to_rg(gdl: &gdl::Game<&str>) -> rg::Game<Id> {
    let mut interner: Interner<&str, u8> = Interner::default();
    let gdl = gdl
        .map_id(&mut |id| interner.intern(id))
        .ground_smart(&interner.intern(&"distinct"), &interner.intern(&"or"))
        .expand_ors(&interner.intern(&"or"))
        .eval_distinct(&interner.intern(&"distinct"), &interner.intern(&"or"))
        .simplify()
        .map_id(&mut |id| Arc::from(*interner.recall(id).unwrap()))
        .symbolify();

    let mut rg = rg::Game::default();
    let subterms = gdl.subterms().to_vec();
    let rules_by_term: BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>> =
        gdl.0
            .iter()
            .fold(BTreeMap::new(), |mut rules_by_term, rule| {
                rules_by_term
                    .entry(rule.term.as_ref())
                    .or_default()
                    .push(rule);
                rules_by_term
            });

    add_common_typedefs(&mut rg, &subterms);
    rg.add_builtins().unwrap();
    add_does_variables(&mut rg, &subterms);
    add_fact_variables(&mut rg, &subterms);
    add_loop_edges(&mut rg, &subterms, &rules_by_term);
    add_next_edges(&mut rg, &subterms, &rules_by_term);
    add_terminal_edges(&mut rg, &rules_by_term);
    add_goal_edges(&mut rg, &subterms, &rules_by_term);

    rg
}

fn add_common_typedefs(rg: &mut rg::Game<Id>, subterms: &[&gdl::Term<Id>]) {
    use gdl::{AtomOrVariable, Term};
    use rg::{Type, Typedef};
    use utils::position::Span;

    let roles = subterms
        .iter()
        .filter_map(|term| match term {
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

    let goals = subterms
        .iter()
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

fn add_does_variables(rg: &mut rg::Game<Id>, subterms: &[&gdl::Term<Id>]) {
    use gdl::{AtomOrVariable, Term};
    use rg::{Type, Typedef, Value, Variable};
    use utils::position::Span;

    let mut role_actions: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for term in subterms {
        if let Term::Legal(AtomOrVariable::Atom(role), action) = term {
            if let Term::Custom0(AtomOrVariable::Atom(id)) = action.as_ref() {
                role_actions.entry(role).or_default().insert(id.clone());
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

fn add_fact_variables(rg: &mut rg::Game<Id>, subterms: &[&gdl::Term<Id>]) {
    use gdl::{AtomOrVariable, Term};
    use rg::{Type, Value, Variable};
    use utils::position::Span;

    let mut inits = BTreeSet::new();
    for term in subterms {
        if let Term::Init(term) = term {
            if let Term::Custom0(AtomOrVariable::Atom(id)) = term.as_ref() {
                inits.insert(id);
            }
        }
    }

    let mut variables = BTreeSet::new();
    for term in subterms {
        if let Term::Base(term) | Term::Next(term) | Term::True(term) = term {
            if let Term::Custom0(AtomOrVariable::Atom(id)) = term.as_ref() {
                if variables.insert(id) {
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

fn add_loop_edges(
    rg: &mut rg::Game<Id>,
    subterms: &[&gdl::Term<Id>],
    rules_by_term: &BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>>,
) {
    use gdl::{AtomOrVariable, Term};
    use rg::{Edge, Expression, Label, Node};
    use utils::position::Span;

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from("begin")),
        rhs: Node::new(Id::from("loop_begin")),
        label: Label::new_skip(),
    }));

    let mut legals: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for term in subterms {
        if let Term::Legal(AtomOrVariable::Atom(role), term) = term {
            if let Term::Custom0(AtomOrVariable::Atom(action)) = term.as_ref() {
                legals.entry(role).or_default().insert(action);
            }
        }
    }

    let roles = subterms
        .iter()
        .filter_map(|term| match term {
            Term::Legal(AtomOrVariable::Atom(role), _) => Some(role),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    for (role_index, role) in roles.iter().enumerate() {
        rg.add_edge_sorted(Arc::from(Edge {
            span: Span::none(),
            lhs: Node::new(if role_index == 0 {
                Id::from("loop_begin")
            } else {
                Id::from(format!("loop_{}_end", roles[role_index - 1]))
            }),
            rhs: Node::new(Id::from(format!("loop_{role}_visibility_0"))),
            label: Label::new_skip(),
        }));

        for (op_index, op) in roles.iter().enumerate() {
            let is_visible = if role_index == op_index { "1" } else { "0" };
            rg.add_edge_sorted(Arc::from(Edge {
                span: Span::none(),
                lhs: Node::new(Id::from(format!("loop_{role}_visibility_{op_index}"))),
                rhs: Node::new(Id::from(format!("loop_{role}_visibility_{}", op_index + 1))),
                label: Label::Assignment {
                    lhs: Arc::from(Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(Expression::new(Id::from("visible"))),
                        rhs: Arc::from(Expression::new((*op).clone())),
                    }),
                    rhs: Arc::from(Expression::new(Id::from(is_visible))),
                },
            }));
        }

        rg.add_edge_sorted(Arc::from(Edge {
            span: Span::none(),
            lhs: Node::new(Id::from(format!("loop_{role}_visibility_{}", roles.len()))),
            rhs: Node::new(Id::from(format!("loop_{role}_begin"))),
            label: Label::Assignment {
                lhs: Arc::from(Expression::new(Id::from("player"))),
                rhs: Arc::from(Expression::new((*role).clone())),
            },
        }));

        for action in legals.get(role).unwrap() {
            connect(
                rg,
                rules_by_term,
                &Term::Legal(
                    AtomOrVariable::Atom((*role).clone()),
                    Arc::new(Term::Custom0(AtomOrVariable::Atom((*action).clone()))),
                ),
                Id::from(format!("loop_{role}_begin")),
                Id::from(format!("loop_{role}_{action}_move")),
                false,
            );
        }

        for action in legals.get(role).unwrap() {
            rg.add_edge_sorted(Arc::from(Edge {
                span: Span::none(),
                lhs: Node::new(Id::from(format!("loop_{role}_{action}_move"))),
                rhs: Node::new(Id::from(format!("loop_{role}_{action}_tag"))),
                label: Label::Assignment {
                    lhs: Arc::from(Expression::new(Id::from(format!("does_{role}")))),
                    rhs: Arc::from(Expression::new((*action).clone())),
                },
            }));

            rg.add_edge_sorted(Arc::from(Edge {
                span: Span::none(),
                lhs: Node::new(Id::from(format!("loop_{role}_{action}_tag"))),
                rhs: Node::new(Id::from(format!("loop_{role}_switch"))),
                label: Label::Tag {
                    symbol: (*action).clone(),
                },
            }));
        }

        rg.add_edge_sorted(Arc::from(Edge {
            span: Span::none(),
            lhs: Node::new(Id::from(format!("loop_{role}_switch"))),
            rhs: Node::new(Id::from(format!("loop_{role}_end"))),
            label: Label::Assignment {
                lhs: Arc::from(Expression::new(Id::from("player"))),
                rhs: Arc::from(Expression::new(Id::from("keeper"))),
            },
        }));
    }

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from(format!("loop_{}_end", roles.last().unwrap()))),
        rhs: Node::new(Id::from("loop_end")),
        label: Label::new_skip(),
    }));
}

fn add_next_edges(
    rg: &mut rg::Game<Id>,
    subterms: &[&gdl::Term<Id>],
    rules_by_term: &BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>>,
) {
    use gdl::{AtomOrVariable, Term};
    use rg::{Edge, Expression, Label, Node};
    use utils::position::Span;

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from("loop_end")),
        rhs: Node::new(Id::from("next_0")),
        label: Label::new_skip(),
    }));

    let mut variables = BTreeSet::new();
    for term in subterms {
        if let Term::Init(term) | Term::Next(term) = term {
            if let Term::Custom0(AtomOrVariable::Atom(id)) = term.as_ref() {
                if !variables.insert(id) || term.is_init() {
                    continue;
                }

                connect(
                    rg,
                    rules_by_term,
                    &Term::Next(Arc::new(Term::Custom0(AtomOrVariable::Atom(id.clone())))),
                    Id::from(format!("next_{}", variables.len() - 1)),
                    Id::from(format!("next_{}_0", variables.len())),
                    true,
                );

                connect(
                    rg,
                    rules_by_term,
                    &Term::Next(Arc::new(Term::Custom0(AtomOrVariable::Atom(id.clone())))),
                    Id::from(format!("next_{}", variables.len() - 1)),
                    Id::from(format!("next_{}_1", variables.len())),
                    false,
                );

                rg.add_edge_sorted(Arc::from(Edge {
                    span: Span::none(),
                    lhs: Node::new(Id::from(format!("next_{}_0", variables.len()))),
                    rhs: Node::new(Id::from(format!("next_{}", variables.len()))),
                    label: Label::Assignment {
                        lhs: Arc::from(Expression::new(Id::from(format!("{id}_next")))),
                        rhs: Arc::from(Expression::new(Id::from("0"))),
                    },
                }));

                rg.add_edge_sorted(Arc::from(Edge {
                    span: Span::none(),
                    lhs: Node::new(Id::from(format!("next_{}_1", variables.len()))),
                    rhs: Node::new(Id::from(format!("next_{}", variables.len()))),
                    label: Label::Assignment {
                        lhs: Arc::from(Expression::new(Id::from(format!("{id}_next")))),
                        rhs: Arc::from(Expression::new(Id::from("1"))),
                    },
                }));
            }
        }
    }

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from(format!("next_{}", variables.len()))),
        rhs: Node::new(Id::from("next_0_set")),
        label: Label::new_skip(),
    }));

    for (index, variable) in variables.iter().enumerate() {
        rg.add_edge_sorted(Arc::from(Edge {
            span: Span::none(),
            lhs: Node::new(Id::from(format!("next_{index}_set"))),
            rhs: Node::new(Id::from(format!("next_{}_set", index + 1))),
            label: Label::Assignment {
                lhs: Arc::from(Expression::new(Id::from(format!("{variable}_prev")))),
                rhs: Arc::from(Expression::new(Id::from(format!("{variable}_next")))),
            },
        }));
    }

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from(format!("next_{}_set", variables.len()))),
        rhs: Node::new(Id::from("next_0_clear")),
        label: Label::new_skip(),
    }));

    for (index, variable) in variables.iter().enumerate() {
        rg.add_edge_sorted(Arc::from(Edge {
            span: Span::none(),
            lhs: Node::new(Id::from(format!("next_{index}_clear"))),
            rhs: Node::new(Id::from(format!("next_{}_clear", index + 1))),
            label: Label::Assignment {
                lhs: Arc::from(Expression::new(Id::from(format!("{variable}_next")))),
                rhs: Arc::from(Expression::new(Id::from("0"))),
            },
        }));
    }

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from(format!("next_{}_clear", variables.len()))),
        rhs: Node::new(Id::from("next_end")),
        label: Label::new_skip(),
    }));
}

fn add_terminal_edges(
    rg: &mut rg::Game<Id>,
    rules_by_term: &BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>>,
) {
    use gdl::Term;
    use rg::{Edge, Label, Node};
    use utils::position::Span;

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from("next_end")),
        rhs: Node::new(Id::from("terminal")),
        label: Label::new_skip(),
    }));

    connect(
        rg,
        rules_by_term,
        &Term::Terminal,
        Id::from("terminal"),
        Id::from("loop_begin"),
        true,
    );

    connect(
        rg,
        rules_by_term,
        &Term::Terminal,
        Id::from("terminal"),
        Id::from("terminal_end"),
        false,
    );
}

fn add_goal_edges(
    rg: &mut rg::Game<Id>,
    subterms: &[&gdl::Term<Id>],
    rules_by_term: &BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>>,
) {
    use gdl::{AtomOrVariable, Term};
    use rg::{Edge, Expression, Label, Node};
    use utils::position::Span;

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from("terminal_end")),
        rhs: Node::new(Id::from("goals_0_set")),
        label: Label::new_skip(),
    }));

    let mut goals: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for term in subterms {
        if let Term::Goal(AtomOrVariable::Atom(role), AtomOrVariable::Atom(goal)) = term {
            goals.entry(role).or_default().insert(goal);
        }
    }

    for (index, (role, goals)) in goals.iter().enumerate() {
        for goal in goals {
            connect(
                rg,
                rules_by_term,
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
            rg.add_edge_sorted(Arc::from(Edge {
                span: Span::none(),
                lhs: Node::new(Id::from(format!("goals_{}_check_{goal}", index + 1))),
                rhs: Node::new(Id::from(format!("goals_{}_set", index + 1))),
                label: Label::Assignment {
                    lhs: Arc::from(Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(Expression::new(Id::from("goals"))),
                        rhs: Arc::from(Expression::new((**role).clone())),
                    }),
                    rhs: Arc::from(Expression::new((**goal).clone())),
                },
            }));
        }
    }

    rg.add_edge_sorted(Arc::from(Edge {
        span: Span::none(),
        lhs: Node::new(Id::from(format!("goals_{}_set", goals.len()))),
        rhs: Node::new(Id::from("end")),
        label: Label::Assignment {
            lhs: Arc::from(Expression::new(Id::from("player"))),
            rhs: Arc::from(Expression::new(Id::from("keeper"))),
        },
    }));
}

fn hash_term(term: &gdl::Term<Id>) -> String {
    fn hash_term_inner(term: &gdl::Term<Id>, string: &mut String) {
        use gdl::{AtomOrVariable, Term};
        match term {
            Term::Custom0(AtomOrVariable::Atom(id)) => string.push_str(id),
            Term::Goal(AtomOrVariable::Atom(role), AtomOrVariable::Atom(goal)) => {
                string.push_str("goal_");
                string.push_str(role);
                string.push('_');
                string.push_str(goal);
            }
            Term::Legal(AtomOrVariable::Atom(role), action) => {
                string.push_str("legal_");
                string.push_str(role);
                string.push('_');
                hash_term_inner(action, string);
            }
            Term::Next(term) => {
                string.push_str("next_");
                hash_term_inner(term, string);
            }
            Term::Terminal => string.push_str("terminal"),
            _ => unimplemented!("{term:?}"),
        }
    }

    let mut string = String::new();
    hash_term_inner(term, &mut string);
    string
}

fn connect(
    rg: &mut rg::Game<Id>,
    rules_by_term: &BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>>,
    goal: &gdl::Term<Id>,
    begin: Id,
    end: Id,
    negated: bool,
) {
    use rg::{Edge, Label, Node};
    use utils::position::Span;

    if let Some((lhs, rhs)) = connect_inner(rg, rules_by_term, goal) {
        rg.add_edge_sorted(Arc::from(Edge {
            span: Span::none(),
            lhs: Node::new(begin),
            rhs: Node::new(end),
            label: Label::Reachability {
                span: Span::none(),
                lhs,
                rhs,
                negated,
            },
        }));
    }
}

fn connect_inner(
    rg: &mut rg::Game<Id>,
    rules_by_term: &BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>>,
    goal: &gdl::Term<Id>,
) -> Option<(rg::Node<Id>, rg::Node<Id>)> {
    use rg::{Edge, Label, Node};
    use utils::position::Span;

    let rules = match rules_by_term.get(goal).map(Vec::as_slice) {
        // If no edges were added, add an always-false one.
        None | Some([]) => return None,
        Some(rules) => rules,
    };

    let hash = hash_term(goal);
    let lhs = Node::new(Id::from(format!("__{hash}_begin")));
    let rhs = Node::new(Id::from(format!("__{hash}_end")));

    let start_present = rg.sorted_outgoing_edges(&lhs).next().is_some();
    if !start_present {
        for (index, rule) in rules.iter().enumerate() {
            let prefix = format!("__{hash}_{index}");
            rg.add_edge_sorted(Arc::from(Edge {
                span: Span::none(),
                lhs: lhs.clone(),
                rhs: Node::new(Id::from(format!("{prefix}_0"))),
                label: Label::new_skip(),
            }));

            for (step, predicate) in rule.predicates.iter().enumerate() {
                if let Some(label) = connect_label(rg, rules_by_term, predicate) {
                    rg.add_edge_sorted(Arc::from(Edge {
                        span: Span::none(),
                        lhs: Node::new(Id::from(format!("{prefix}_{step}"))),
                        rhs: Node::new(Id::from(format!("{prefix}_{}", step + 1))),
                        label,
                    }));
                }
            }

            rg.add_edge_sorted(Arc::from(Edge {
                span: Span::none(),
                lhs: Node::new(Id::from(format!("{prefix}_{}", rule.predicates.len()))),
                rhs: rhs.clone(),
                label: Label::new_skip(),
            }));
        }
    }

    Some((lhs, rhs))
}

fn connect_label(
    rg: &mut rg::Game<Id>,
    rules_by_term: &BTreeMap<&gdl::Term<Id>, Vec<&gdl::Rule<Id>>>,
    predicate: &gdl::Predicate<Id>,
) -> Option<rg::Label<Id>> {
    use gdl::{AtomOrVariable, Term};
    use rg::{Expression, Label};
    use utils::position::Span;

    Some(match predicate.term.as_ref() {
        Term::Custom0(_) | Term::CustomN(_, _) => {
            let (lhs, rhs) = connect_inner(rg, rules_by_term, &predicate.term)?;
            Label::Reachability {
                span: Span::none(),
                lhs,
                rhs,
                negated: predicate.is_negated,
            }
        }
        Term::Does(AtomOrVariable::Atom(role), action) => match action.as_ref() {
            Term::Custom0(AtomOrVariable::Atom(id)) => Label::Comparison {
                lhs: Arc::from(Expression::new(Id::from(format!("does_{role}")))),
                rhs: Arc::from(Expression::new(id.clone())),
                negated: predicate.is_negated,
            },
            _ => unreachable!(),
        },
        Term::Role(AtomOrVariable::Atom(role)) => Label::Comparison {
            lhs: Arc::from(Expression::new(Id::from("player"))),
            rhs: Arc::from(Expression::new(role.clone())),
            negated: predicate.is_negated,
        },
        Term::True(proposition) => match proposition.as_ref() {
            Term::Custom0(AtomOrVariable::Atom(id)) => Label::Comparison {
                lhs: Arc::from(Expression::new(Id::from(format!("{id}_prev")))),
                rhs: Arc::from(Expression::new(Id::from("1"))),
                negated: predicate.is_negated,
            },
            _ => unreachable!(),
        },
        _ => unreachable!(),
    })
}
