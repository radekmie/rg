use crate::ist;
use rg::ast;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

type Id = Arc<str>;

struct Context {
    constants_indexes: BTreeMap<Id, usize>,
    game: ist::Game<Id>,
    types: BTreeMap<Id, Rc<ist::Type<Id>>>,
    variables_indexes: BTreeMap<Id, usize>,
}

impl From<ast::Game<Id>> for ist::Game<Id> {
    fn from(ast: ast::Game<Id>) -> Self {
        let placeholder_value = Rc::new(ist::Value::Element {
            value: Arc::from(""),
        });

        let mut context = Context {
            constants_indexes: BTreeMap::default(),
            game: Self {
                constants: Vec::default(),
                disjoints: BTreeSet::default(),
                edges: BTreeMap::default(),
                initial_goals: placeholder_value.clone(),
                initial_player: placeholder_value.clone(),
                initial_values: Rc::default(),
                initial_visible: placeholder_value.clone(),
                repeats: BTreeMap::default(),
                uniques: BTreeSet::default(),
            },
            types: BTreeMap::default(),
            variables_indexes: BTreeMap::default(),
        };

        build_typedefs(&mut context, ast.typedefs);
        build_constants(&mut context, ast.constants);
        build_variables(&mut context, ast.variables);
        build_edges(&mut context, ast.edges);
        build_pragmas(&mut context, ast.pragmas);

        // Make sure no placeholders are left.
        assert_ne!(context.game.initial_goals, placeholder_value);
        assert_ne!(context.game.initial_player, placeholder_value);
        assert_ne!(context.game.initial_visible, placeholder_value);

        context.game
    }
}

fn build_constants(context: &mut Context, constants: Vec<ast::Constant<Id>>) {
    for constant in constants {
        let type_ = build_type_or_fail(context, &constant.type_);
        let value = build_value(context, &type_, &constant.value);
        context
            .constants_indexes
            .insert(constant.identifier, context.game.constants.len());
        context.game.constants.push(value);
    }
}

fn build_label(context: &mut Context, label: ast::Label<Id>) -> ist::EdgeLabel<Id> {
    match label {
        ast::Label::Assignment { lhs, rhs } => ist::EdgeLabel::Assignment {
            lhs: build_expression(context, &lhs),
            rhs: build_expression(context, &rhs),
        },
        ast::Label::Comparison { lhs, rhs, negated } => ist::EdgeLabel::Comparison {
            lhs: build_expression(context, &lhs),
            rhs: build_expression(context, &rhs),
            negated,
        },
        ast::Label::Reachability {
            lhs, rhs, negated, ..
        } => ist::EdgeLabel::Reachability {
            lhs: build_node(lhs),
            rhs: build_node(rhs),
            negated,
        },
        ast::Label::Skip { .. } => ist::EdgeLabel::Skip,
        ast::Label::Tag { symbol } => ist::EdgeLabel::Tag { symbol },
    }
}

fn build_node(mut node: ast::Node<Id>) -> Id {
    assert!(node.parts.len() == 1, "Only trivial EdgeName allowed.");
    let Some(ast::NodePart::Literal { identifier }) = node.parts.pop() else {
        panic!("Only trivial EdgeName allowed.")
    };
    identifier
}

fn build_edges(context: &mut Context, edges: Vec<Arc<ast::Edge<Id>>>) {
    for edge in edges {
        let edge = Arc::unwrap_or_clone(edge);
        let lhs = build_node(edge.lhs);
        let rhs = build_node(edge.rhs);
        let label = build_label(context, edge.label);

        context
            .game
            .edges
            .entry(lhs)
            .or_default()
            .push(ist::Edge { label, next: rhs });
    }
}

fn build_expression(
    context: &mut Context,
    expression: &ast::Expression<Id>,
) -> ist::Expression<Id> {
    match expression {
        ast::Expression::Access { lhs, rhs, .. } => ist::Expression::Access {
            lhs: Rc::new(build_expression(context, lhs)),
            rhs: Rc::new(build_expression(context, rhs)),
        },
        ast::Expression::Cast { rhs, .. } => build_expression(context, rhs),
        ast::Expression::Reference { identifier } => {
            match identifier.as_ref() {
                "goals" => return ist::Expression::GoalsReference,
                "player" => return ist::Expression::PlayerReference,
                "visible" => return ist::Expression::VisibleReference,
                _ => {}
            }

            let identifier = identifier.clone();
            if let Some(&index) = context.constants_indexes.get(&identifier) {
                return ist::Expression::ConstantReference { index };
            }

            if let Some(&index) = context.variables_indexes.get(&identifier) {
                return ist::Expression::VariableReference { index };
            }

            ist::Expression::Literal {
                value: Rc::new(ist::Value::Element { value: identifier }),
            }
        }
    }
}

