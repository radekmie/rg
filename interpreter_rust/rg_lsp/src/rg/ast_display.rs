use crate::rg::{
    ast::*,
    position::*,
};
use std::fmt::{Display, Formatter, Result};

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { start, end } = self;
        write!(f, "({},{})", start, end)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { line, column } = self;
        write!(f, "{}:{}", line, column)
    }
}

impl Display for Constant {
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

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            label, lhs, rhs, ..
        } = self;
        write!(f, "{lhs}, {rhs}: {label};")
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { identifier, .. } = self;
        // let span = self.span();
        // write!(f, "{span}{identifier}")
        write!(f, "{identifier}")
    }
}

impl Display for EdgeLabel {
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

impl Display for EdgeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for part in &self.parts {
            write!(f, "{part}")?;
        }

        Ok(())
    }
}

impl Display for EdgeNamePart {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Binding {
                identifier,
                type_,
                ..
            } => write!(f, "({identifier}: {type_})"),
            Self::Literal { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Access { lhs, rhs, .. } => write!(f, "{lhs}[{rhs}]"),
            Self::Cast { lhs, rhs, .. } => write!(f, "{lhs}({rhs})"),
            Self::Reference { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl Display for Pragma {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let edge_name = &self.edge_name;
        match self.kind {
            PragmaKind::Any => write!(f, "@any {edge_name};"),
            PragmaKind::Disjoint => write!(f, "@disjoint {edge_name};"),
            PragmaKind::MultiAny => write!(f, "@multiAny {edge_name};"),
            PragmaKind::Unique => write!(f, "@unique {edge_name};"),
        }
    }
}

impl Display for Type {
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

impl Display for Typedef {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            identifier, type_, ..
        } = self;
        write!(f, "type {identifier} = {type_};")
    }
}

impl Display for Value {
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

impl Display for ValueEntry {
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

impl Display for Variable {
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

impl Display for Stat {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Constant(constant) => write!(f, "{constant}"),
            Self::Edge(edge) => write!(f, "{edge}"),
            Self::Typedef(typedef) => write!(f, "{typedef}"),
            Self::Variable(variable) => write!(f, "{variable}"),
            Self::Pragma(pragma) => write!(f, "{pragma}"),
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for stat in &self.stats {
            writeln!(f, "{stat}")?;
        }
        Ok(())
    }
}