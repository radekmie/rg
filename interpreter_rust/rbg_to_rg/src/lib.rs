use rbg::ast as rbg;
use rg::ast as rg;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use utils::position::Span;

type Id = Arc<str>;

struct Context {
    expose_index: usize,
    node_index: usize,
    rbg: rbg::Game<Id>,
    rg: rg::Game<Id>,
    rule_automatons: BTreeMap<rbg::Rule<Id>, (rg::Node<Id>, rg::Node<Id>)>,
    shift_patterns: BTreeMap<(Id, rbg::Rule<Id>), Vec<Id>>,
}

impl Context {
    fn connect(&mut self, lhs: rg::Node<Id>, rhs: rg::Node<Id>, label: rg::Label<Id>) {
        self.rg
            .edges
            .push(Arc::from(rg::Edge::new(lhs, rhs, label)));
    }

    fn create_constant_map(&mut self, pairs: Vec<(Id, Id)>, default_value: Id) -> Id {
        let mut entries: Vec<_> = pairs
            .iter()
            .filter(|(_, rhs)| *rhs != default_value)
            .map(|(lhs, rhs)| rg::ValueEntry {
                span: Span::none(),
                identifier: Some(lhs.clone()),
                value: Arc::from(rg::Value::new(rhs.clone())),
            })
            .collect();
        entries.sort_unstable();

        let value = rg::Value::Map {
            span: Span::none(),
            entries: Some(rg::ValueEntry::new_default(Arc::from(rg::Value::new(
                default_value,
            ))))
            .into_iter()
            .chain(entries)
            .collect(),
        };

        let (mut lhss, mut rhss): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        lhss.sort_unstable();
        lhss.dedup();
        rhss.sort_unstable();
        rhss.dedup();

        let type_ = rg::Type::Arrow {
            lhs: self.create_type_from_set(lhss),
            rhs: self.create_type_from_set(rhss),
        };

        if let Some(constant) = self
            .rg
            .constants
            .iter()
            .find(|constant| *constant.type_ == type_ && *constant.value == value)
        {
            return constant.identifier.clone();
        }

        let identifier = (1..)
            .find_map(|index| {
                let identifier = Id::from(format!("RbgCoordMap{index}"));
                let exists = self
                    .rg
                    .constants
                    .iter()
                    .any(|constant| constant.identifier == identifier);
                (!exists).then_some(identifier)
            })
            .unwrap();

        self.rg.constants.push(rg::Constant {
            span: Span::none(),
            identifier: identifier.clone(),
            type_: Arc::from(type_),
            value: Arc::from(value),
        });

        identifier
    }

    fn create_math_operator(
        &mut self,
        limit: usize,
        lhs: Arc<rg::Expression<Id>>,
        rhs: Arc<rg::Expression<Id>>,
        operator: rbg::Operator,
    ) -> Arc<rg::Expression<Id>> {
        use rbg::Operator::{Comparison, Expression};
        use rbg::{ComparisonOperator, ExpressionOperator};

        let rhs_is_one = rhs.is_reference_and(|id| id.as_ref() == "1");
        let operator = match operator {
            Comparison(ComparisonOperator::Gt) => InternalOperator::Gt,
            Comparison(ComparisonOperator::Gte) => InternalOperator::Gte,
            Comparison(ComparisonOperator::Lt) => InternalOperator::Lt,
            Comparison(ComparisonOperator::Lte) => InternalOperator::Lte,
            Expression(ExpressionOperator::Add) if rhs_is_one => InternalOperator::Incr,
            Expression(ExpressionOperator::Add) => InternalOperator::Add,
            Expression(ExpressionOperator::Sub) if rhs_is_one => InternalOperator::Decr,
            Expression(ExpressionOperator::Sub) => InternalOperator::Sub,
            _ => unimplemented!("{operator:?}"),
        };

        let math_operator = Id::from(format!(
            "math_{}_{limit}",
            match operator {
                InternalOperator::Add => "add",
                InternalOperator::Decr => "decr",
                InternalOperator::Gt => "gt",
                InternalOperator::Gte => "gte",
                InternalOperator::Incr => "incr",
                InternalOperator::Lt => "lt",
                InternalOperator::Lte => "lte",
                InternalOperator::Sub => "sub",
            }
        ));

        let is_binary = matches!(operator, InternalOperator::Decr | InternalOperator::Incr);

        if !self
            .rg
            .constants
            .iter()
            .any(|constant| constant.identifier == math_operator)
        {
            let type_ = self.create_math_type(limit);
            let value = self.create_math_operator_value(limit, operator);
            let binary = Arc::from(rg::Type::Arrow {
                lhs: type_.clone(),
                rhs: type_.clone(),
            });

            self.rg.constants.push(rg::Constant {
                span: Span::none(),
                identifier: math_operator.clone(),
                type_: if is_binary {
                    binary
                } else {
                    Arc::from(rg::Type::Arrow {
                        lhs: type_,
                        rhs: binary,
                    })
                },
                value,
            });
        }

        let binary = Arc::from(rg::Expression::Access {
            span: Span::none(),
            lhs: Arc::from(rg::Expression::new(math_operator)),
            rhs: lhs,
        });

        if is_binary {
            binary
        } else {
            Arc::from(rg::Expression::Access {
                span: Span::none(),
                lhs: binary,
                rhs,
            })
        }
    }

