use crate::ast::{Constant, Error, Game, Type, Typedef, Variable};
use std::rc::Rc;

impl Constant<String> {
    pub fn normalize_type(&self, game: &mut Game<String>) -> Result<Self, Error<String>> {
        Ok(Self {
            identifier: self.identifier.clone(),
            type_: Rc::new(self.type_.normalize(game)?),
            value: self.value.clone(),
        })
    }
}

impl Game<String> {
    pub fn normalize_types(mut self) -> Result<Self, Error<String>> {
        for (index, typedef) in self.typedefs.clone().into_iter().enumerate() {
            self.typedefs[index] = typedef.normalize_type(&mut self)?;
        }

        for (index, constant) in self.constants.clone().into_iter().enumerate() {
            self.constants[index] = constant.normalize_type(&mut self)?;
        }

        for (index, variable) in self.variables.clone().into_iter().enumerate() {
            self.variables[index] = variable.normalize_type(&mut self)?;
        }

        Ok(self)
    }
}

impl Type<String> {
    pub fn normalize(&self, game: &mut Game<String>) -> Result<Self, Error<String>> {
        if matches!(self, Self::TypeReference { .. }) {
            return Ok(self.clone());
        }

        let self_normalized = match self {
            Self::Arrow { lhs, rhs } => Self::Arrow {
                lhs: lhs.clone(),
                rhs: Rc::new(rhs.normalize(game)?),
            },
            _ => self.clone(),
        };

        if let Some(typedef) = game.resolve_typedef_by_type(&self_normalized)? {
            return Ok(Self::TypeReference {
                identifier: typedef.identifier.clone(),
            });
        }

        let normalized_type_name = "NormalizedType";
        let identifier = match self_normalized {
            Self::Arrow { ref lhs, ref rhs } => match &**rhs {
                Self::TypeReference { identifier } => Some(format!("{lhs}_{identifier}")),
                _ => None,
            },
            _ => None,
        }
        .unwrap_or_else(|| normalized_type_name.to_string());

        let mut index: Option<usize> = None;
        let identifier = loop {
            let identifier_with_index = format!(
                "{identifier}{}",
                index.map_or("".to_string(), |index| index.to_string())
            );

            if !game
                .typedefs
                .iter()
                .any(|typedef| typedef.identifier == identifier_with_index)
            {
                break identifier_with_index;
            }

            index = Some(index.map_or_else(
                || {
                    if identifier == normalized_type_name {
                        1
                    } else {
                        2
                    }
                },
                |index| index + 1,
            ));
        };

        game.typedefs.push(Typedef {
            identifier: identifier.clone(),
            type_: Rc::new(self_normalized),
        });

        Ok(Self::TypeReference { identifier })
    }
}

impl Typedef<String> {
    pub fn normalize_type(&self, game: &mut Game<String>) -> Result<Self, Error<String>> {
        match &*self.type_ {
            Type::Arrow { lhs, rhs } => Ok(Self {
                identifier: self.identifier.clone(),
                type_: Rc::new(Type::Arrow {
                    lhs: lhs.clone(),
                    rhs: Rc::new(rhs.normalize(game)?),
                }),
            }),
            _ => Ok(self.clone()),
        }
    }
}

impl Variable<String> {
    pub fn normalize_type(&self, game: &mut Game<String>) -> Result<Self, Error<String>> {
        Ok(Self {
            default_value: self.default_value.clone(),
            identifier: self.identifier.clone(),
            type_: Rc::new(self.type_.normalize(game)?),
        })
    }
}
