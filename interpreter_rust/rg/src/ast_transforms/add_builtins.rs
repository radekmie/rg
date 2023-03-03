use crate::ast::{Error, ErrorReason, Game, Type, Typedef, Value, ValueEntry, Variable};
use std::rc::Rc;

impl Game<Rc<str>> {
    pub fn add_builtins(&mut self) -> Result<(), Error<Rc<str>>> {
        // |- Bool
        self.add_builtin_type(Typedef {
            identifier: Rc::from("Bool"),
            type_: Rc::new(Type::from(vec![Rc::from("0"), Rc::from("1")])),
        })?;

        // Player ^ Score |- Goals
        self.resolve_typedef(&Rc::from("Score"))?;
        self.add_builtin_type(Typedef {
            identifier: Rc::from("Goals"),
            type_: Rc::new(Type::Arrow {
                lhs: Rc::new(Type::from(Rc::from("Player"))),
                rhs: Rc::new(Type::from(Rc::from("Score"))),
            }),
        })?;

        // Player |- Visibility
        self.resolve_typedef(&Rc::from("Player"))?;
        self.add_builtin_type(Typedef {
            identifier: Rc::from("Visibility"),
            type_: Rc::new(Type::Arrow {
                lhs: Rc::new(Type::from(Rc::from("Player"))),
                rhs: Rc::new(Type::from(Rc::from("Bool"))),
            }),
        })?;

        // Player ^ isSet(Player) |- PlayerOrKeeper
        let player_type = &self.resolve_typedef(&Rc::from("Player"))?.type_;
        let players = match &**player_type {
            Type::Set { identifiers } => {
                if identifiers.contains(&Rc::from("keeper")) {
                    identifiers.clone()
                } else {
                    let mut identifiers = identifiers.clone();
                    identifiers.push(Rc::from("keeper"));
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
            identifier: Rc::from("PlayerOrKeeper"),
            type_: Rc::new(Type::from(players)),
        })?;

        // Goals ^ Score ^ isSet(Score) |- goals
        self.resolve_typedef(&Rc::from("Goals"))?;
        let score_type = &self.resolve_typedef(&Rc::from("Score"))?.type_;
        let default_score = match &**score_type {
            Type::Set { identifiers } => identifiers.first().cloned().map_or_else(
                || {
                    self.make_error(ErrorReason::EmptySetType {
                        identifier: Rc::from("Score"),
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
            identifier: Rc::from("goals"),
            type_: Rc::new(Type::from(Rc::from("Goals"))),
            default_value: Rc::new(Value::Map {
                entries: vec![ValueEntry::DefaultEntry {
                    value: Rc::new(Value::from(default_score)),
                }],
            }),
        })?;

        // PlayerOrKeeper |- player
        self.resolve_typedef(&Rc::from("PlayerOrKeeper"))?;
        self.add_builtin_variable(Variable {
            identifier: Rc::from("player"),
            type_: Rc::new(Type::from(Rc::from("PlayerOrKeeper"))),
            default_value: Rc::new(Value::from(Rc::from("keeper"))),
        })?;

        // Visibility |- visibility
        self.resolve_typedef(&Rc::from("Visibility"))?;
        self.add_builtin_variable(Variable {
            identifier: Rc::from("visible"),
            type_: Rc::new(Type::from(Rc::from("Visibility"))),
            default_value: Rc::new(Value::Map {
                entries: vec![ValueEntry::DefaultEntry {
                    value: Rc::new(Value::from(Rc::from("1"))),
                }],
            }),
        })?;

        Ok(())
    }

    fn add_builtin_type(&mut self, builtin: Typedef<Rc<str>>) -> Result<(), Error<Rc<str>>> {
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

    fn add_builtin_variable(&mut self, builtin: Variable<Rc<str>>) -> Result<(), Error<Rc<str>>> {
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
