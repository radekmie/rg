use crate::ast::{
    ConstantDeclaration, Error, GameDeclaration, Type, TypeDeclaration, VariableDeclaration,
};
use std::rc::Rc;

impl ConstantDeclaration<String> {
    pub fn normalize_type(
        &self,
        game_declaration: &mut GameDeclaration<String>,
    ) -> Result<Self, Error<String>> {
        Ok(Self {
            identifier: self.identifier.clone(),
            type_: Rc::new(self.type_.normalize(game_declaration)?),
            value: self.value.clone(),
        })
    }
}

impl GameDeclaration<String> {
    pub fn normalize_types(mut self) -> Result<Self, Error<String>> {
        for (index, type_declaration) in self.types.clone().into_iter().enumerate() {
            self.types[index] = Rc::new(type_declaration.normalize_type(&mut self)?);
        }

        for (index, constant_declaration) in self.constants.clone().into_iter().enumerate() {
            self.constants[index] = Rc::new(constant_declaration.normalize_type(&mut self)?);
        }

        for (index, variable_declaration) in self.variables.clone().into_iter().enumerate() {
            self.variables[index] = Rc::new(variable_declaration.normalize_type(&mut self)?);
        }

        Ok(self)
    }
}

impl Type<String> {
    pub fn normalize(
        &self,
        game_declaration: &mut GameDeclaration<String>,
    ) -> Result<Self, Error<String>> {
        if matches!(self, Self::TypeReference { .. }) {
            return Ok(self.clone());
        }

        let self_normalized = match self {
            Self::Arrow { lhs, rhs } => Self::Arrow {
                lhs: lhs.clone(),
                rhs: Rc::new(rhs.normalize(game_declaration)?),
            },
            _ => self.clone(),
        };

        if let Some(type_declaration) =
            game_declaration.resolve_type_declaration(&self_normalized)?
        {
            return Ok(Self::TypeReference {
                identifier: type_declaration.identifier.clone(),
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

            if !game_declaration
                .types
                .iter()
                .any(|type_declaration| type_declaration.identifier == identifier_with_index)
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

        game_declaration.types.push(Rc::new(TypeDeclaration {
            identifier: identifier.clone(),
            type_: Rc::new(self_normalized),
        }));

        Ok(Self::TypeReference { identifier })
    }
}

impl TypeDeclaration<String> {
    pub fn normalize_type(
        &self,
        game_declaration: &mut GameDeclaration<String>,
    ) -> Result<Self, Error<String>> {
        match &*self.type_ {
            Type::Arrow { lhs, rhs } => Ok(Self {
                identifier: self.identifier.clone(),
                type_: Rc::new(Type::Arrow {
                    lhs: lhs.clone(),
                    rhs: Rc::new(rhs.normalize(game_declaration)?),
                }),
            }),
            _ => Ok(self.clone()),
        }
    }
}

impl VariableDeclaration<String> {
    pub fn normalize_type(
        &self,
        game_declaration: &mut GameDeclaration<String>,
    ) -> Result<Self, Error<String>> {
        Ok(Self {
            default_value: self.default_value.clone(),
            identifier: self.identifier.clone(),
            type_: Rc::new(self.type_.normalize(game_declaration)?),
        })
    }
}
