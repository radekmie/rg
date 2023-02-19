use crate::ast;
use crate::ist;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;

impl<Id: Display + Ord> From<ast::GameDeclaration<Id>> for ist::Game<String> {
    fn from(game_declaration: ast::GameDeclaration<Id>) -> Self {
        let mut game = Self {
            constants: BTreeMap::default(),
            edges: BTreeMap::default(),
            pragmas: Vec::default(),
            types: BTreeMap::default(),
            variables: BTreeMap::default(),
        };

        build_pragmas(&mut game, &game_declaration.pragmas);
        build_types(&mut game, &game_declaration.types);
        build_constants(&mut game, &game_declaration.constants);
        build_variables(&mut game, &game_declaration.variables);
        build_edges(&mut game, &game_declaration.edges);

        game
    }
}

fn build_constants<Id: Display + Ord>(
    game: &mut ist::Game<String>,
    constant_declarations: &Vec<Rc<ast::ConstantDeclaration<Id>>>,
) {
    for constant_declaration in constant_declarations {
        let type_ = build_type_or_fail(game, &constant_declaration.type_);
        let value = build_value(game, &type_, &constant_declaration.value);
        game.constants
            .insert(constant_declaration.identifier.to_string(), value);
    }
}

fn build_edge_label<Id: Display>(
    game: &mut ist::Game<String>,
    edge_label: &ast::EdgeLabel<Id>,
) -> ist::EdgeLabel<String> {
    match edge_label {
        ast::EdgeLabel::Assignment { lhs, rhs } => ist::EdgeLabel::Assignment {
            lhs: build_expression(game, lhs),
            rhs: build_expression(game, rhs),
        },
        ast::EdgeLabel::Comparison { lhs, rhs, negated } => ist::EdgeLabel::Comparison {
            lhs: build_expression(game, lhs),
            rhs: build_expression(game, rhs),
            negated: *negated,
        },
        ast::EdgeLabel::Reachability { lhs, rhs, negated } => ist::EdgeLabel::Reachability {
            lhs: build_edge_name(lhs),
            rhs: build_edge_name(rhs),
            negated: *negated,
        },
        ast::EdgeLabel::Skip => ist::EdgeLabel::Skip,
    }
}

fn build_edge_name<Id: Display>(edge_name: &ast::EdgeName<Id>) -> String {
    match &edge_name.parts[..] {
        [edge_name_part] => match &**edge_name_part {
            ast::EdgeNamePart::Literal { identifier } => identifier.to_string(),
            _ => panic!("Only Literal allowed."),
        },
        _ => panic!("Exactly one EdgeNamePart allowed."),
    }
}

fn build_edges<Id: Display>(
    game: &mut ist::Game<String>,
    edges: &Vec<Rc<ast::EdgeDeclaration<Id>>>,
) {
    for edge_declaration in edges {
        let lhs = build_edge_name(&edge_declaration.lhs);
        let rhs = build_edge_name(&edge_declaration.rhs);
        let label = build_edge_label(game, &edge_declaration.label);

        game.edges
            .entry(lhs)
            .or_insert_with(Vec::default)
            .push(ist::Edge { label, next: rhs });
    }
}