    fn create_math_operator_value(
        &self,
        limit: usize,
        operator: InternalOperator,
    ) -> Arc<rg::Value<Id>> {
        use InternalOperator::{Add, Decr, Gt, Gte, Incr, Lt, Lte, Sub};

        let nan = Id::from("nan");
        let nan_value = Arc::from(rg::Value::new(nan.clone()));

        let mut numbers = BTreeMap::new();
        macro_rules! number {
            ($expr:expr) => {{
                let number = usize::from($expr);
                numbers
                    .entry(number)
                    .or_insert_with(|| Arc::from(rg::Value::new(Id::from(number.to_string()))))
                    .clone()
            }};
        }

        if operator == Decr || operator == Incr {
            return Arc::from(rg::Value::from_pairs(
                (0..limit)
                    .map(|lhs| {
                        let value = match operator {
                            Decr if lhs < 1 => nan_value.clone(),
                            Decr => number!(lhs - 1),
                            Incr if lhs + 1 >= limit => nan_value.clone(),
                            Incr => number!(lhs + 1),
                            _ => unreachable!(),
                        };

                        (Id::from(lhs.to_string()), value)
                    })
                    .chain([(nan.clone(), nan_value.clone())])
                    .collect(),
            ));
        }

        let nan_map = Arc::from(rg::Value::from_pairs(vec![(
            nan.clone(),
            nan_value.clone(),
        )]));

        Arc::from(rg::Value::from_pairs(
            (0..limit)
                .map(|lhs| {
                    let value = Arc::from(rg::Value::from_pairs(
                        (0..limit)
                            .map(|rhs| {
                                let value = match operator {
                                    Add if lhs + rhs >= limit => nan_value.clone(),
                                    Add => number!(lhs + rhs),
                                    Gt => number!(lhs > rhs),
                                    Gte => number!(lhs >= rhs),
                                    Lt => number!(lhs < rhs),
                                    Lte => number!(lhs <= rhs),
                                    Sub if lhs < rhs => nan_value.clone(),
                                    Sub => number!(lhs - rhs),
                                    _ => unreachable!(),
                                };

                                (Id::from(rhs.to_string()), value)
                            })
                            .chain([(nan.clone(), nan_value.clone())])
                            .collect(),
                    ));

                    (Id::from(lhs.to_string()), value)
                })
                .chain([(nan.clone(), nan_map.clone())])
                .collect(),
        ))
    }

    fn create_math_type(&mut self, limit: usize) -> Arc<rg::Type<Id>> {
        self.create_type_from_set(
            Some(Id::from("nan"))
                .into_iter()
                .chain((0..limit).map(|index| Id::from(index.to_string())))
                .collect(),
        )
    }

    fn create_type_from_set(&mut self, identifiers: Vec<Id>) -> Arc<rg::Type<Id>> {
        #[allow(clippy::map_unwrap_or)]
        let identifier = self.rg.typedefs.iter()
            .find(|typedef| matches!(typedef.type_.as_ref(), rg::Type::Set { identifiers: ids, .. } if *ids == identifiers))
            .map(|typedef| typedef.identifier.clone())
            .unwrap_or_else(|| {
                let identifier = (1..).find_map(|index| {
                    let identifier = Id::from(format!("RbgType{index}"));
                    let exists = self.rg.typedefs.iter().any(|typedef| typedef.identifier == identifier);
                    (!exists).then_some(identifier)
                }).unwrap();

                self.rg.typedefs.push(rg::Typedef {
                    span: Span::none(),
                    identifier: identifier.clone(),
                    type_: Arc::from(rg::Type::Set { span: Span::none(), identifiers }),
                });

                identifier
            });

        Arc::from(rg::Type::new(identifier))
    }

