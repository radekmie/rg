use crate::ist;
use rg::ast;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Arc;

impl From<ast::Game<Arc<str>>> for ist::Game<Arc<str>> {
    fn from(ast: ast::Game<Arc<str>>) -> Self {
        let mut ist = Self {
            constants: BTreeMap::default(),
            edges: BTreeMap::default(),
            types: BTreeMap::default(),
            uniques: BTreeSet::default(),
            variables: BTreeMap::default(),
        };

        build_uniques(&mut ist, ast.pragmas);
        build_typedefs(&mut ist, ast.typedefs);
        build_constants(&mut ist, ast.constants);
        build_variables(&mut ist, ast.variables);
        build_edges(&mut ist, ast.edges);

        ist
    }
}

fn build_constants(game: &mut ist::Game<Arc<str>>, constants: Vec<ast::Constant<Arc<str>>>) {
    for constant in constants {
        let type_ = build_type_or_fail(game, &constant.type_);
        let value = build_value(game, &type_, &constant.value);
        game.constants.insert(constant.identifier, value);
    }
}

fn build_label(
    game: &mut ist::Game<Arc<str>>,
    label: ast::Label<Arc<str>>,
) -> ist::EdgeLabel<Arc<str>> {
    match label {
        ast::Label::Assignment { lhs, rhs } => ist::EdgeLabel::Assignment {
            lhs: build_expression(game, &lhs),
            rhs: build_expression(game, &rhs),
        },
        ast::Label::Comparison { lhs, rhs, negated } => ist::EdgeLabel::Comparison {
            lhs: build_expression(game, &lhs),
            rhs: build_expression(game, &rhs),
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

fn build_node(mut node: ast::Node<Arc<str>>) -> Arc<str> {
    assert!(node.parts.len() == 1, "Only trivial EdgeName allowed.");
    let Some(ast::NodePart::Literal { identifier }) = node.parts.pop() else {
        panic!("Only trivial EdgeName allowed.")
    };
    identifier
}

fn build_edges(game: &mut ist::Game<Arc<str>>, edges: Vec<ast::Edge<Arc<str>>>) {
    for edge in edges {
        let lhs = build_node(edge.lhs);
        let rhs = build_node(edge.rhs);
        let label = build_label(game, edge.label);

        game.edges
            .entry(lhs)
            .or_default()
            .push(ist::Edge { label, next: rhs });
    }
}

fn build_expression(
    game: &mut ist::Game<Arc<str>>,
    expression: &ast::Expression<Arc<str>>,
) -> ist::Expression<Arc<str>> {
    match expression {
        ast::Expression::Access { lhs, rhs, .. } => ist::Expression::Access {
            lhs: Rc::new(build_expression(game, lhs)),
            rhs: Rc::new(build_expression(game, rhs)),
        },
        ast::Expression::Cast { rhs, .. } => build_expression(game, rhs),
        ast::Expression::Reference { identifier } => {
            let identifier = identifier.clone();
            if game.constants.contains_key(&identifier) {
                return ist::Expression::ConstantReference { identifier };
            }

            if game.variables.contains_key(&identifier) {
                return ist::Expression::VariableReference { identifier };
            }

            ist::Expression::Literal {
                value: Rc::new(ist::Value::Element { value: identifier }),
            }
        }
    }
}

fn build_value(
    game: &mut ist::Game<Arc<str>>,
    type_: &ist::Type<Arc<str>>,
    value: &ast::Value<Arc<str>>,
) -> Rc<ist::Value<Arc<str>>> {
    match value {
        ast::Value::Element { identifier } => {
            let identifier = identifier.clone();
            game.constants
                .get(&identifier)
                .cloned()
                .unwrap_or_else(|| Rc::new(ist::Value::Element { value: identifier }))
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
                default: build_value(game, rhs, default_value.unwrap()),
                values: Rc::new(
                    entries
                        .iter()
                        .filter_map(
                            |ast::ValueEntry {
                                 identifier, value, ..
                             }| {
                                identifier.as_ref().map(|identifier| {
                                    (identifier.clone(), build_value(game, rhs, value))
                                })
                            },
                        )
                        .collect::<BTreeMap<_, _>>(),
                ),
            })
        }
    }
}

fn build_type(
    game: &mut ist::Game<Arc<str>>,
    type_: &ast::Type<Arc<str>>,
) -> Option<Rc<ist::Type<Arc<str>>>> {
    match type_ {
        ast::Type::Arrow { lhs, rhs } => {
            game.types
                .get::<str>(&lhs.to_string())
                .cloned()
                .and_then(|lhs| {
                    build_type(game, rhs).map(|rhs| Rc::new(ist::Type::Arrow { lhs, rhs }))
                })
        }
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
        ast::Type::TypeReference { identifier } => game.types.get::<str>(identifier).cloned(),
    }
}

fn build_type_or_fail(
    game: &mut ist::Game<Arc<str>>,
    type_: &ast::Type<Arc<str>>,
) -> Rc<ist::Type<Arc<str>>> {
    build_type(game, type_).unwrap_or_else(|| {
        panic!("Unresolved type {type_}. (Builtins are not automatically added yet.)")
    })
}

fn build_typedefs(game: &mut ist::Game<Arc<str>>, typedefs: Vec<ast::Typedef<Arc<str>>>) {
    let typedefs_len = typedefs.len();
    let unresolved_typedefs = typedefs
        .into_iter()
        .filter_map(|typedef| match build_type(game, &typedef.type_) {
            Some(type_) => {
                game.types.insert(typedef.identifier, type_);
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

        build_typedefs(game, unresolved_typedefs);
    }
}

fn build_uniques(game: &mut ist::Game<Arc<str>>, pragmas: Vec<ast::Pragma<Arc<str>>>) {
    for pragma in pragmas {
        if let ast::Pragma::Unique { nodes, .. } = pragma {
            for node in nodes {
                game.uniques.insert(build_node(node));
            }
        }
    }
}

fn build_variables(game: &mut ist::Game<Arc<str>>, variables: Vec<ast::Variable<Arc<str>>>) {
    for variable in variables {
        let type_ = build_type_or_fail(game, &variable.type_);
        let default = build_value(game, &type_, &variable.default_value);
        game.variables
            .insert(variable.identifier, ist::Variable { default, type_ });
    }
}
