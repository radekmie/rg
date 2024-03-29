use std::fmt::{Display, Formatter, Result};

use super::{
    Binop, DomainDeclaration, DomainElement, DomainValue, Expression, Function, FunctionArg,
    FunctionDeclaration, GameDeclaration, Identifier, Pattern, Statement, Type, TypeDeclaration,
    VariableDeclaration,
};

fn join<T: Display>(f: &mut Formatter<'_>, items: &Vec<T>, separator: &str) -> Result {
    let mut iter = items.iter();
    if let Some(item) = iter.next() {
        write!(f, "{}", item)?;
        for item in iter {
            write!(f, "{}{}", separator, item)?;
        }
    }
    Ok(())
}

fn write_statements<Id: Display>(f: &mut Formatter<'_>, statements: &Vec<Statement<Id>>) -> Result {
    for statement in statements {
        write!(f, "{}\n", statement)?;
    }
    Ok(())
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.identifier)
    }
}

impl<Id: Display> Display for Statement<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Assignment {
                identifier,
                accessors,
                expression,
            } => {
                write!(f, "{}", identifier)?;
                for accessor in accessors {
                    write!(f, "[{}]", accessor)?;
                }
                write!(f, " = {}", expression)
            }
            Self::Branch { arms } => {
                write!(f, "branch {{\n")?;
                let mut iter = arms.iter();
                if let Some(fst_arm) = iter.next() {
                    write_statements(f, fst_arm)?;
                    for arm in iter {
                        write!(f, "}} or {{\n")?;
                        write_statements(f, arm)?;
                    }
                }
                write!(f, "}}")
            }
            Self::Call { identifier, args } => {
                write!(f, "{}(", identifier)?;
                join(f, args, ", ")?;
                write!(f, ")")
            }
            Self::Forall {
                identifier,
                type_,
                body,
            } => {
                write!(f, "forall {}:{} {{\n", identifier, type_)?;
                write_statements(f, body)?;
                write!(f, "}}")
            }
            Self::Loop { body } => {
                write!(f, "loop {{\n")?;
                write_statements(f, body)?;
                write!(f, "}}")
            }
            Self::Pragma { identifier } => write!(f, "@{}", identifier),

            Self::Tag { symbol } => write!(f, "$ {}", symbol),

            Self::When { condition, body } => {
                write!(f, "when {} {{\n", condition)?;
                write_statements(f, body)?;
                write!(f, "}}")
            }
            Self::While { condition, body } => {
                write!(f, "while {} {{\n", condition)?;
                write_statements(f, body)?;
                write!(f, "}}")
            }
        }
    }
}

impl<Id: Display> Display for DomainElement<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Generator {
                identifier,
                args,
                values,
            } => {
                write!(f, "{}(", identifier)?;
                join(f, args, ", ")?;
                write!(f, ")")?;
                if !values.is_empty() {
                    write!(f, " where ")?;
                    join(f, values, ", ")?;
                }
                Ok(())
            }
            Self::Literal { identifier } => write!(f, "{}", identifier),
        }
    }
}

impl<Id: Display> Display for DomainValue<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Range {
                identifier,
                min,
                max,
            } => write!(f, "{} in {}..{}", identifier, min, max),
            Self::Set { identifier, values } => {
                write!(f, "{} in {{ ", identifier)?;
                join(f, values, ", ")?;
                write!(f, " }}")
            }
        }
    }
}

impl Display for Binop {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Gt => write!(f, ">"),
            Self::Lte => write!(f, "<="),
            Self::Gte => write!(f, ">="),
        }
    }
}

impl<Id: Display> Display for Expression<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Access { lhs, rhs } => write!(f, "{}[{}]", lhs, rhs),
            Self::BinExpr {
                lhs,
                op: Binop::And,
                rhs,
            } => {
                write_expr_parens(f, lhs)?;
                write!(f, " && ")?;
                write_expr_parens(f, rhs)
            }
            Self::BinExpr {
                lhs,
                op: Binop::Or,
                rhs,
            } => {
                write_expr_parens(f, lhs)?;
                write!(f, " || ")?;
                write_expr_parens(f, rhs)
            }
            Self::BinExpr { lhs, op, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
            Self::Call { expression, args } => {
                write!(f, "{}(", expression)?;
                join(f, args, ", ")?;
                write!(f, ")")
            }
            Self::Constructor { identifier, args } => {
                write!(f, "{}(", identifier)?;
                join(f, args, ", ")?;
                write!(f, ")")
            }
            Self::Literal { identifier } => write!(f, "{}", identifier),
            Self::Map {
                pattern,
                expression,
                domains,
            } => {
                write!(f, "{{ {} = {}", pattern, expression)?;
                if !domains.is_empty() {
                    write!(f, " where ")?;
                    join(f, domains, ", ")?;
                }
                write!(f, " }}")
            }
            Self::If {
                condition,
                then,
                else_,
            } => write!(f, "if {}\nthen {}\nelse {}", condition, then, else_),
        }
    }
}

fn write_expr_parens<Id: Display>(f: &mut Formatter<'_>, expr: &Expression<Id>) -> Result {
    match expr {
        Expression::BinExpr { op: Binop::Or, .. } => write!(f, "({})", expr),
        _ => write!(f, "{}", expr),
    }
}

impl<Id: Display> Display for Pattern<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Constructor { identifier, args } => {
                write!(f, "{}(", identifier)?;
                join(f, args, ", ")?;
                write!(f, ")")
            }
            Self::Literal { pattern } => write!(f, "{}", pattern),
            Self::Variable { identifier } => write!(f, "{}", identifier),
            Self::Wildcard => write!(f, "_"),
        }
    }
}

impl<Id: Display> Display for Type<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Function { lhs, rhs } => write!(f, "{} -> {}", lhs, rhs),
            Self::Name { identifier } => write!(f, "{}", identifier),
        }
    }
}

impl<Id: Display> Display for Function<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "graph {}(", self.name)?;
        join(f, &self.args, ", ")?;
        write!(f, ") {{\n")?;
        write_statements(f, &self.body)?;
        write!(f, "}}")
    }
}

impl<Id: Display> Display for FunctionArg<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {}", self.identifier, self.type_)
    }
}

impl<Id: Display> Display for TypeDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} : {}", self.identifier, self.type_)
    }
}

impl<Id: Display> Display for DomainDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "domain {} = ", self.identifier)?;
        join(f, &self.elements, " | ")?;
        write!(f, "\n")
    }
}

impl<Id: Display> Display for FunctionDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} : {}\n", self.identifier, self.type_)?;
        for case in &self.cases {
            write!(f, "{}(", self.identifier)?;
            join(f, &case.args, ", ")?;
            write!(f, ") = {}\n", case.body)?;
        }
        Ok(())
    }
}

impl<Id: Display> Display for VariableDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} : {}\n", self.identifier, self.type_)?;
        if let Some(default_value) = self.default_value.as_ref() {
            write!(f, "{} = {}\n", self.identifier, default_value)?;
        }
        Ok(())
    }
}

impl<Id: Display> Display for GameDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.domains
            .iter()
            .try_for_each(|domain| write!(f, "{}\n", domain))?;
        self.functions
            .iter()
            .try_for_each(|domain| write!(f, "{}\n", domain))?;
        self.variables
            .iter()
            .try_for_each(|domain| write!(f, "{}\n", domain))?;
        self.automaton
            .iter()
            .try_for_each(|domain| write!(f, "{}\n\n", domain))?;
        self.types
            .iter()
            .try_for_each(|domain| write!(f, "{}\n\n", domain))?;
        Ok(())
    }
}