    fn make_shift_pattern(&mut self, coord: &Id, rule: &rbg::Rule<Id>) -> Vec<Id> {
        let key = (coord.clone(), rule.clone());
        if let Some(coords) = self.shift_patterns.get(&key).cloned() {
            return coords;
        }

        let coords = rule
            .elements
            .iter()
            .flat_map(|concatenation| {
                concatenation.iter().fold(
                    vec![coord.clone()],
                    |coords, rbg::Atom { content, power }| {
                        let mut reachable_coords = if *power {
                            coords.iter().cloned().collect()
                        } else {
                            BTreeSet::new()
                        };

                        for mut coord in coords {
                            match content {
                                rbg::ActionOrRule::Action(rbg::Action::Shift { label }) => loop {
                                    let node =
                                        self.rbg.board.iter().find(|node| node.node == coord);
                                    let edge = node.and_then(|node| {
                                        node.edges.iter().find(|edge| edge.label == *label)
                                    });
                                    if let Some(edge) = edge {
                                        reachable_coords.insert(edge.node.clone());
                                        coord = edge.node.clone();
                                    } else {
                                        break;
                                    }

                                    if !power {
                                        break;
                                    }
                                },
                                rbg::ActionOrRule::Action(_) => {
                                    panic!("Incorrect shift pattern: {content:?}")
                                }
                                rbg::ActionOrRule::Rule(rule) => {
                                    for node in self.make_shift_pattern(&coord, rule) {
                                        reachable_coords.insert(node.clone());
                                    }
                                }
                            }
                        }

                        reachable_coords.into_iter().collect()
                    },
                )
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        self.shift_patterns.insert(key, coords.clone());
        coords
    }

    fn random_node(&mut self) -> rg::Node<Id> {
        self.node_index += 1;
        rg::Node::new(Id::from(self.node_index.to_string()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InternalOperator {
    Add,
    Decr,
    Gt,
    Gte,
    Incr,
    Lt,
    Lte,
    Sub,
}

pub fn rbg_to_rg(rbg: rbg::Game<Id>) -> Result<rg::Game<Id>, rbg::Error<Id>> {
    let mut context = Context {
        expose_index: 0,
        node_index: 0,
        rbg,
        rg: rg::Game::default(),
        rule_automatons: BTreeMap::new(),
        shift_patterns: BTreeMap::new(),
    };

    context.rbg.rules = group_shift_patterns(context.rbg.rules);
    translate_game(&mut context);

    Ok(context.rg)
}

fn add_builtin_constants(context: &mut Context) {
    let default_direction =
        rg::ValueEntry::new_default(Arc::from(rg::Value::new(Id::from("null"))));

    context.rg.constants.push(rg::Constant {
        span: Span::none(),
        identifier: Id::from("direction"),
        type_: Arc::from(rg::Type::Arrow {
            lhs: Arc::from(rg::Type::new(Id::from("Label"))),
            rhs: Arc::from(rg::Type::Arrow {
                lhs: Arc::from(rg::Type::new(Id::from("Coord"))),
                rhs: Arc::from(rg::Type::new(Id::from("Coord"))),
            }),
        }),
        value: Arc::from(rg::Value::Map {
            span: Span::none(),
            entries: Some(rg::ValueEntry::new_default(Arc::from(rg::Value::Map {
                span: Span::none(),
                entries: vec![default_direction.clone()],
            })))
            .into_iter()
            .chain(
                context
                    .rbg
                    .board
                    .iter()
                    .flat_map(|node| {
                        node.edges.iter().map(|edge| {
                            (
                                edge.label.clone(),
                                rg::ValueEntry {
                                    span: Span::none(),
                                    identifier: Some(node.node.clone()),
                                    value: Arc::from(rg::Value::new(edge.node.clone())),
                                },
                            )
                        })
                    })
                    .fold(
                        Vec::<(_, _)>::new(),
                        |mut entries, (identifier, value_entry)| {
                            match entries.iter_mut().find(|entry| entry.0 == identifier) {
                                None => entries.push((
                                    identifier,
                                    vec![default_direction.clone(), value_entry],
                                )),
                                Some(entry) => entry.1.push(value_entry),
                            }

                            entries
                        },
                    )
                    .into_iter()
                    .map(|(identifier, entries)| {
                        rg::ValueEntry::new(
                            Span::none(),
                            Some(identifier),
                            Arc::from(rg::Value::Map {
                                span: Span::none(),
                                entries,
                            }),
                        )
                    }),
            )
            .collect(),
        }),
    });
}

fn add_builtin_types(context: &mut Context) {
    let pieces_type = context.create_type_from_set(
        (0..=context.rbg.board.len())
            .map(|index| Id::from(index.to_string()))
            .collect(),
    );

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Player"),
        type_: Arc::from(rg::Type::Set {
            span: Span::none(),
            identifiers: context
                .rbg
                .players
                .iter()
                .map(|player| player.name.clone())
                .collect(),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("PlayerOrKeeper"),
        type_: Arc::from(rg::Type::Set {
            span: Span::none(),
            identifiers: context
                .rbg
                .players
                .iter()
                .map(|player| player.name.clone())
                .chain(Some(Id::from("keeper")))
                .collect(),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Score"),
        type_: Arc::from(rg::Type::Set {
            span: Span::none(),
            identifiers: (0..=context
                .rbg
                .players
                .iter()
                .map(|player| player.bound)
                .max()
                .unwrap_or(0))
                .map(|index| Id::from(index.to_string()))
                .collect(),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Goals"),
        type_: Arc::from(rg::Type::Arrow {
            lhs: Arc::from(rg::Type::new(Id::from("Player"))),
            rhs: Arc::from(rg::Type::new(Id::from("Score"))),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Piece"),
        type_: Arc::from(rg::Type::Set {
            span: Span::none(),
            identifiers: context.rbg.pieces.clone(),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Label"),
        type_: Arc::from(rg::Type::Set {
            span: Span::none(),
            identifiers: context
                .rbg
                .board
                .iter()
                .flat_map(|node| &node.edges)
                .map(|edge| edge.label.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Coord"),
        type_: Arc::from(rg::Type::Set {
            span: Span::none(),
            identifiers: (Some(Id::from("null")))
                .into_iter()
                .chain(context.rbg.board.iter().map(|node| node.node.clone()))
                .collect(),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Board"),
        type_: Arc::from(rg::Type::Arrow {
            lhs: Arc::from(rg::Type::new(Id::from("Coord"))),
            rhs: Arc::from(rg::Type::new(Id::from("Piece"))),
        }),
    });

    context.rg.typedefs.push(rg::Typedef {
        span: Span::none(),
        identifier: Id::from("Counters"),
        type_: Arc::from(rg::Type::Arrow {
            lhs: Arc::from(rg::Type::new(Id::from("Piece"))),
            rhs: pieces_type,
        }),
    });
}

fn add_builtin_variables(context: &mut Context) {
    context.rg.variables.push(rg::Variable {
        span: Span::none(),
        identifier: Id::from("player"),
        type_: Arc::from(rg::Type::new(Id::from("PlayerOrKeeper"))),
        default_value: Arc::from(rg::Value::new(Id::from("keeper"))),
    });

    context.rg.variables.push(rg::Variable {
        span: Span::none(),
        identifier: Id::from("goals"),
        type_: Arc::from(rg::Type::new(Id::from("Goals"))),
        default_value: Arc::from(rg::Value::Map {
            span: Span::none(),
            entries: vec![rg::ValueEntry::new_default(Arc::from(rg::Value::new(
                Id::from("0"),
            )))],
        }),
    });

    context.rg.variables.push(rg::Variable {
        span: Span::none(),
        identifier: Id::from("board"),
        type_: Arc::from(rg::Type::new(Id::from("Board"))),
        default_value: Arc::from(rg::Value::from_pairs(
            context
                .rbg
                .board
                .iter()
                .map(|rbg::Node { node, piece, .. }| {
                    (node.clone(), Arc::from(rg::Value::new(piece.clone())))
                })
                .collect(),
        )),
    });

    context.rg.variables.push(rg::Variable {
        span: Span::none(),
        identifier: Id::from("coord"),
        type_: Arc::from(rg::Type::new(Id::from("Coord"))),
        default_value: Arc::from(rg::Value::new(context.rbg.board[0].node.clone())),
    });

    context.rg.variables.push(rg::Variable {
        span: Span::none(),
        identifier: Id::from("counters"),
        type_: Arc::from(rg::Type::new(Id::from("Counters"))),
        default_value: Arc::from(rg::Value::from_pairs(
            context
                .rbg
                .pieces
                .iter()
                .map(|piece| {
                    let count = context
                        .rbg
                        .board
                        .iter()
                        .filter(|node| node.piece == *piece)
                        .count()
                        .to_string();
                    (piece.clone(), Arc::from(rg::Value::new(Id::from(count))))
                })
                .collect(),
        )),
    });
}

fn bound_rvalue(context: &Context, rvalue: &rbg::RValue<Id>) -> usize {
    match rvalue {
        rbg::RValue::Expression(rbg::Expression { lhs, rhs, .. }) => {
            bound_rvalue(context, lhs).max(bound_rvalue(context, rhs))
        }
        rbg::RValue::Number(number) => *number,
        rbg::RValue::String(string) => {
            for player in &context.rbg.players {
                if player.name == *string {
                    return player.bound;
                }
            }

            for variable in &context.rbg.variables {
                if variable.name == *string {
                    return variable.bound;
                }
            }

            if context.rbg.pieces.contains(string) {
                return context.rbg.board.len();
            }

            panic!("Unbounded rvalue {rvalue:?}.")
        }
    }
}

fn copy_path(
    context: &mut Context,
    original_from: rg::Node<Id>,
    original_to: rg::Node<Id>,
) -> (rg::Node<Id>, rg::Node<Id>) {
    fn prefix_node(prefix: &Id, mut node: rg::Node<Id>) -> rg::Node<Id> {
        match node.parts.first_mut().unwrap() {
            rg::NodePart::Binding { .. } => {
                node.parts.insert(0, rg::NodePart::new(prefix.clone()));
            }
            rg::NodePart::Literal { identifier } => {
                *identifier = Id::from(format!("{prefix}_{identifier}"));
            }
        }

        node
    }

    fn copy(
        context: &mut Context,
        distances: &mut BTreeMap<rg::Node<Id>, Option<usize>>,
        prefix: &Id,
        edge: &rg::Edge<Id>,
        distance: usize,
    ) {
        // If the edge cannot reach the end _yet_, we check whether it is on a cycle
        // and if so, then add it anyway. It will copy too many edges, though.
        if distance == usize::MAX && distances.get(&edge.rhs).map_or(true, Option::is_some) {
            return;
        }

        let mut edge = rg::Edge {
            span: Span::none(),
            label: edge.label.clone(),
            lhs: prefix_node(prefix, edge.lhs.clone()),
            rhs: prefix_node(prefix, edge.rhs.clone()),
        };

        // Skip switch-generated edges:
        //   - coordGenerator == coord
        //   - $ coordGenerator
        //   - $ index_N
        if distance <= 3 {
            edge.lhs.parts.truncate(1);
            edge.rhs.parts.truncate(1);
            edge.skip();
        }

        if let Err(index) = context.rg.edges.binary_search_by(|x| x.cmp_outgoing(&edge)) {
            context.rg.edges.insert(index, Arc::from(edge));
        }
    }

    fn copy_if_on_path(
        context: &mut Context,
        distances: &mut BTreeMap<rg::Node<Id>, Option<usize>>,
        prefix: &Id,
        original_to: &rg::Node<Id>,
        node: &rg::Node<Id>,
    ) -> Option<usize> {
        if !distances.contains_key(node) {
            distances.insert(node.clone(), (node == original_to).then_some(0));

            // If it's not reached, copy and check.
            if distances[node].is_none() {
                let next_edges = context
                    .rg
                    .sorted_outgoing_edges(node)
                    .filter(|edge| !edge.label.is_player_assignment())
                    .cloned()
                    .collect::<Vec<_>>();

                for next in next_edges {
                    let distance =
                        copy_if_on_path(context, distances, prefix, original_to, &next.rhs)
                            .unwrap_or(usize::MAX)
                            .saturating_add(1);
                    copy(context, distances, prefix, &next, distance);
                    distances.insert(
                        node.clone(),
                        Some(distances[node].unwrap_or(usize::MAX).min(distance)),
                    );
                }
            }

            // If it wasn't reached, mark it as not reachable.
            if distances[node].is_none() {
                distances.insert(node.clone(), Some(usize::MAX));
            }
        }

        distances[node]
    }

    assert!(
        !original_from.has_bindings(),
        "Only simple nodes can be copied."
    );
    assert!(
        !original_to.has_bindings(),
        "Only simple nodes can be copied."
    );

    // Represent minimum distance to `original_to`. A `None` is an intermediate
    // state where we don't know if it's reachable or no. (It is used to copy
    // edges on cycles.) A `usize::MAX` means the `original_to` is not reachable.
    let mut distances = BTreeMap::new();
    let prefix = Id::from(format!("{original_from}_{original_to}"));

    context.rg.edges.sort_by(|x, y| x.cmp_outgoing(y));
    copy_if_on_path(
        context,
        &mut distances,
        &prefix,
        &original_to,
        &original_from,
    );

    (
        prefix_node(&prefix, original_from),
        prefix_node(&prefix, original_to),
    )
}

fn expose_position(context: &mut Context, from: rg::Node<Id>, to: rg::Node<Id>) {
    let mut local_coord = context.random_node();
    let bind = Id::from("coordGenerator");
    local_coord.add_binding(bind.clone(), Arc::from(rg::Type::new(Id::from("Coord"))));
    context.connect(
        from,
        local_coord.clone(),
        rg::Label::Comparison {
            lhs: Arc::from(rg::Expression::new(bind.clone())),
            rhs: Arc::from(rg::Expression::new(Id::from("coord"))),
            negated: false,
        },
    );

    let local_index = context.random_node();
    context.connect(
        local_coord,
        local_index.clone(),
        rg::Label::Tag { symbol: bind },
    );

    context.expose_index += 1;
    let expose_tag = Id::from(format!("index_{}", context.expose_index));
    context.connect(local_index, to, rg::Label::Tag { symbol: expose_tag });
}

fn group_shift_patterns(rule: rbg::Rule<Id>) -> rbg::Rule<Id> {
    if is_shift_pattern_rule(&rule) {
        return rule;
    }

    rbg::Rule {
        elements: rule
            .elements
            .into_iter()
            .map(|concatenation| {
                concatenation.into_iter().fold(
                    Vec::<rbg::Atom<Id>>::new(),
                    |mut concatenation, mut atom| {
                        match atom.content {
                            rbg::ActionOrRule::Action(rbg::Action::Check { negated, rule }) => {
                                atom.content = rbg::ActionOrRule::Action(rbg::Action::Check {
                                    negated,
                                    rule: group_shift_patterns(rule),
                                });
                            }
                            rbg::ActionOrRule::Action(_) => {}
                            rbg::ActionOrRule::Rule(rule) => {
                                atom.content = rbg::ActionOrRule::Rule(group_shift_patterns(rule));
                            }
                        };

                        if is_shift_pattern(&atom.content) {
                            let previous = concatenation.last_mut();
                            if let Some(previous) = previous {
                                if !previous.power {
                                    if let rbg::ActionOrRule::Rule(rbg::Rule { ref mut elements }) =
                                        &mut previous.content
                                    {
                                        if elements.len() == 1 {
                                            elements[0].push(atom);
                                            return concatenation;
                                        }
                                    }
                                }
                            }

                            concatenation.push(rbg::Atom {
                                content: rbg::ActionOrRule::Rule(rbg::Rule {
                                    elements: vec![vec![atom]],
                                }),
                                power: false,
                            });
                            return concatenation;
                        }

                        concatenation.push(atom);
                        concatenation
                    },
                )
            })
            .collect(),
    }
}

fn is_expandable_shift_pattern(rule: &rbg::Rule<Id>) -> bool {
    rule.elements
        .iter()
        .any(|concatenation| concatenation.len() > 1)
        && is_shift_pattern_rule(rule)
}

fn is_shift_pattern(content: &rbg::ActionOrRule<Id>) -> bool {
    match content {
        rbg::ActionOrRule::Action(action) => matches!(action, rbg::Action::Shift { .. }),
        rbg::ActionOrRule::Rule(rule) => is_shift_pattern_rule(rule),
    }
}

fn is_shift_pattern_rule(rule: &rbg::Rule<Id>) -> bool {
    rule.elements.iter().all(|concatenation| {
        concatenation
            .iter()
            .all(|atom| is_shift_pattern(&atom.content))
    })
}

fn has_math_expression(expression: &rg::Expression<Id>) -> bool {
    match expression {
        rg::Expression::Access { lhs, rhs, .. } => {
            has_math_expression(lhs) || has_math_expression(rhs)
        }
        rg::Expression::Cast { rhs, .. } => has_math_expression(rhs),
        rg::Expression::Reference { identifier } => identifier.starts_with("math_"),
    }
}

fn remove_power_skip_edges(context: &mut Context) {
    let mut edges_count = usize::MAX;
    while edges_count != context.rg.edges.len() {
        edges_count = context.rg.edges.len();

        let mut edges_to_remove = vec![];
        for x_index in 0..context.rg.edges.len() {
            let x = context.rg.edges[x_index].clone();
            if x.rhs.is_end() && x.label.is_skip() {
                edges_to_remove.push(x_index);
                for y_index in 0..context.rg.edges.len() {
                    let y = &mut context.rg.edges[y_index];
                    if y.rhs == x.lhs {
                        Arc::make_mut(y).rhs = x.rhs.clone();
                    }
                }
            }
        }

        for index in (0..context.rg.edges.len()).rev() {
            let edge = &context.rg.edges[index];
            if edge.rhs.is_end() && edge.label.is_skip() {
                context.rg.edges.remove(index);
            }
        }
    }
}

fn terminate_on_zero_moves(context: &mut Context) {
    let mut moves = vec![];

    // 1. For every `A, B: player = P`, where `P != keeper`.
    //   2. Find all paths from `B` to `D` ending in `E, _: player = *`.
    //   3. Add new edges from `B` to `end` with all `! C -> E`, where `C` is a fresh node between `B` and `D`.
    let player_assignments: Vec<_> = context
        .rg
        .edges
        .iter()
        .filter(|edge| edge.label.is_player_assignment())
        .cloned()
        .collect();

    for edge in player_assignments {
        let b = &edge.rhs;
        let mut visited = BTreeSet::new();
        let mut reachable_player_assignments = Vec::new();
        context.rg.edges.sort_by(|x, y| x.cmp_outgoing(y));
        for c in context.rg.sorted_outgoing_edges(b).map(|edge| &edge.rhs) {
            let mut queue = vec![c.clone()];
            while let Some(node) = queue.pop() {
                for edge in context.rg.sorted_outgoing_edges(&node) {
                    if edge.label.is_player_assignment() {
                        if !reachable_player_assignments.contains(&edge.lhs) {
                            reachable_player_assignments.push(edge.lhs.clone());
                        }
                    } else if visited.insert(edge.rhs.clone()) {
                        queue.push(edge.rhs.clone());
                    }
                }
            }
        }

        match reachable_player_assignments.len() {
            0 => {}
            1 => {
                let (lhs, rhs) = copy_path(
                    context,
                    b.clone(),
                    reachable_player_assignments.pop().unwrap(),
                );
                moves.push((edge.clone(), lhs, rhs));
            }
            _ => {
                let check_from = context.random_node();
                let check_to = context.random_node();
                for c in reachable_player_assignments {
                    let (lhs, rhs) = copy_path(context, b.clone(), c);
                    context.connect(
                        check_from.clone(),
                        check_to.clone(),
                        rg::Label::Reachability {
                            span: Span::none(),
                            lhs,
                            rhs,
                            negated: false,
                        },
                    );
                }
                moves.push((edge.clone(), check_from, check_to));
            }
        }
    }

    for (edge, lhs, rhs) in moves {
        context
            .rg
            .edges
            .remove(context.rg.edges.iter().position(|x| *x == edge).unwrap());
        let rg::Edge {
            label,
            lhs: a,
            rhs: b,
            ..
        } = Arc::unwrap_or_clone(edge);

        let check = context.random_node();
        context.connect(
            a.clone(),
            check.clone(),
            rg::Label::Reachability {
                span: Span::none(),
                lhs: lhs.clone(),
                rhs: rhs.clone(),
                negated: false,
            },
        );
        context.connect(check, b, label);

        let assign = context.random_node();
        context.connect(
            a,
            assign.clone(),
            rg::Label::Reachability {
                span: Span::none(),
                lhs,
                rhs,
                negated: true,
            },
        );
        context.connect(
            assign,
            rg::Node::new(Id::from("end")),
            rg::Label::Assignment {
                lhs: Arc::from(rg::Expression::new(Id::from("player"))),
                rhs: Arc::from(rg::Expression::new(Id::from("keeper"))),
            },
        );
    }
}

fn translate_atom_content(
    context: &mut Context,
    content: rbg::ActionOrRule<Id>,
    mut from: rg::Node<Id>,
    to: rg::Node<Id>,
) {
    match content {
        rbg::ActionOrRule::Action(rbg::Action::Assignment { variable, rvalue }) => {
            let (_, lhs, rhs) =
                translate_rvalue_pair(context, rbg::RValue::String(variable), rvalue);

            // Check for overflow.
            if has_math_expression(&rhs) {
                let local = context.random_node();
                context.connect(
                    from,
                    local.clone(),
                    rg::Label::Comparison {
                        lhs: rhs.clone(),
                        rhs: Arc::from(rg::Expression::new(Id::from("nan"))),
                        negated: true,
                    },
                );

                from = local;
            }

            let local = context.random_node();
            context.connect(from, local.clone(), rg::Label::Assignment { lhs, rhs });
            expose_position(context, local, to);
        }
        rbg::ActionOrRule::Action(rbg::Action::Check { negated, rule }) => {
            let (local_from, local_to) = match context.rule_automatons.get(&rule) {
                None => {
                    let local_from = context.random_node();
                    let local_to = context.random_node();
                    context
                        .rule_automatons
                        .insert(rule.clone(), (local_from.clone(), local_to.clone()));
                    translate_atom_content(
                        context,
                        rbg::ActionOrRule::Rule(rule),
                        local_from.clone(),
                        local_to.clone(),
                    );
                    (local_from, local_to)
                }
                Some((local_from, local_to)) => (local_from.clone(), local_to.clone()),
            };

            context.connect(
                from,
                to,
                rg::Label::Reachability {
                    span: Span::none(),
                    lhs: local_from,
                    rhs: local_to,
                    negated,
                },
            );
        }
        rbg::ActionOrRule::Action(rbg::Action::Comparison { lhs, rhs, operator }) => {
            let (limit, lhs, rhs) = translate_rvalue_pair(context, lhs, rhs);
            let label = match operator {
                rbg::ComparisonOperator::Eq | rbg::ComparisonOperator::Ne => {
                    rg::Label::Comparison {
                        lhs,
                        rhs,
                        negated: operator != rbg::ComparisonOperator::Eq,
                    }
                }
                _ => rg::Label::Comparison {
                    lhs: context.create_math_operator(
                        limit + 1,
                        lhs,
                        rhs,
                        rbg::Operator::Comparison(operator),
                    ),
                    rhs: Arc::from(rg::Expression::new(Id::from("1"))),
                    negated: false,
                },
            };
            context.connect(from, to, label);
        }
        rbg::ActionOrRule::Action(rbg::Action::Off { piece }) => {
            let one_element = Arc::from(rg::Expression::new(Id::from("1")));

            // Decrease.
            let counter_dec = context.random_node();
            let counter_dec_value = Arc::from(rg::Expression::Access {
                span: Span::none(),
                lhs: Arc::from(rg::Expression::new(Id::from("counters"))),
                rhs: Arc::from(rg::Expression::Access {
                    span: Span::none(),
                    lhs: Arc::from(rg::Expression::new(Id::from("board"))),
                    rhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                }),
            });

            let rhs = context.create_math_operator(
                context.rbg.board.len() + 1,
                counter_dec_value.clone(),
                one_element.clone(),
                rbg::Operator::Expression(rbg::ExpressionOperator::Sub),
            );

            context.connect(
                from,
                counter_dec.clone(),
                rg::Label::Assignment {
                    lhs: counter_dec_value,
                    rhs,
                },
            );

            // Increase.
            let counter_inc = context.random_node();
            let counter_inc_value = Arc::from(rg::Expression::Access {
                span: Span::none(),
                lhs: Arc::from(rg::Expression::new(Id::from("counters"))),
                rhs: Arc::from(rg::Expression::new(piece.clone())),
            });

            let rhs = context.create_math_operator(
                context.rbg.board.len() + 1,
                counter_inc_value.clone(),
                one_element,
                rbg::Operator::Expression(rbg::ExpressionOperator::Add),
            );

            context.connect(
                counter_dec,
                counter_inc.clone(),
                rg::Label::Assignment {
                    lhs: counter_inc_value,
                    rhs,
                },
            );

            // Set piece.
            let set_piece = context.random_node();
            context.connect(
                counter_inc,
                set_piece.clone(),
                rg::Label::Assignment {
                    lhs: Arc::from(rg::Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(rg::Expression::new(Id::from("board"))),
                        rhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                    }),
                    rhs: Arc::from(rg::Expression::new(piece.clone())),
                },
            );

            expose_position(context, set_piece, to);
        }
        rbg::ActionOrRule::Action(rbg::Action::On { pieces }) if pieces.is_empty() => {
            context.connect(from, rg::Node::new(Id::from("end")), rg::Label::new_skip());
        }
        rbg::ActionOrRule::Action(rbg::Action::On { pieces }) => {
            for piece in pieces {
                // Add an empty edge to make sure we won't create multiedges.
                let local = context.random_node();
                context.connect(from.clone(), local.clone(), rg::Label::new_skip());
                context.connect(
                    local,
                    to.clone(),
                    rg::Label::Comparison {
                        lhs: Arc::from(rg::Expression::Access {
                            span: Span::none(),
                            lhs: Arc::from(rg::Expression::new(Id::from("board"))),
                            rhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                        }),
                        rhs: Arc::from(rg::Expression::new(piece)),
                        negated: false,
                    },
                );
            }
        }
        rbg::ActionOrRule::Action(rbg::Action::Shift { label }) => {
            let local = context.random_node();

            context.connect(
                from,
                local.clone(),
                rg::Label::Assignment {
                    lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                    rhs: Arc::from(rg::Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(rg::Expression::Access {
                            span: Span::none(),
                            lhs: Arc::from(rg::Expression::new(Id::from("direction"))),
                            rhs: Arc::from(rg::Expression::new(label)),
                        }),
                        rhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                    }),
                },
            );

            context.connect(
                local,
                to,
                rg::Label::Comparison {
                    lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                    rhs: Arc::from(rg::Expression::new(Id::from("null"))),
                    negated: true,
                },
            );
        }
        rbg::ActionOrRule::Action(rbg::Action::Switch { player }) => {
            let local = context.random_node();
            expose_position(context, from, local.clone());
            context.connect(
                local,
                to,
                rg::Label::Assignment {
                    lhs: Arc::from(rg::Expression::new(Id::from("player"))),
                    rhs: Arc::from(rg::Expression::new(
                        player.unwrap_or_else(|| Id::from("keeper")),
                    )),
                },
            );
        }
        rbg::ActionOrRule::Rule(rule) if is_expandable_shift_pattern(&rule) => {
            let pairs: Vec<_> = context
                .rbg
                .board
                .clone()
                .into_iter()
                .map(|node| {
                    (
                        node.node.clone(),
                        context.make_shift_pattern(&node.node, &rule),
                    )
                })
                .collect();
            let pairs_len = pairs.len();

            if pairs.iter().all(|(_, coords)| coords.len() == pairs_len) {
                let mut local = context.random_node();
                local.add_binding(
                    Id::from("coordGenerator"),
                    Arc::from(rg::Type::new(Id::from("Coord"))),
                );

                context.connect(
                    from,
                    local.clone(),
                    rg::Label::Comparison {
                        lhs: Arc::from(rg::Expression::Cast {
                            span: Span::none(),
                            lhs: Arc::from(rg::Type::new(Id::from("Coord"))),
                            rhs: Arc::from(rg::Expression::new(Id::from("coordGenerator"))),
                        }),
                        rhs: Arc::from(rg::Expression::Cast {
                            span: Span::none(),
                            lhs: Arc::from(rg::Type::new(Id::from("Coord"))),
                            rhs: Arc::from(rg::Expression::new(Id::from("null"))),
                        }),
                        negated: true,
                    },
                );

                context.connect(
                    local,
                    to.clone(),
                    rg::Label::Assignment {
                        lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                        rhs: Arc::from(rg::Expression::new(Id::from("coordGenerator"))),
                    },
                );
                return;
            }

            if pairs.iter().all(|(_, coords)| coords.len() <= 1) {
                let map: Vec<_> = pairs
                    .into_iter()
                    .map(|(coord, mut coords)| {
                        (coord, coords.pop().unwrap_or_else(|| Id::from("null")))
                    })
                    .chain(Some((Id::from("null"), Id::from("null"))))
                    .collect();

                let constant = context.create_constant_map(map, Id::from("null"));
                let local = context.random_node();

                context.connect(
                    from,
                    local.clone(),
                    rg::Label::Assignment {
                        lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                        rhs: Arc::from(rg::Expression::Access {
                            span: Span::none(),
                            lhs: Arc::from(rg::Expression::new(constant)),
                            rhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                        }),
                    },
                );

                context.connect(
                    local,
                    to,
                    rg::Label::Comparison {
                        lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                        rhs: Arc::from(rg::Expression::new(Id::from("null"))),
                        negated: true,
                    },
                );
                return;
            }

            for (coord, mut reachable_coords) in pairs {
                if reachable_coords.is_empty() {
                    continue;
                }

                let mut local = context.random_node();
                if reachable_coords.len() == 1 {
                    context.connect(
                        from.clone(),
                        local.clone(),
                        rg::Label::Comparison {
                            lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                            rhs: Arc::from(rg::Expression::new(coord.clone())),
                            negated: false,
                        },
                    );
                    context.connect(
                        local,
                        to.clone(),
                        rg::Label::Assignment {
                            lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                            rhs: Arc::from(rg::Expression::new(reachable_coords.pop().unwrap())),
                        },
                    );
                    continue;
                }

                let uses_all_coords = reachable_coords.len() == pairs_len;
                local.add_binding(
                    Id::from("coordGenerator"),
                    if uses_all_coords {
                        Arc::from(rg::Type::new(Id::from("Coord")))
                    } else {
                        context.create_type_from_set(reachable_coords)
                    },
                );

                if uses_all_coords {
                    context.connect(
                        from.clone(),
                        local.clone(),
                        rg::Label::Comparison {
                            lhs: Arc::from(rg::Expression::Cast {
                                span: Span::none(),
                                lhs: Arc::from(rg::Type::new(Id::from("Coord"))),
                                rhs: Arc::from(rg::Expression::new(Id::from("coordGenerator"))),
                            }),
                            rhs: Arc::from(rg::Expression::Cast {
                                span: Span::none(),
                                lhs: Arc::from(rg::Type::new(Id::from("Coord"))),
                                rhs: Arc::from(rg::Expression::new(Id::from("null"))),
                            }),
                            negated: true,
                        },
                    );

                    context.connect(
                        local,
                        to.clone(),
                        rg::Label::Assignment {
                            lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                            rhs: Arc::from(rg::Expression::new(Id::from("coordGenerator"))),
                        },
                    );

                    continue;
                }

                context.connect(
                    from.clone(),
                    local.clone(),
                    rg::Label::Comparison {
                        lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                        rhs: Arc::from(rg::Expression::Cast {
                            span: Span::none(),
                            lhs: Arc::from(rg::Type::new(Id::from("Coord"))),
                            rhs: Arc::from(rg::Expression::new(coord)),
                        }),
                        negated: false,
                    },
                );

                context.connect(
                    local.clone(),
                    to.clone(),
                    rg::Label::Assignment {
                        lhs: Arc::from(rg::Expression::new(Id::from("coord"))),
                        rhs: Arc::from(rg::Expression::new(Id::from("coordGenerator"))),
                    },
                );
            }
        }
        rbg::ActionOrRule::Rule(rbg::Rule { elements }) => {
            for concatenation in elements {
                let mut local_from = from.clone();
                for atom in concatenation {
                    let local_to = context.random_node();
                    if atom.power {
                        let local_pre = context.random_node();
                        let local_after = context.random_node();
                        translate_atom_content(
                            context,
                            atom.content,
                            local_pre.clone(),
                            local_after.clone(),
                        );
                        context.connect(
                            local_from.clone(),
                            local_pre.clone(),
                            rg::Label::new_skip(),
                        );
                        context.connect(local_from, local_to.clone(), rg::Label::new_skip());
                        context.connect(local_after.clone(), local_pre, rg::Label::new_skip());
                        context.connect(local_after, local_to.clone(), rg::Label::new_skip());
                    } else {
                        translate_atom_content(context, atom.content, local_from, local_to.clone());
                    }
                    local_from = local_to;
                }
                context.connect(local_from, to.clone(), rg::Label::new_skip());
            }
        }
    }
}

fn translate_game(context: &mut Context) {
    add_builtin_types(context);
    add_builtin_constants(context);
    add_builtin_variables(context);

    for variable in context.rbg.variables.clone() {
        let variable = translate_variable(context, variable);
        context.rg.variables.push(variable);
    }

    translate_atom_content(
        context,
        rbg::ActionOrRule::Rule(context.rbg.rules.clone()),
        rg::Node::new(Id::from("begin")),
        rg::Node::new(Id::from("end")),
    );

    remove_power_skip_edges(context);
    terminate_on_zero_moves(context);
}

fn translate_rvalue(
    context: &mut Context,
    rvalue: rbg::RValue<Id>,
    limit: usize,
) -> Arc<rg::Expression<Id>> {
    match rvalue {
        rbg::RValue::Expression(rbg::Expression { lhs, rhs, operator }) => {
            let lhs = translate_rvalue(context, Arc::unwrap_or_clone(lhs), limit);
            let rhs = translate_rvalue(context, Arc::unwrap_or_clone(rhs), limit);
            context.create_math_operator(limit + 1, lhs, rhs, rbg::Operator::Expression(operator))
        }
        rbg::RValue::Number(number) => Arc::from(rg::Expression::new(Id::from(number.to_string()))),
        rbg::RValue::String(string) => {
            for player in &context.rbg.players {
                if player.name == string {
                    return Arc::from(rg::Expression::Access {
                        span: Span::none(),
                        lhs: Arc::from(rg::Expression::new(Id::from("goals"))),
                        rhs: Arc::from(rg::Expression::new(string)),
                    });
                }
            }

            if context.rbg.pieces.contains(&string) {
                return Arc::from(rg::Expression::Access {
                    span: Span::none(),
                    lhs: Arc::from(rg::Expression::new(Id::from("counters"))),
                    rhs: Arc::from(rg::Expression::new(string)),
                });
            }

            Arc::from(rg::Expression::new(string))
        }
    }
}

fn translate_rvalue_pair(
    context: &mut Context,
    lhs: rbg::RValue<Id>,
    rhs: rbg::RValue<Id>,
) -> (usize, Arc<rg::Expression<Id>>, Arc<rg::Expression<Id>>) {
    let limit = bound_rvalue(
        context,
        &rbg::RValue::Expression(rbg::Expression {
            lhs: Arc::from(lhs.clone()),
            rhs: Arc::from(rhs.clone()),
            operator: rbg::ExpressionOperator::Add,
        }),
    );

    (
        limit,
        translate_rvalue(context, lhs, limit),
        translate_rvalue(context, rhs, limit),
    )
}

fn translate_variable(context: &mut Context, variable: rbg::Variable<Id>) -> rg::Variable<Id> {
    rg::Variable {
        span: Span::none(),
        default_value: Arc::from(rg::Value::new(Id::from("0"))),
        identifier: variable.name,
        type_: context.create_type_from_set(
            (0..=variable.bound)
                .map(|index| Id::from(index.to_string()))
                .collect(),
        ),
    }
}
