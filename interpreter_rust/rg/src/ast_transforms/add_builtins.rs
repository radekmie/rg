use crate::ast::{Error, ErrorReason, Game, Type, Typedef, Value, ValueEntry, Variable};
use std::rc::Rc;

impl Game<String> {
    pub fn add_builtins(mut self) -> Result<Self, Error<String>> {
        // |- Bool
        self.add_builtin_type(Typedef {
            identifier: "Bool".to_string(),
            type_: Rc::new(Type::from(vec!["0".to_string(), "1".to_string()])),
        })?;

        // Player ^ Score |- Goals
        self.resolve_typedef(&"Score".to_string())?;
        self.add_builtin_type(Typedef {
            identifier: "Goals".to_string(),
            type_: Rc::new(Type::Arrow {
                lhs: "Player".to_string(),
                rhs: Rc::new(Type::from("Score".to_string())),
            }),
        })?;

        // Player |- Visibility
        self.resolve_typedef(&"Player".to_string())?;
        self.add_builtin_type(Typedef {
            identifier: "Visibility".to_string(),
            type_: Rc::new(Type::Arrow {
                lhs: "Player".to_string(),
                rhs: Rc::new(Type::from("Bool".to_string())),
            }),
        })?;

        // Player ^ isSet(Player) |- PlayerOrKeeper
        let player_type = &self.resolve_typedef(&"Player".to_string())?.type_;
        let players = match &**player_type {
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
                return self.make_error(ErrorReason::SetTypeExpected {
                    got: player_type.clone(),
                });
            }
        };
        self.add_builtin_type(Typedef {
            identifier: "PlayerOrKeeper".to_string(),
            type_: Rc::new(Type::from(players)),
        })?;

        // Goals ^ Score ^ isSet(Score) |- goals
        self.resolve_typedef(&"Goals".to_string())?;
        let score_type = &self.resolve_typedef(&"Score".to_string())?.type_;
        let default_score = match &**score_type {
            Type::Set { identifiers } => identifiers.first().cloned().map_or_else(
                || {
                    self.make_error(ErrorReason::EmptySetType {
                        identifier: "Score".to_string(),
                    })
                },
                Ok,
            )?,
            _ => {
                return self.make_error(ErrorReason::SetTypeExpected {
                    got: score_type.clone(),
                });
            }
        };
        self.add_builtin_variable(Variable {
            identifier: "goals".to_string(),
            type_: Rc::new(Type::from("Goals".to_string())),
            default_value: Rc::new(Value::Map {
                entries: vec![ValueEntry::DefaultEntry {
                    value: Rc::new(Value::from(default_score)),
                }],
            }),
        })?;

        // PlayerOrKeeper |- player
        self.resolve_typedef(&"PlayerOrKeeper".to_string())?;
        self.add_builtin_variable(Variable {
            identifier: "player".to_string(),
            type_: Rc::new(Type::from("PlayerOrKeeper".to_string())),
            default_value: Rc::new(Value::from("keeper".to_string())),
        })?;

        // Visibility |- visibility
        self.resolve_typedef(&"Visibility".to_string())?;
        self.add_builtin_variable(Variable {
            identifier: "visible".to_string(),
            type_: Rc::new(Type::from("Visibility".to_string())),
            default_value: Rc::new(Value::Map {
                entries: vec![ValueEntry::DefaultEntry {
                    value: Rc::new(Value::from("1".to_string())),
                }],
            }),
        })?;

        Ok(self)
    }

    fn add_builtin_type(&mut self, builtin: Typedef<String>) -> Result<(), Error<String>> {
        if let Ok(defined) = self.resolve_typedef(&builtin.identifier) {
            if !self.is_equal_type(&builtin.type_, &defined.type_, false)? {
                return self.make_error(ErrorReason::TypeDeclarationMismatch {
                    identifier: builtin.identifier,
                    expected: builtin.type_,
                    resolved: defined.type_.clone(),
                });
            }
        } else {
            self.typedefs.push(builtin);
        }
        Ok(())
    }

    fn add_builtin_variable(&mut self, builtin: Variable<String>) -> Result<(), Error<String>> {
        if let Ok(defined) = self.resolve_variable(&builtin.identifier) {
            if !self.is_equal_type(&builtin.type_, &defined.type_, false)? {
                return self.make_error(ErrorReason::VariableDeclarationMismatch {
                    identifier: builtin.identifier,
                    expected: builtin.type_,
                    resolved: defined.type_.clone(),
                });
            }
        } else {
            self.variables.push(builtin);
        }
        Ok(())
    }
}