fn build_pragmas(context: &mut Context, pragmas: Vec<ast::Pragma<Id>>) {
    for pragma in pragmas {
        match pragma {
            ast::Pragma::Disjoint { node, nodes, .. }
            | ast::Pragma::DisjointExhaustive { node, nodes, .. } => {
                let node = build_node(node);
                if let Some(next) = context.game.edges.get(&node) {
                    if next.len() == nodes.len() {
                        context.game.disjoints.insert(node);
                    }
                }
            }
            ast::Pragma::Repeat {
                nodes, identifiers, ..
            } => {
                let variables: Rc<Vec<_>> = Rc::new(
                    identifiers
                        .into_iter()
                        .filter_map(|identifier| context.variables_indexes.get(&identifier))
                        .cloned()
                        .collect(),
                );

                for node in nodes {
                    context
                        .game
                        .repeats
                        .insert(build_node(node), variables.clone());
                }
            }
            ast::Pragma::Unique { nodes, .. } => {
                for node in nodes {
                    context.game.uniques.insert(build_node(node));
                }
            }
            _ => {}
        }
    }
}

fn build_value(
    context: &mut Context,
    type_: &ist::Type<Id>,
    value: &ast::Value<Id>,
) -> Rc<ist::Value<Id>> {
    match value {
        ast::Value::Element { identifier } => {
            let identifier = identifier.clone();
            context.constants_indexes.get(&identifier).map_or_else(
                || Rc::new(ist::Value::Element { value: identifier }),
                |&index| context.game.constants[index].clone(),
            )
        }
        ast::Value::Map { entries, .. } => {
            let ist::Type::Arrow { rhs, .. } = type_ else {
                panic!("Incorrect Map type found.")
            };

            let default_value = entries.iter().find_map(
                |ast::ValueEntry {
                     identifier, value, ..
                 }| match identifier {
                    Some(_) => None,
                    None => Some(value),
                },
            );

            assert!(default_value.is_some(), "Map is missing default value.");

            Rc::new(ist::Value::Map {
                default: build_value(context, rhs, default_value.unwrap()),
                values: Rc::new(
                    entries
                        .iter()
                        .filter_map(
                            |ast::ValueEntry {
                                 identifier, value, ..
                             }| {
                                identifier.as_ref().map(|identifier| {
                                    (identifier.clone(), build_value(context, rhs, value))
                                })
                            },
                        )
                        .collect::<BTreeMap<_, _>>(),
                ),
            })
        }
    }
}

fn build_type(context: &mut Context, type_: &ast::Type<Id>) -> Option<Rc<ist::Type<Id>>> {
    match type_ {
        ast::Type::Arrow { lhs, rhs } => context
            .types
            .get::<str>(&lhs.to_string())
            .cloned()
            .and_then(|lhs| {
                build_type(context, rhs).map(|rhs| Rc::new(ist::Type::Arrow { lhs, rhs }))
            }),
        ast::Type::Set { identifiers, .. } => Some(Rc::new(ist::Type::Set {
            values: identifiers
                .iter()
                .map(|identifier| {
                    Rc::new(ist::Value::Element {
                        value: identifier.clone(),
                    })
                })
                .collect(),
        })),
        ast::Type::TypeReference { identifier } => context.types.get::<str>(identifier).cloned(),
    }
}

fn build_type_or_fail(context: &mut Context, type_: &ast::Type<Id>) -> Rc<ist::Type<Id>> {
    build_type(context, type_).unwrap_or_else(|| {
        panic!("Unresolved type {type_}. (Builtins are not automatically added yet.)")
    })
}

fn build_typedefs(context: &mut Context, typedefs: Vec<ast::Typedef<Id>>) {
    let typedefs_len = typedefs.len();
    let unresolved_typedefs = typedefs
        .into_iter()
        .filter_map(|typedef| match build_type(context, &typedef.type_) {
            Some(type_) => {
                context.types.insert(typedef.identifier, type_);
                None
            }
            None => Some(typedef),
        })
        .collect::<Vec<_>>();

    if let Some(unresolved_typedef) = unresolved_typedefs.first() {
        assert_ne!(
            typedefs_len,
            unresolved_typedefs.len(),
            "Unresolved type: {unresolved_typedef}"
        );

        build_typedefs(context, unresolved_typedefs);
    }
}

fn build_variables(context: &mut Context, variables: Vec<ast::Variable<Id>>) {
    let mut typed_initial_values: Vec<_> = variables
        .into_iter()
        .filter_map(|variable| {
            let type_ = build_type_or_fail(context, &variable.type_);
            let initial = build_value(context, &type_, &variable.default_value);
            match variable.identifier.as_ref() {
                "goals" => context.game.initial_goals = initial,
                "player" => context.game.initial_player = initial,
                "visible" => context.game.initial_visible = initial,
                _ => return Some((type_.size(), variable.identifier, initial)),
            }

            None
        })
        .collect();

    // Sort by type size. In practice smaller types compare faster and thus want
    // have them earlier in the values list.
    typed_initial_values.sort_unstable();

    for (_, identifier, initial) in typed_initial_values {
        context
            .variables_indexes
            .insert(identifier, context.game.initial_values.len());
        Rc::make_mut(&mut context.game.initial_values).push(initial);
    }
}
