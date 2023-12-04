use crate::ast::*;
use crate::position::{Position, Span};
use std::fmt::{Display, Formatter, Result};

impl<Id: Display> Display for Constant<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            identifier,
            type_,
            value,
            ..
        } = self;
        write!(f, "const {identifier}: {type_} = {value};")
    }
}

impl<Id: Display> Display for Edge<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            label, lhs, rhs, ..
        } = self;
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
                ..
            } => write!(f, "? {lhs} -> {rhs}"),
            Self::Reachability {
                lhs,
                rhs,
                negated: true,
                ..
            } => write!(f, "! {lhs} -> {rhs}"),
            Self::Skip { .. } => write!(f, ""),
            Self::Tag { symbol } => write!(f, "$ {symbol}"),
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
            Self::Binding {
                identifier, type_, ..
            } => write!(f, "({identifier}: {type_})"),
            Self::Literal { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl<Id: Display> Display for Error<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { game, reason } = self;

        writeln!(f, "{reason}")?;

        if !game.typedefs.is_empty() {
            writeln!(f, "  Type definitions:")?;
            for typedef in &game.typedefs {
                let Typedef {
                    identifier, type_, ..
                } = typedef;
                writeln!(f, "    {identifier}: {type_}")?;
            }
        }

        if !game.constants.is_empty() {
            writeln!(f, "  Constant definitions:")?;
            for constant in &game.constants {
                let Constant {
                    identifier, type_, ..
                } = constant;
                writeln!(f, "    {identifier}: {type_}")?;
            }
        }

        if !game.variables.is_empty() {
            writeln!(f, "  Variable definitions:")?;
            for variable in &game.variables {
                let Variable {
                    identifier, type_, ..
                } = variable;
                writeln!(f, "    {identifier}: {type_}")?;
            }
        }

        // TODO: Handle operation scopes.

        Ok(())
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
            Self::DuplicatedMapKey { key, value } => match key {
                Some(key) => write!(f, "Duplicated key {key} in map {value}."),
                None => write!(f, "Duplicated default value in map {value}."),
            },
            Self::EmptySetType { identifier } => {
                write!(f, "Type {identifier} should not be empty.")
            }
            Self::MissingDefaultValue { value } => write!(f, "Missing default value in {value}."),
            Self::MultipleEdges { lhs, rhs } => write!(
                f,
                "Multiple edges between two nodes are not allowed ({lhs} -> {rhs})."
            ),
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
            Self::Access { lhs, rhs, .. } => write!(f, "{lhs}[{rhs}]"),
            Self::Cast { lhs, rhs, .. } => write!(f, "{lhs}({rhs})"),
            Self::Reference { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl<Id: Display> Display for Game<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for pragma in &self.pragmas {
            writeln!(f, "{pragma}")?;
        }
        for typedef in &self.typedefs {
            writeln!(f, "{typedef}")?;
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

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { identifier, .. } = self;
        write!(f, "{identifier}")
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { line, column } = self;
        write!(f, "{line}:{column}")
    }
}

impl<Id: Display> Display for Pragma<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Any { edge_name, .. } => write!(f, "@any {edge_name};"),
            Self::Disjoint { edge_name, .. } => write!(f, "@disjoint {edge_name};"),
            Self::MultiAny { edge_name, .. } => write!(f, "@multiAny {edge_name};"),
            Self::Unique { edge_name, .. } => write!(f, "@unique {edge_name};"),
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { start, end } = self;
        if start == end {
            write!(f, "({start})")
        } else {
            write!(f, "({start}, {end})")
        }
    }
}

impl<Id: Display> Display for Type<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Arrow { lhs, rhs } => write!(f, "{lhs} -> {rhs}"),
            Self::Set { identifiers, .. } => {
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

impl<Id: Display> Display for Typedef<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            identifier, type_, ..
        } = self;
        write!(f, "type {identifier} = {type_};")
    }
}

impl<Id: Display> Display for Value<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Element { identifier } => write!(f, "{identifier}"),
            Self::Map { entries, .. } => {
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
        let Self {
            identifier, value, ..
        } = self;
        match identifier {
            Some(identifier) => write!(f, "{identifier}: {value}"),
            None => write!(f, ":{value}"),
        }
    }
}

impl<Id: Display> Display for Variable<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            default_value,
            identifier,
            type_,
            ..
        } = self;
        write!(f, "var {identifier}: {type_} = {default_value};",)
    }
}
