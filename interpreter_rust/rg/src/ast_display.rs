use crate::ast::{
    ConstantDeclaration, EdgeDeclaration, EdgeLabel, EdgeName, EdgeNamePart, Error, ErrorReason,
    Expression, GameDeclaration, Pragma, Type, TypeDeclaration, Value, ValueEntry,
    VariableDeclaration,
};
use std::fmt::{Display, Formatter, Result};

impl<Id: Display> Display for ConstantDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            identifier,
            type_,
            value,
        } = self;
        write!(f, "const {identifier}: {type_} = {value};")
    }
}

impl<Id: Display> Display for EdgeDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { label, lhs, rhs } = self;
        write!(f, "{lhs}, {rhs}: {label};")
    }
}

impl<Id: Display> Display for EdgeLabel<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Assignment { lhs, rhs } => write!(f, "{lhs} = {rhs}"),
            Self::Comparison {
                lhs,
                rhs,
                negated: false,
            } => write!(f, "{lhs} == {rhs}"),
            Self::Comparison {
                lhs,
                rhs,
                negated: true,
            } => write!(f, "{lhs} != {rhs}"),
            Self::Reachability {
                lhs,
                rhs,
                negated: false,
            } => write!(f, "? {lhs} -> {rhs}"),
            Self::Reachability {
                lhs,
                rhs,
                negated: true,
            } => write!(f, "! {lhs} -> {rhs}"),
            Self::Skip => write!(f, ""),
        }
    }
}

impl<Id: Display> Display for EdgeName<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for part in &self.parts {
            write!(f, "{part}")?;
        }

        Ok(())
    }
}

impl<Id: Display> Display for EdgeNamePart<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Binding { identifier, type_ } => write!(f, "({identifier}: {type_})"),
            Self::Literal { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl<Id: Display> Display for Error<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            game_declaration,
            reason,
        } = self;

        writeln!(f, "{reason}")?;

        if !game_declaration.types.is_empty() {
            writeln!(f, "  Type definitions:")?;
            for type_declaration in &game_declaration.types {
                let TypeDeclaration { identifier, type_ } = type_declaration;
                writeln!(f, "    {identifier}: {type_}")?;
            }
        }

        if !game_declaration.constants.is_empty() {
            writeln!(f, "  Constant definitions:")?;
            for constant_declaration in &game_declaration.constants {
                let ConstantDeclaration {
                    identifier, type_, ..
                } = constant_declaration;
                writeln!(f, "    {identifier}: {type_}")?;
            }
        }

        if !game_declaration.variables.is_empty() {
            writeln!(f, "  Variable definitions:")?;
            for variable_declaration in &game_declaration.variables {
                let VariableDeclaration {
                    identifier, type_, ..
                } = variable_declaration;
                writeln!(f, "    {identifier}: {type_}")?;
            }
        }

        // TODO: Handle operation scopes.

        Ok(())
    }
}

impl<Id: Display> From<Error<Id>> for String {
    fn from(error: Error<Id>) -> Self {
        format!("{}", error)
    }
}

impl<Id: Display> Display for ErrorReason<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::ArrowTypeExpected { got } => write!(f, "Expected Arrow type, got {got}."),
            Self::AssignmentTypeMismatch { lhs, rhs } => {
                write!(f, "{rhs} is not assignable to {lhs}")
            }
            Self::ComparisonTypeMismatch { lhs, rhs } => {
                write!(f, "{lhs} is not comparable to {rhs}")
            }
            Self::EmptySetType { identifier } => {
                write!(f, "Type {identifier} should not be empty.")
            }
            Self::SetTypeExpected { got } => write!(f, "Expected Set type, got {got}."),
            Self::TypeDeclarationMismatch {
                expected,
                identifier,
                resolved,
            } => {
                writeln!(f, "Type {identifier} is incorrect.")?;
                writeln!(f, "  Expected: {expected}")?;
                write!(f, "  Resolved: {resolved}")
            }
            Self::Unreachable { lhs, rhs } => write!(f, "{rhs} is not reachable from {lhs}."),
            Self::UnresolvedConstant { identifier } => {
                write!(f, "Unresolved constant {identifier}.")
            }
            Self::UnresolvedType { identifier } => write!(f, "Unresolved type {identifier}."),
            Self::UnresolvedVariable { identifier } => {
                write!(f, "Unresolved variable {identifier}.")
            }
            Self::VariableDeclarationMismatch {
                expected,
                identifier,
                resolved,
            } => {
                writeln!(f, "Variable {identifier} has incorrect type.")?;
                writeln!(f, "  Expected: {expected}")?;
                write!(f, "  Resolved: {resolved}")
            }
        }
    }
}

impl<Id: Display> Display for Expression<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Access { lhs, rhs } => write!(f, "{lhs}[{rhs}]"),
            Self::Cast { lhs, rhs } => write!(f, "{lhs}({rhs})"),
            Self::Reference { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl<Id: Display> Display for GameDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for pragma in &self.pragmas {
            writeln!(f, "{pragma}")?;
        }
        for type_ in &self.types {
            writeln!(f, "{type_}")?;
        }
        for constant in &self.constants {
            writeln!(f, "{constant}")?;
        }
        for variable in &self.variables {
            writeln!(f, "{variable}")?;
        }
        for edge in &self.edges {
            writeln!(f, "{edge}")?;
        }
        Ok(())
    }
}

impl<Id: Display> Display for Pragma<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Disjoint { edge_name } => write!(f, "@disjoint {edge_name};"),
        }
    }
}

impl<Id: Display> Display for Type<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Arrow { lhs, rhs } => write!(f, "{lhs} -> {rhs}"),
            Self::Set { identifiers } => {
                write!(f, "{{ ")?;
                for (index, entry) in identifiers.iter().enumerate() {
                    let separator = if index == 0 { "" } else { ", " };
                    write!(f, "{separator}{entry}")?;
                }
                write!(f, " }}")
            }
            Self::TypeReference { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl<Id: Display> Display for TypeDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { identifier, type_ } = self;
        write!(f, "type {identifier} = {type_};")
    }
}

impl<Id: Display> Display for Value<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Element { identifier } => write!(f, "{identifier}"),
            Self::Map { entries } => {
                write!(f, "{{ ")?;
                for (index, entry) in entries.iter().enumerate() {
                    let separator = if index == 0 { "" } else { ", " };
                    write!(f, "{separator}{entry}")?;
                }
                write!(f, " }}")
            }
        }
    }
}

impl<Id: Display> Display for ValueEntry<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::DefaultEntry { value } => write!(f, ":{value}"),
            Self::NamedEntry { identifier, value } => write!(f, "{identifier}: {value}"),
        }
    }
}

impl<Id: Display> Display for VariableDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            default_value,
            identifier,
            type_,
        } = self;
        write!(f, "var {identifier}: {type_} = {default_value};",)
    }
}
