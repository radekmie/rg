use super::{
    Action, ActionOrRule, Atom, ComparisonOperator, Edge, Error, Expression, ExpressionOperator,
    Game, Node, Operator, RValue, Rule, Variable,
};
use std::fmt::{Display, Formatter, Result};
use utils::display::write_with_separator;

macro_rules! should_wrap {
    ($xs:expr) => {
        $xs.iter().map(|x| x.len()).sum::<usize>() >= 50 || $xs.iter().any(|x| x.contains("\n"))
    };
}

fn indent_if_needed(string: String) -> String {
    if string.contains("\n") {
        string.replace("\n", "\n  ")
    } else {
        string
    }
}

impl<Id: Display> Display for Action<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Assignment { variable, rvalue } => write!(f, "[$ {variable} = {rvalue}]"),
            Self::Check { negated, rule } => {
                let sign = if *negated { '!' } else { '?' };
                let rule = rule.to_string();
                if should_wrap!([&rule]) {
                    write!(f, "{{{sign}\n  {}\n}}", indent_if_needed(rule))
                } else {
                    write!(f, "{{{sign} {rule}}}")
                }
            }
            Self::Comparison { lhs, rhs, operator } => {
                write!(f, "{{$ {lhs} {operator} {rhs}}}")
            }
            Self::Off { piece } => write!(f, "[{piece}]"),
            Self::On { pieces } => {
                write!(f, "{{")?;
                write_with_separator(f, pieces, ", ")?;
                write!(f, "}}")
            }
            Self::Shift { label } => write!(f, "{label}"),
            Self::Switch { player } => match player {
                None => write!(f, "->>"),
                Some(player) => write!(f, "->{player}"),
            },
        }
    }
}

impl<Id: Display> Display for ActionOrRule<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Action(action) => write!(f, "{action}"),
            Self::Rule(rule) => {
                let rule = rule.to_string();
                if should_wrap!([&rule]) {
                    write!(f, "(\n  {}\n)", indent_if_needed(rule))
                } else {
                    write!(f, "({rule})")
                }
            }
        }
    }
}

impl<Id: Display> Display for Atom<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { content, power } = self;
        write!(f, "{content}{}", if *power { "*" } else { "" })
    }
}

impl Display for ComparisonOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Lte => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Gte => write!(f, ">="),
        }
    }
}

impl<Id: Display> Display for Edge<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { label, node } = self;
        write!(f, "{label}:{node}")
    }
}

impl<Id: Display> Display for Error<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Todo(identifier) => {
                write!(f, "TODO({identifier})")
            }
        }
    }
}

impl<Id: Display> Display for Expression<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { lhs, rhs, operator } = self;
        write!(f, "({lhs} {operator} {rhs})")
    }
}

impl Display for ExpressionOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
        }
    }
}

impl<Id: Display> Display for Game<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self {
            pieces,
            variables,
            players,
            board,
            rules,
        } = self;
        write!(f, "#board =\n  ")?;
        write_with_separator(f, board, "\n  ")?;
        write!(f, "\n#players = ")?;
        write_with_separator(f, players, ", ")?;
        write!(f, "\n#pieces = ")?;
        write_with_separator(f, pieces, ", ")?;
        write!(f, "\n#variables = ")?;
        write_with_separator(f, variables, ", ")?;
        write!(f, "\n#rules = {rules}")
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Comparison(operator) => write!(f, "{operator}"),
            Self::Expression(operator) => write!(f, "{operator}"),
        }
    }
}

impl<Id: Display> Display for Node<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { node, piece, edges } = self;
        write!(f, "{node}[{piece}]{{")?;
        write_with_separator(f, edges, ",")?;
        write!(f, "}}")
    }
}

impl<Id: Display> Display for Rule<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { elements } = self;
        if let [atoms] = &elements[..] {
            let atoms: Vec<_> = atoms.iter().map(ToString::to_string).collect();
            let separator = if should_wrap!(&atoms) { "\n" } else { " " };
            return write_with_separator(f, &atoms, separator);
        }

        let elements = elements
            .iter()
            .map(|concatenation| {
                concatenation
                    .iter()
                    .map(ToString::to_string)
                    .map(indent_if_needed)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>();

        if should_wrap!(&elements) {
            write!(f, "  ")?;
            write_with_separator(f, &elements, "\n+ ")
        } else {
            write_with_separator(f, &elements, " + ")
        }
    }
}

impl<Id: Display> Display for RValue<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Expression(expression) => write!(f, "{expression}"),
            Self::Number(number) => write!(f, "{number}"),
            Self::String(string) => write!(f, "{string}"),
        }
    }
}

impl<Id: Display> Display for Variable<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { name, bound } = self;
        write!(f, "{name}({bound})")
    }
}
