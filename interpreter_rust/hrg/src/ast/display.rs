use super::{
    Binop, DomainDeclaration, DomainElement, DomainElementPattern, DomainValue, Error, Expression,
    Function, FunctionArg, FunctionDeclaration, Game, Pattern, Statement, Type, TypeDeclaration,
    Value, ValueMapEntry, VariableDeclaration,
};
use std::fmt::{Display, Formatter, Result};
use utils::display::write_with_separator;

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
                write!(f, "{identifier}(")?;
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

impl<Id: Display> Display for DomainElementPattern<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Literal { identifier } => write!(f, "{identifier}"),
            Self::Variable { identifier } => write!(f, "{identifier}"),
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
            Self::Set {
                identifier,
                elements,
            } => {
                write!(f, "{identifier} in {{ ")?;
                write_with_separator(f, elements, ", ")?;
                write!(f, " }}")
            }
        }
    }
}

impl<Id: Display> Display for Error<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::DuplicatedDomainValue { identifier } => {
                write!(f, "Duplicated domain value \"{identifier}\".")
            }
            Self::DuplicatedMapKey { key } => {
                write!(f, "Duplicated map key \"{key}\".")
            }
            Self::EmptyMap => write!(f, "At least one map entry is required to construct a map."),
            Self::FunctionCaseNotCovered { identifier, args } => {
                write!(f, "No case for {identifier}(")?;
                write_with_separator(f, args, ", ")?;
                write!(f, ")")
            }
            Self::IncomparableValues { lhs, rhs } => {
                write!(f, "Values \"{lhs}\" and \"{rhs}\" are not comparable.")
            }
            Self::IncorrectNumberOfArguments {
                identifier,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Function \"{identifier}\" expected {expected} arguments but got {got}."
                )
            }
            Self::InvalidCondition { expression } => {
                write!(f, "Expression \"{expression}\" is not a valid condition.")
            }
            Self::NotImplemented { message } => write!(f, "Not implemented ({message})."),
            Self::UnknownAutomatonFunction { identifier } => {
                write!(f, "Unknown automaton function \"{identifier}\".")
            }
            Self::UnknownFunction { identifier } => {
                write!(f, "Unknown function \"{identifier}\".")
            }
            Self::UnknownType { identifier } => {
                write!(f, "Unknown type \"{identifier}\".")
            }
            Self::UnknownVariable { identifier } => {
                write!(f, "Unknown variable \"{identifier}\".")
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
            Self::Mod => write!(f, "%"),
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
            Self::Literal { identifier } => write!(f, "{identifier}"),
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

impl<Id: Display> Display for Value<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Constructor { identifier, args } => {
                write!(f, "{identifier}(")?;
                write_with_separator(f, args, ", ")?;
                write!(f, ")")
            }
            Self::Element { identifier } => write!(f, "{identifier}"),
            Self::Map { entries, .. } => {
                write!(f, "{{ ")?;
                write_with_separator(f, entries, ", ")?;
                write!(f, " }}")
            }
        }
    }
}

impl<Id: Display> Display for ValueMapEntry<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let Self { key, value } = self;
        write!(f, "{key}: {value}")
    }
}

impl<Id: Display> Display for VariableDeclaration<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let Some(default_value) = self.default_value.as_ref() {
            write!(f, "{} : {} = ", self.identifier, self.type_)?;
            write_expression(f, default_value, 0)?;
            writeln!(f)
        } else {
            writeln!(f, "{} : {}", self.identifier, self.type_)
        }
    }
}

fn write_toplevel<T: Display>(f: &mut Formatter<'_>, items: &[T]) -> Result {
    items.iter().try_for_each(|item| writeln!(f, "{item}"))
}

