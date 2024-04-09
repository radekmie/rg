use super::{
    Binop, DomainDeclaration, DomainElement, DomainValue, Expression, Function, FunctionArg,
    FunctionDeclaration, GameDeclaration, Pattern, Statement, Type, TypeDeclaration,
    VariableDeclaration,
};
use std::fmt::{Display, Formatter, Result};

fn write_with_separator<T: Display>(f: &mut Formatter<'_>, items: &[T], separator: &str) -> Result {
    let mut iter = items.iter();
    if let Some(item) = iter.next() {
        write!(f, "{item}")?;
        for item in iter {
            write!(f, "{separator}{item}")?;
        }
    }
    Ok(())
}

impl<Id: Display> Display for Statement<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_statement(f, self, 0)
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
                write_with_separator(f, args, ", ")?;
                write!(f, ")")?;
                if !values.is_empty() {
                    write!(f, " where ")?;
                    write_with_separator(f, values, ", ")?;
                }
                Ok(())
            }
            Self::Literal { identifier } => write!(f, "{identifier}"),
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
            } => write!(f, "{identifier} in {min}..{max}"),
            Self::Set { identifier, values } => {
                write!(f, "{identifier} in {{ ")?;
                write_with_separator(f, values, ", ")?;
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
        write_expression(f, self, 0)
    }
}

impl<Id: Display> Display for Pattern<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Constructor { identifier, args } => {
                write!(f, "{identifier}(")?;
                write_with_separator(f, args, ", ")?;
                write!(f, ")")
            }
            Self::Literal { pattern } => write!(f, "{pattern}"),
            Self::Variable { identifier } => write!(f, "{identifier}"),
            Self::Wildcard => write!(f, "_"),
        }
    }
}

impl<Id: Display> Display for Type<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Function { lhs, rhs } => write!(f, "{lhs} -> {rhs}"),
            Self::Name { identifier } => write!(f, "{identifier}"),
        }
    }
}

impl<Id: Display> Display for Function<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_function(f, self, 0)
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
        write_with_separator(f, &self.elements, " | ")
    }
}

impl<Id: Display> Display for FunctionDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_function_declaration(f, self, 0)
    }
}

impl<Id: Display> Display for VariableDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "{} : {}", self.identifier, self.type_)?;
        if let Some(default_value) = self.default_value.as_ref() {
            write!(f, "{} = ", self.identifier)?;
            write_expression(f, default_value.as_ref(), 2)?;
        }
        Ok(())
    }
}

impl<Id: Display> Display for GameDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.domains
            .iter()
            .try_for_each(|domain| writeln!(f, "{domain}"))?;
        self.functions
            .iter()
            .try_for_each(|function| writeln!(f, "{function}"))?;
        self.variables
            .iter()
            .try_for_each(|variable| writeln!(f, "{variable}"))?;
        self.automaton
            .iter()
            .try_for_each(|function| write!(f, "{function}\n\n"))?;
        self.types
            .iter()
            .try_for_each(|type_| write!(f, "{type_}\n\n"))?;
        Ok(())
    }
}

fn write_expression<Id: Display>(
    f: &mut Formatter<'_>,
    expression: &Expression<Id>,
    indent: usize,
) -> Result {
    match expression {
        Expression::Access { lhs, rhs } => write!(f, "{lhs}[{rhs}]"),
        Expression::BinExpr {
            lhs,
            op: Binop::And,
            rhs,
        } => {
            write_expr_parens(f, lhs, indent)?;
            write!(f, " && ")?;
            write_expr_parens(f, rhs, indent)
        }
        Expression::BinExpr {
            lhs,
            op: Binop::Or,
            rhs,
        } => {
            write_expr_parens(f, lhs, indent)?;
            write!(f, " || ")?;
            write_expr_parens(f, rhs, indent)
        }
        Expression::BinExpr { lhs, op, rhs } => write!(f, "{lhs} {op} {rhs}"),
        Expression::Call { expression, args } => {
            write!(f, "{expression}(")?;
            write_with_separator(f, args, ", ")?;
            write!(f, ")")
        }
        Expression::Constructor { identifier, args } => {
            write!(f, "{identifier}(")?;
            write_with_separator(f, args, ", ")?;
            write!(f, ")")
        }
        Expression::Literal { identifier } => write!(f, "{identifier}"),
        Expression::Map {
            pattern,
            expression,
            domains,
        } => {
            writeln!(f, "{{")?;
            write_indent(f, indent)?;
            write!(f, "{} = ", pattern)?;
            write_expression(f, expression, indent + 2)?;
            if !domains.is_empty() {
                write!(f, " where ")?;
                write_with_separator(f, domains, ", ")?;
            }
            writeln!(f)?;
            write_rbrace(f, indent)
        }
        Expression::If {
            condition,
            then,
            else_,
        } => {
            write!(f, "if ")?;
            write_expression(f, condition.as_ref(), indent + 2)?;
            writeln!(f)?;
            write_indent(f, indent)?;
            write!(f, "then ")?;
            write_expression(f, then.as_ref(), indent + 2)?;
            writeln!(f)?;
            write_indent(f, indent)?;
            write!(f, "else ")?;
            write_expression(f, else_.as_ref(), indent + 2)
        }
    }
}

