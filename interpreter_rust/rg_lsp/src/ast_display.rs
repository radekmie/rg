use crate::position::Span;
use crate::{
    ast::{
        Constant, Edge, EdgeLabel, EdgeName, EdgeNamePart, Expression, Game, Identifier, Pragma,
        Type, Typedef, Value, ValueEntry, Variable,
    },
    position::Position,
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
            span,
            identifier,
            type_,
            value,
        } = self;
        write!(f, "const {identifier}: {type_} = {value};")
    }
}

impl Display for Edge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            span,
            label,
            lhs,
            rhs,
        } = self;
        write!(f, "{lhs}, {rhs}: {label};")
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { span, identifier } = self;
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
                span,
                lhs,
                rhs,
                negated: false,
            } => write!(f, "? {lhs} -> {rhs}"),
            Self::Reachability {
                span,
                lhs,
                rhs,
                negated: true,
            } => write!(f, "! {lhs} -> {rhs}"),
            Self::Skip { span } => write!(f, ""),
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
                span,
                identifier,
                type_,
            } => write!(f, "({identifier}: {type_})"),
            Self::Literal { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Access { span, lhs, rhs } => write!(f, "{lhs}[{rhs}]"),
            Self::Cast { span, lhs, rhs } => write!(f, "{lhs}({rhs})"),
            Self::Reference { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl Display for Pragma {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Any { span, edge_name } => write!(f, "@any {edge_name};"),
            Self::Disjoint { span, edge_name } => write!(f, "@disjoint {edge_name};"),
            Self::MultiAny { span, edge_name } => write!(f, "@multiAny {edge_name};"),
            Self::Unique { span, edge_name } => write!(f, "@unique {edge_name};"),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Arrow { lhs, rhs } => write!(f, "{lhs} -> {rhs}"),
            Self::Set { span, identifiers } => {
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
            span,
            identifier,
            type_,
        } = self;
        write!(f, "type {identifier} = {type_};")
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Element { identifier } => write!(f, "{identifier}"),
            Self::Map { span, entries } => {
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
            span,
            identifier,
            value,
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
            span,
            default_value,
            identifier,
            type_,
        } = self;
        write!(f, "var {identifier}: {type_} = {default_value};",)
    }
}

impl Display for Game {
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

// impl Display for Game {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//         let Self {
//             constants,
//             edges,
//             typedefs,
//             variables,
//             pragmas
//         } = self;

//         let mut stats: Vec<Box<dyn PositionedDisplay>> = Vec::new();
//         stats.extend(constants.iter().map(|x| Box::new(x.clone()) as Box<dyn PositionedDisplay>));
//         stats.extend(edges.iter().map(|x| Box::new(x.clone()) as Box<dyn PositionedDisplay>));
//         stats.extend(typedefs.iter().map(|x| Box::new(x.clone()) as Box<dyn PositionedDisplay>));
//         stats.extend(variables.iter().map(|x| Box::new(x.clone()) as Box<dyn PositionedDisplay>));
//         stats.extend(pragmas.iter().map(|x| Box::new(x.clone()) as Box<dyn PositionedDisplay>));
//         stats.sort_by(|a, b| a.span().cmp(&b.span()));

//         for stat in stats {
//             write!(f, "{stat}")?;
//         }
//         Ok(())
//     }
// }
