use crate::ast;
use crate::ist;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;

impl<Id: Display + Ord> From<ast::Game<Id>> for ist::Game<Rc<str>> {
    fn from(ast: ast::Game<Id>) -> Self {
        let mut ist = Self {
            constants: BTreeMap::default(),
            edges: BTreeMap::default(),
            pragmas: Vec::default(),
            types: BTreeMap::default(),
            variables: BTreeMap::default(),
        };

        build_pragmas(&mut ist, ast.pragmas);
        build_typedefs(&mut ist, ast.typedefs);
        build_constants(&mut ist, ast.constants);
        build_variables(&mut ist, ast.variables);
        build_edges(&mut ist, ast.edges);

        ist
    }
}

fn build_constants<Id: Display + Ord>(
    game: &mut ist::Game<Rc<str>>,
    constants: Vec<ast::Constant<Id>>,
) {
    for constant in constants {
        let type_ = build_type_or_fail(game, &constant.type_);
        let value = build_value(game, &type_, &constant.value);
        game.constants
            .insert(Rc::from(constant.identifier.to_string()), value);
    }
}

fn build_edge_label<Id: Display>(
    game: &mut ist::Game<Rc<str>>,
    edge_label: ast::EdgeLabel<Id>,
) -> ist::EdgeLabel<Rc<str>> {
    match edge_label {
        ast::EdgeLabel::Assignment { lhs, rhs } => ist::EdgeLabel::Assignment {
            lhs: build_expression(game, &lhs),
            rhs: build_expression(game, &rhs),
        },
        ast::EdgeLabel::Comparison { lhs, rhs, negated } => ist::EdgeLabel::Comparison {
            lhs: build_expression(game, &lhs),
            rhs: build_expression(game, &rhs),
            negated,
        },
        ast::EdgeLabel::Reachability {
            lhs, rhs, negated, ..
        } => ist::EdgeLabel::Reachability {
            lhs: build_edge_name(lhs),
            rhs: build_edge_name(rhs),
            negated,
        },
        ast::EdgeLabel::Skip { .. } => ist::EdgeLabel::Skip,
        ast::EdgeLabel::Tag { symbol } => ist::EdgeLabel::Tag {
            symbol: Rc::from(symbol.to_string()),
        },
    }
}

fn build_edge_name<Id: Display>(edge_name: ast::EdgeName<Id>) -> Rc<str> {
    let [ast::EdgeNamePart::Literal { identifier }] = &edge_name.parts[..] else {
        panic!("Only trivial EdgeName allowed.")
    };
    Rc::from(identifier.to_string())
}

fn build_edges<Id: Display>(game: &mut ist::Game<Rc<str>>, edges: Vec<ast::Edge<Id>>) {
    for edge in edges {
        let lhs = build_edge_name(edge.lhs);
        let rhs = build_edge_name(edge.rhs);
        let label = build_edge_label(game, edge.label);

        game.edges
            .entry(lhs)
            .or_insert_with(Vec::default)
            .push(ist::Edge { label, next: rhs });
    }
}

fn build_expression<Id: Display>(
    game: &mut ist::Game<Rc<str>>,
    expression: &ast::Expression<Id>,
) -> ist::Expression<Rc<str>> {
    match expression {
        ast::Expression::Access { lhs, rhs, .. } => ist::Expression::Access {
            lhs: Rc::new(build_expression(game, lhs)),
            rhs: Rc::new(build_expression(game, rhs)),
        },
        ast::Expression::Cast { rhs, .. } => build_expression(game, rhs),
        ast::Expression::Reference { identifier } => {
            let identifier = Rc::from(identifier.to_string());
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

fn build_pragma<Id: Display>(pragma: ast::Pragma<Id>) -> ist::Pragma<Rc<str>> {
    match pragma {
        ast::Pragma::Any { edge_name, .. } => ist::Pragma::Any {
            edge_name: Rc::from(edge_name.to_string()),
        },
        ast::Pragma::Disjoint { edge_name, .. } => ist::Pragma::Disjoint {
            edge_name: Rc::from(edge_name.to_string()),
        },
        ast::Pragma::MultiAny { edge_name, .. } => ist::Pragma::MultiAny {
            edge_name: Rc::from(edge_name.to_string()),
        },
        ast::Pragma::Unique { edge_name, .. } => ist::Pragma::Unique {
            edge_name: Rc::from(edge_name.to_string()),
        },
    }
}

fn build_pragmas<Id: Display>(game: &mut ist::Game<Rc<str>>, pragmas: Vec<ast::Pragma<Id>>) {
    for pragma in pragmas {
        game.pragmas.push(build_pragma(pragma));
    }
}

fn build_value<Id: Display + Ord>(
    game: &mut ist::Game<Rc<str>>,
    type_: &ist::Type<Rc<str>>,
    value: &ast::Value<Id>,
) -> Rc<ist::Value<Rc<str>>> {
    match value {
        ast::Value::Element { identifier } => {
            let identifier = Rc::from(identifier.to_string());
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
                        .flat_map(
                            |ast::ValueEntry {
                                 identifier, value, ..
                             }| {
                                identifier.as_ref().map(|identifier| {
                                    (
                                        Rc::from(identifier.to_string()),
                                        build_value(game, rhs, value),
                                    )
                                })
                            },
                        )
                        .collect::<BTreeMap<_, _>>(),
                ),
            })
        }
    }
}

fn build_type<Id: Display>(
    game: &mut ist::Game<Rc<str>>,
    type_: &ast::Type<Id>,
) -> Option<Rc<ist::Type<Rc<str>>>> {
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
                        value: Rc::from(identifier.to_string()),
                    })
                })
                .collect(),
        })),
        ast::Type::TypeReference { identifier } => {
            game.types.get::<str>(&identifier.to_string()).cloned()
        }
    }
}

fn build_type_or_fail<Id: Display>(
    game: &mut ist::Game<Rc<str>>,
    type_: &ast::Type<Id>,
) -> Rc<ist::Type<Rc<str>>> {
    match build_type(game, type_) {
        Some(type_) => type_,
        None => panic!("Unresolved type {type_}. (Builtins are not automatically added yet.)"),
    }
}

fn build_typedefs<Id: Display>(game: &mut ist::Game<Rc<str>>, typedefs: Vec<ast::Typedef<Id>>) {
    let typedefs_len = typedefs.len();
    let unresolved_typedefs = typedefs
        .into_iter()
        .flat_map(|typedef| match build_type(game, &typedef.type_) {
            Some(type_) => {
                game.types
                    .insert(Rc::from(typedef.identifier.to_string()), type_);
                None
            }
            None => Some(typedef),
        })
        .collect::<Vec<_>>();

    if let Some(unresolved_typedef) = unresolved_typedefs.first() {
        assert_ne!(
            typedefs_len,
            unresolved_typedefs.len(),
            "Unresolved type: {}",
            unresolved_typedef
        );

        build_typedefs(game, unresolved_typedefs);
    }
}

fn build_variables<Id: Display + Ord>(
    game: &mut ist::Game<Rc<str>>,
    variables: Vec<ast::Variable<Id>>,
) {
    for variable in variables {
        let type_ = build_type_or_fail(game, &variable.type_);
        let default = build_value(game, &type_, &variable.default_value);
        game.variables.insert(
            Rc::from(variable.identifier.to_string()),
            ist::Variable { type_, default },
        );
    }
}