fn write_expr_parens<Id: Display>(
    f: &mut Formatter<'_>,
    expr: &Expression<Id>,
    indent: usize,
) -> Result {
    match expr {
        Expression::BinExpr { op: Binop::Or, .. } => {
            write!(f, "(")?;
            write_expression(f, expr, indent)?;
            write!(f, ")")
        }
        _ => write_expression(f, expr, indent),
    }
}

fn write_indent(f: &mut Formatter<'_>, indent: usize) -> Result {
    write!(f, "{}", " ".repeat(indent))
}

fn write_rbrace(f: &mut Formatter<'_>, indent: usize) -> Result {
    write_indent(f, indent)?;
    write!(f, "}}")
}

fn write_statement<Id: Display>(
    f: &mut Formatter<'_>,
    statement: &Statement<Id>,
    indent: usize,
) -> Result {
    write_indent(f, indent)?;
    match statement {
        Statement::Assignment {
            identifier,
            accessors,
            expression,
        } => {
            write!(f, "{identifier}")?;
            for accessor in accessors {
                write!(f, "[{accessor}]")?;
            }
            write!(f, " = ")?;
            write_expression(f, expression, indent)
        }
        Statement::Branch { arms } => {
            writeln!(f, "branch {{")?;
            let mut iter = arms.iter();
            if let Some(fst_arm) = iter.next() {
                write_statements(f, fst_arm, indent + 2)?;
                for arm in iter {
                    write_indent(f, indent)?;
                    writeln!(f, "}} or {{")?;
                    write_statements(f, arm, indent + 2)?;
                }
            }
            write_rbrace(f, indent)
        }
        Statement::Call { identifier, args } => {
            write!(f, "{identifier}(")?;
            write_with_separator(f, args, ", ")?;
            write!(f, ")")
        }
        Statement::Forall {
            identifier,
            type_,
            body,
        } => {
            writeln!(f, "forall {identifier}:{type_} {{")?;
            write_statements(f, body, indent + 2)?;
            write_rbrace(f, indent)
        }
        Statement::Loop { body } => {
            writeln!(f, "loop {{")?;
            write_statements(f, body, indent + 2)?;
            write_rbrace(f, indent)
        }
        Statement::Pragma { identifier } => write!(f, "@{identifier}"),

        Statement::Tag { symbol } => write!(f, "$ {symbol}"),

        Statement::When { condition, body } => {
            writeln!(f, "when {condition} {{")?;
            write_statements(f, body, indent + 2)?;
            write_rbrace(f, indent)
        }
        Statement::While { condition, body } => {
            writeln!(f, "while {condition} {{")?;
            write_statements(f, body, indent + 2)?;
            write_rbrace(f, indent)
        }
    }
}

fn write_statements<Id: Display>(
    f: &mut Formatter<'_>,
    statements: &Vec<Statement<Id>>,
    indent: usize,
) -> Result {
    for statement in statements {
        write_statement(f, statement, indent)?;
        writeln!(f)?;
    }
    Ok(())
}

fn write_function<Id: Display>(
    f: &mut Formatter<'_>,
    function: &Function<Id>,
    indent: usize,
) -> Result {
    write_indent(f, indent)?;
    write!(f, "graph {}(", function.name)?;
    write_with_separator(f, &function.args, ", ")?;
    writeln!(f, ") {{")?;
    write_statements(f, &function.body, indent + 2)?;
    write_indent(f, indent)?;
    write!(f, "}}")
}

fn write_function_declaration<Id: Display>(
    f: &mut Formatter<'_>,
    decl: &FunctionDeclaration<Id>,
    indent: usize,
) -> Result {
    write_indent(f, indent)?;
    writeln!(f, "{} : {}", decl.identifier, decl.type_)?;
    for case in &decl.cases {
        write_indent(f, indent)?;
        write!(f, "{}(", decl.identifier)?;
        write_with_separator(f, &case.args, ", ")?;
        write!(f, ") = ")?;
        write_expression(f, case.body.as_ref(), indent + 2)?;
        writeln!(f)?;
    }
    Ok(())
}