fn build_expression<Id: Display>(
    game: &mut ist::Game<String>,
    expression: &ast::Expression<Id>,
) -> ist::Expression<String> {
    match expression {
        ast::Expression::Access { lhs, rhs } => ist::Expression::Access {
            lhs: Rc::new(build_expression(game, lhs)),
            rhs: Rc::new(build_expression(game, rhs)),
        },
        ast::Expression::Cast { rhs, .. } => build_expression(game, rhs),
        ast::Expression::Reference { identifier } => {
            let identifier = identifier.to_string();
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

fn build_pragma<Id: Display>(pragma: &ast::Pragma<Id>) -> ist::Pragma<String> {
    match pragma {
        ast::Pragma::Disjoint { edge_name } => ist::Pragma::Disjoint {
            edge_name: edge_name.to_string(),
        },
    }
}

fn build_pragmas<Id: Display>(game: &mut ist::Game<String>, pragmas: &Vec<Rc<ast::Pragma<Id>>>) {
    for pragma in pragmas {
        game.pragmas.push(build_pragma(pragma));
    }
}

fn build_value<Id: Display + Ord>(
    game: &mut ist::Game<String>,
    type_: &ist::Type<String>,
    value: &ast::Value<Id>,
) -> Rc<ist::Value<String>> {
    match value {
        ast::Value::Element { identifier } => {
            let identifier = identifier.to_string();
            game.constants
                .get(&identifier)
                .cloned()
                .unwrap_or_else(|| Rc::new(ist::Value::Element { value: identifier }))
        }
        ast::Value::Map { entries } => match type_ {
            ist::Type::Arrow { rhs, .. } => {
                let default_values = entries
                    .iter()
                    .flat_map(|entry| match &**entry {
                        ast::ValueEntry::DefaultEntry { value } => Some(value),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                assert_eq!(
                    default_values.len(),
                    1,
                    "Exactly one DefaultEntry required."
                );

                Rc::new(ist::Value::Map {
                    default: build_value(game, rhs, default_values.first().unwrap()),
                    values: Rc::new(
                        entries
                            .iter()
                            .flat_map(|entry| match &**entry {
                                ast::ValueEntry::NamedEntry { identifier, value } => {
                                    Some((identifier.to_string(), build_value(game, rhs, value)))
                                }
                                _ => None,
                            })
                            .collect::<BTreeMap<_, _>>(),
                    ),
                })
            }
            _ => panic!("Incorrect Map type found."),
        },
    }
}

fn build_type<Id: Display>(
    game: &mut ist::Game<String>,
    type_: &ast::Type<Id>,
) -> Option<Rc<ist::Type<String>>> {
    match type_ {
        ast::Type::Arrow { lhs, rhs } => {
            game.types.get(&lhs.to_string()).cloned().and_then(|lhs| {
                build_type(game, rhs).map(|rhs| Rc::new(ist::Type::Arrow { lhs, rhs }))
            })
        }
        ast::Type::Set { identifiers } => Some(Rc::new(ist::Type::Set {
            values: identifiers
                .iter()
                .map(|identifier| {
                    Rc::new(ist::Value::Element {
                        value: identifier.to_string(),
                    })
                })
                .collect(),
        })),
        ast::Type::TypeReference { identifier } => game.types.get(&identifier.to_string()).cloned(),
    }
}

fn build_type_or_fail<Id: Display>(
    game: &mut ist::Game<String>,
    type_: &ast::Type<Id>,
) -> Rc<ist::Type<String>> {
    match build_type(game, type_) {
        Some(type_) => type_,
        None => panic!("Unresolved type {type_}. (Builtins are not automatically added yet.)"),
    }
}

fn build_types<Id: Display>(
    game: &mut ist::Game<String>,
    type_declarations: &Vec<Rc<ast::TypeDeclaration<Id>>>,
) {
    let unresolved_type_declarations = type_declarations
        .iter()
        .flat_map(
            |type_declaration| match build_type(game, &type_declaration.type_) {
                Some(type_) => {
                    game.types
                        .insert(type_declaration.identifier.to_string(), type_);
                    None
                }
                None => Some(type_declaration.clone()),
            },
        )
        .collect::<Vec<_>>();

    if let Some(unresolved_type_declaration) = unresolved_type_declarations.first() {
        assert_ne!(
            type_declarations.len(),
            unresolved_type_declarations.len(),
            "Unresolved type: {}",
            unresolved_type_declaration
        );

        build_types(game, &unresolved_type_declarations);
    }
}

fn build_variables<Id: Display + Ord>(
    game: &mut ist::Game<String>,
    variable_declarations: &Vec<Rc<ast::VariableDeclaration<Id>>>,
) {
    for variable_declaration in variable_declarations {
        let type_ = build_type_or_fail(game, &variable_declaration.type_);
        let default = build_value(game, &type_, &variable_declaration.default_value);
        game.variables.insert(
            variable_declaration.identifier.to_string(),
            ist::Variable { type_, default },
        );
    }
}
