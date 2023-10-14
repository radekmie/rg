use crate::position::{Positioned, Span};
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

impl<'a> Display for Constant<'a> {
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

impl<'a> Display for Edge<'a> {
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

impl<'a> Display for Identifier<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { span, identifier } = self;
        write!(f, "{span}{identifier}")
    }
}

impl<'a> Display for EdgeLabel<'a> {
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

impl<'a> Display for EdgeName<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for part in &self.parts {
            write!(f, "{part}")?;
        }

        Ok(())
    }
}

impl<'a> Display for EdgeNamePart<'a> {
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

impl<'a> Display for Expression<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Access { span, lhs, rhs } => write!(f, "{lhs}[{rhs}]"),
            Self::Cast { span, lhs, rhs } => write!(f, "{lhs}({rhs})"),
            Self::Reference { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl<'a> Display for Pragma<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Any { span, edge_name } => write!(f, "@any {edge_name};"),
            Self::Disjoint { span, edge_name } => write!(f, "@disjoint {edge_name};"),
            Self::MultiAny { span, edge_name } => write!(f, "@multiAny {edge_name};"),
            Self::Unique { span, edge_name } => write!(f, "@unique {edge_name};"),
        }
    }
}

impl<'a> Display for Type<'a> {
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

impl<'a> Display for Typedef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            span,
            identifier,
            type_,
        } = self;
        write!(f, "type {identifier} = {type_};")
    }
}

impl<'a> Display for Value<'a> {
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

impl<'a> Display for ValueEntry<'a> {
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

impl<'a> Display for Variable<'a> {
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

impl<'a> Display for Game<'a> {
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

// impl<'a> Display for Game<'a> {
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
