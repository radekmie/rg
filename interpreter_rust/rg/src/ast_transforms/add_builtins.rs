use crate::{
    ast::{Error, ErrorReason, Game, Type, Typedef, Value, ValueEntry, Variable},
    position::Span,
};
use std::{rc::Rc, sync::Arc};

impl Game<Rc<str>> {
    pub fn add_builtins(&mut self) -> Result<(), Error<Rc<str>>> {
        // |- Bool
        self.add_builtin_type(Typedef {
            span: Span::none(),
            identifier: Rc::from("Bool"),
            type_: Arc::new(Type::Set {
                span: Span::none(),
                identifiers: vec![Rc::from("0"), Rc::from("1")],
            }),
        })?;

        // Player ^ Score |- Goals
        self.resolve_typedef_or_fail(&Rc::from("Score"))?;
        self.add_builtin_type(Typedef {
            span: Span::none(),
            identifier: Rc::from("Goals"),
            type_: Arc::new(Type::Arrow {
                lhs: Arc::new(Type::from(Rc::from("Player"))),
                rhs: Arc::new(Type::from(Rc::from("Score"))),
            }),
        })?;

        // Player |- Visibility
        self.resolve_typedef_or_fail(&Rc::from("Player"))?;
        self.add_builtin_type(Typedef {
            span: Span::none(),
            identifier: Rc::from("Visibility"),
            type_: Arc::new(Type::Arrow {
                lhs: Arc::new(Type::from(Rc::from("Player"))),
                rhs: Arc::new(Type::from(Rc::from("Bool"))),
            }),
        })?;

        // Player ^ isSet(Player) |- PlayerOrKeeper
        let player_type = &self.resolve_typedef_or_fail(&Rc::from("Player"))?.type_;
        let Type::Set { identifiers, .. } = &**player_type else {
            return self.make_error(ErrorReason::SetTypeExpected {
                got: player_type.clone(),
            });
        };
        let players = if identifiers.contains(&Rc::from("keeper")) {
            identifiers.clone()
        } else {
            let mut identifiers = identifiers.clone();
            identifiers.push(Rc::from("keeper"));
            identifiers
        };
        self.add_builtin_type(Typedef {
            span: Span::none(),
            identifier: Rc::from("PlayerOrKeeper"),
            type_: Arc::new(Type::Set {
                span: Span::none(),
                identifiers: players,
            }),
        })?;

        // Goals ^ Score ^ isSet(Score) |- goals
        self.resolve_typedef_or_fail(&Rc::from("Goals"))?;
        let score_type = &self.resolve_typedef_or_fail(&Rc::from("Score"))?.type_;
        let Type::Set { identifiers, .. } = &**score_type else {
            return self.make_error(ErrorReason::SetTypeExpected {
                got: score_type.clone(),
            });
        };
        let Some(default_score) = identifiers.first().cloned() else {
            return self.make_error(ErrorReason::EmptySetType {
                identifier: Rc::from("Score"),
            });
        };
        self.add_builtin_variable(Variable {
            span: Span::none(),
            identifier: Rc::from("goals"),
            type_: Arc::new(Type::from(Rc::from("Goals"))),
            default_value: Arc::new(Value::Map {
                span: Span::none(),
                entries: vec![ValueEntry {
                    span: Span::none(),
                    identifier: None,
                    value: Arc::new(Value::from(default_score)),
                }],
            }),
        })?;

        // PlayerOrKeeper |- player
        self.resolve_typedef_or_fail(&Rc::from("PlayerOrKeeper"))?;
        self.add_builtin_variable(Variable {
            span: Span::none(),
            identifier: Rc::from("player"),
            type_: Arc::new(Type::from(Rc::from("PlayerOrKeeper"))),
            default_value: Arc::new(Value::from(Rc::from("keeper"))),
        })?;

        // Visibility |- visibility
        self.resolve_typedef_or_fail(&Rc::from("Visibility"))?;
        self.add_builtin_variable(Variable {
            span: Span::none(),
            identifier: Rc::from("visible"),
            type_: Arc::new(Type::from(Rc::from("Visibility"))),
            default_value: Arc::new(Value::Map {
                span: Span::none(),
                entries: vec![ValueEntry {
                    span: Span::none(),
                    identifier: None,
                    value: Arc::new(Value::from(Rc::from("1"))),
                }],
            }),
        })?;

        Ok(())
    }

    fn add_builtin_type(&mut self, builtin: Typedef<Rc<str>>) -> Result<(), Error<Rc<str>>> {
        if let Some(defined) = self.resolve_typedef(&builtin.identifier) {
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
        if let Some(defined) = self.resolve_variable(&builtin.identifier) {
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
