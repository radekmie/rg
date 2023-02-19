use rg::ast::{GameDeclaration, Type, TypeDeclaration, Value, ValueEntry, VariableDeclaration};
use std::rc::Rc;

pub fn add_builtins(game_declaration: &mut GameDeclaration<String>) -> Result<(), String> {
    // |- Bool
    add_builtin_type(
        game_declaration,
        TypeDeclaration {
            identifier: "Bool".to_string(),
            type_: Rc::new(Type::Set {
                identifiers: vec!["0".to_string(), "1".to_string()],
            }),
        },
    )?;

    // Player ^ Score |- Goals
    game_declaration.resolve_type(&"Score".to_string())?;
    add_builtin_type(
        game_declaration,
        TypeDeclaration {
            identifier: "Goals".to_string(),
            type_: Rc::new(Type::Arrow {
                lhs: "Player".to_string(),
                rhs: Rc::new(Type::TypeReference {
                    identifier: "Score".to_string(),
                }),
            }),
        },
    )?;

    // Player |- Visibility
    game_declaration.resolve_type(&"Player".to_string())?;
    add_builtin_type(
        game_declaration,
        TypeDeclaration {
            identifier: "Visibility".to_string(),
            type_: Rc::new(Type::Arrow {
                lhs: "Player".to_string(),
                rhs: Rc::new(Type::TypeReference {
                    identifier: "Bool".to_string(),
                }),
            }),
        },
    )?;

    // Player ^ isSet(Player) |- PlayerOrKeeper
    let players = match &*game_declaration.resolve_type(&"Player".to_string())?.type_ {
        Type::Set { identifiers } => {
            if identifiers.contains(&"keeper".to_string()) {
                identifiers.clone()
            } else {
                let mut identifiers = identifiers.clone();
                identifiers.push("keeper".to_string());
                identifiers
            }
        }
        _ => {
            return Err(game_declaration.type_error_context("Type Player should be a Set.".into()));
        }
    };
    add_builtin_type(
        game_declaration,
        TypeDeclaration {
            identifier: "PlayerOrKeeper".to_string(),
            type_: Rc::new(Type::Set {
                identifiers: players,
            }),
        },
    )?;

    // Goals ^ Score ^ isSet(Score) |- goals
    game_declaration.resolve_type(&"Goals".to_string())?;
    let default_score = match &*game_declaration.resolve_type(&"Score".to_string())?.type_ {
        Type::Set { identifiers } => identifiers.first().ok_or_else(|| {
            game_declaration.type_error_context("Type Score should not be empty.".into())
        })?,
        _ => {
            return Err(game_declaration.type_error_context("Type Score should be a Set.".into()));
        }
    };
    add_builtin_variable(
        game_declaration,
        VariableDeclaration {
            identifier: "goals".to_string(),
            type_: Rc::new(Type::TypeReference {
                identifier: "Goals".to_string(),
            }),
            default_value: Rc::new(Value::Map {
                entries: vec![Rc::new(ValueEntry::DefaultEntry {
                    value: Rc::new(Value::Element {
                        identifier: default_score.clone(),
                    }),
                })],
            }),
        },
    )?;

    // PlayerOrKeeper |- player
    game_declaration.resolve_type(&"PlayerOrKeeper".to_string())?;
    add_builtin_variable(
        game_declaration,
        VariableDeclaration {
            identifier: "player".to_string(),
            type_: Rc::new(Type::TypeReference {
                identifier: "PlayerOrKeeper".to_string(),
            }),
            default_value: Rc::new(Value::Element {
                identifier: "keeper".to_string(),
            }),
        },
    )?;

    // Visibility |- visibility
    game_declaration.resolve_type(&"Visibility".to_string())?;
    add_builtin_variable(
        game_declaration,
        VariableDeclaration {
            identifier: "visible".to_string(),
            type_: Rc::new(Type::TypeReference {
                identifier: "Visibility".to_string(),
            }),
            default_value: Rc::new(Value::Map {
                entries: vec![Rc::new(ValueEntry::DefaultEntry {
                    value: Rc::new(Value::Element {
                        identifier: "1".to_string(),
                    }),
                })],
            }),
        },
    )?;

    Ok(())
}

fn add_builtin_type(
    game_declaration: &mut GameDeclaration<String>,
    type_declaration: TypeDeclaration<String>,
) -> Result<(), String> {
    match game_declaration.resolve_type(&type_declaration.identifier) {
        Ok(resolved_type_declaration) => {
            if !game_declaration.is_equal_type(
                &type_declaration.type_,
                &resolved_type_declaration.type_,
                false,
            )? {
                return Err(game_declaration.type_error_context(format!(
                    "Incorrect builtin type {}.",
                    type_declaration.identifier
                )));
            }
        }
        Err(_) => {
            game_declaration.types.push(Rc::new(type_declaration));
        }
    }
    Ok(())
}

fn add_builtin_variable(
    game_declaration: &mut GameDeclaration<String>,
    variable_declaration: VariableDeclaration<String>,
) -> Result<(), String> {
    match game_declaration.resolve_variable(&variable_declaration.identifier) {
        Ok(resolved_variable_declaration) => {
            if !game_declaration.is_equal_type(
                &variable_declaration.type_,
                &resolved_variable_declaration.type_,
                false,
            )? {
                return Err(game_declaration.type_error_context(format!(
                    "Incorrect builtin variable {}.",
                    variable_declaration.identifier
                )));
            }
        }
        Err(_) => {
            game_declaration
                .variables
                .push(Rc::new(variable_declaration));
        }
    }
    Ok(())
}
