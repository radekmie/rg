use crate::ast::{
    Error, ErrorReason, GameDeclaration, Type, TypeDeclaration, Value, ValueEntry,
    VariableDeclaration,
};
use std::rc::Rc;

impl GameDeclaration<String> {
    pub fn add_builtins(mut self) -> Result<Self, Error<String>> {
        // |- Bool
        self.add_builtin_type(TypeDeclaration {
            identifier: "Bool".to_string(),
            type_: Rc::new(Type::Set {
                identifiers: vec!["0".to_string(), "1".to_string()],
            }),
        })?;

        // Player ^ Score |- Goals
        self.resolve_type(&"Score".to_string())?;
        self.add_builtin_type(TypeDeclaration {
            identifier: "Goals".to_string(),
            type_: Rc::new(Type::Arrow {
                lhs: "Player".to_string(),
                rhs: Rc::new(Type::TypeReference {
                    identifier: "Score".to_string(),
                }),
            }),
        })?;

        // Player |- Visibility
        self.resolve_type(&"Player".to_string())?;
        self.add_builtin_type(TypeDeclaration {
            identifier: "Visibility".to_string(),
            type_: Rc::new(Type::Arrow {
                lhs: "Player".to_string(),
                rhs: Rc::new(Type::TypeReference {
                    identifier: "Bool".to_string(),
                }),
            }),
        })?;

        // Player ^ isSet(Player) |- PlayerOrKeeper
        let players = match &*self.resolve_type(&"Player".to_string())?.type_ {
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
                return Err(self.make_error(ErrorReason::SetTypeExpected {
                    identifier: "Player".to_string(),
                }));
            }
        };
        self.add_builtin_type(TypeDeclaration {
            identifier: "PlayerOrKeeper".to_string(),
            type_: Rc::new(Type::Set {
                identifiers: players,
            }),
        })?;

        // Goals ^ Score ^ isSet(Score) |- goals
        self.resolve_type(&"Goals".to_string())?;
        let default_score = match &*self.resolve_type(&"Score".to_string())?.type_ {
            Type::Set { identifiers } => identifiers.first().cloned().ok_or_else(|| {
                self.make_error(ErrorReason::EmptySetType {
                    identifier: "Score".to_string(),
                })
            })?,
            _ => {
                return Err(self.make_error(ErrorReason::SetTypeExpected {
                    identifier: "Score".to_string(),
                }));
            }
        };
        self.add_builtin_variable(VariableDeclaration {
            identifier: "goals".to_string(),
            type_: Rc::new(Type::TypeReference {
                identifier: "Goals".to_string(),
            }),
            default_value: Rc::new(Value::Map {
                entries: vec![Rc::new(ValueEntry::DefaultEntry {
                    value: Rc::new(Value::Element {
                        identifier: default_score,
                    }),
                })],
            }),
        })?;

        // PlayerOrKeeper |- player
        self.resolve_type(&"PlayerOrKeeper".to_string())?;
        self.add_builtin_variable(VariableDeclaration {
            identifier: "player".to_string(),
            type_: Rc::new(Type::TypeReference {
                identifier: "PlayerOrKeeper".to_string(),
            }),
            default_value: Rc::new(Value::Element {
                identifier: "keeper".to_string(),
            }),
        })?;

        // Visibility |- visibility
        self.resolve_type(&"Visibility".to_string())?;
        self.add_builtin_variable(VariableDeclaration {
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
        })?;

        Ok(self)
    }

    fn add_builtin_type(&mut self, builtin: TypeDeclaration<String>) -> Result<(), Error<String>> {
        if let Ok(defined) = self.resolve_type(&builtin.identifier) {
            if !self.is_equal_type(&builtin.type_, &defined.type_, false)? {
                return Err(self.make_error(ErrorReason::TypeDeclarationMismatch {
                    identifier: builtin.identifier,
                    expected: builtin.type_,
                    resolved: defined.type_.clone(),
                }));
            }
        } else {
            self.types.push(Rc::new(builtin));
        }
        Ok(())
    }

    fn add_builtin_variable(
        &mut self,
        builtin: VariableDeclaration<String>,
    ) -> Result<(), Error<String>> {
        if let Ok(defined) = self.resolve_variable(&builtin.identifier) {
            if !self.is_equal_type(&builtin.type_, &defined.type_, false)? {
                return Err(self.make_error(ErrorReason::VariableDeclarationMismatch {
                    identifier: builtin.identifier,
                    expected: builtin.type_,
                    resolved: defined.type_.clone(),
                }));
            }
        } else {
            self.variables.push(Rc::new(builtin));
        }
        Ok(())
    }
}