impl<Id: Display> Display for Game<Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write_toplevel(f, &self.domains)?;
        writeln!(f)?;
        write_toplevel(f, &self.functions)?;
        write_toplevel(f, &self.variables)?;
        write_toplevel(f, &self.automaton)?;
        write_toplevel(f, &self.types)?;
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
        Expression::BinExpr { lhs, op, rhs } => {
            write_expr_parens(f, *op, lhs, indent)?;
            write!(f, " {op} ")?;
            write_expr_parens(f, *op, rhs, indent)
        }
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
            default_value,
            parts,
        } => {
            let indent = if indent == 0 { 0 } else { indent - 2 };
            writeln!(f, "{{")?;

            if let Some(default_value) = default_value {
                write_indent(f, indent + 2)?;
                write!(f, ":")?;
                write_expression(f, default_value, indent + 4)?;

                if !parts.is_empty() {
                    writeln!(f, ";")?;
                }
            }

            for (index, part) in parts.iter().enumerate() {
                write_indent(f, indent + 2)?;
                write!(f, "{} = ", part.pattern)?;
                write_expression(f, &part.expression, indent + 4)?;
                if !part.domains.is_empty() {
                    writeln!(f)?;
                    write_indent(f, indent + 4)?;
                    write!(f, "where ")?;
                    write_with_separator(f, &part.domains, ", ")?;
                }

                if index + 1 != parts.len() {
                    write!(f, ";")?;
                }

                writeln!(f)?;
            }

            write_rbrace(f, indent)
        }
        Expression::If { cond, then, else_ } => {
            write!(f, "if ")?;
            write_expression(f, cond.as_ref(), indent + 2)?;
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
    op_outer: Binop,
    expr: &Expression<Id>,
    indent: usize,
) -> Result {
    match expr {
        Expression::BinExpr { op, .. } if op_outer.precedence() >= op.precedence() => {
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
        Statement::AssignmentAny {
            identifier,
            accessors,
            type_,
        } => {
            write!(f, "{identifier}")?;
            for accessor in accessors {
                write!(f, "[{accessor}]")?;
            }
            write!(f, " = {type_}(*)")
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
        Statement::If {
            expression,
            then,
            else_,
        } => {
            writeln!(f, "if {expression} {{")?;
            write_statements(f, then, indent + 2)?;
            if let Some(else_) = else_ {
                write_indent(f, indent)?;
                writeln!(f, "}} else {{")?;
                write_statements(f, else_, indent + 2)?;
            }
            write_rbrace(f, indent)
        }
        Statement::Loop { body } => {
            writeln!(f, "loop {{")?;
            write_statements(f, body, indent + 2)?;
            write_rbrace(f, indent)
        }
        Statement::Repeat { count, body } => {
            writeln!(f, "repeat {count} {{")?;
            write_statements(f, body, indent + 2)?;
            write_rbrace(f, indent)
        }
        Statement::Tag { symbol } => write!(f, "$ {symbol}"),
        Statement::TagVariable { identifier } => write!(f, "$$ {identifier}"),
        Statement::While { expression, body } => {
            writeln!(f, "while {expression} {{")?;
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
    if function.reusable {
        write!(f, "reusable ")?;
    }
    write!(f, "graph {}(", function.name)?;
    write_with_separator(f, &function.args, ", ")?;
    writeln!(f, ") {{")?;
    write_statements(f, &function.body, indent + 2)?;
    write_indent(f, indent)?;
    writeln!(f, "}}")
}

fn write_function_declaration<Id: Display>(
    f: &mut Formatter<'_>,
    decl: &FunctionDeclaration<Id>,
    indent: usize,
) -> Result {
    write_indent(f, indent)?;
    write!(f, "{} : {}", decl.identifier, decl.type_)?;
    for case in &decl.cases {
        writeln!(f)?;
        write_indent(f, indent)?;
        write!(f, "{}(", decl.identifier)?;
        write_with_separator(f, &case.args, ", ")?;
        write!(f, ") = ")?;
        write_expression(f, case.body.as_ref(), indent + 2)?;
    }
    writeln!(f)?;
    Ok(())
}
