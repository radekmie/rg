mod display;

use map_id::MapId;
use map_id_macro::MapId;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;

/// `(caller, callee) -> count`.
pub type CallsCount<Id> = BTreeMap<(Id, Id), usize>;

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Statement<Id> {
    Assignment {
        identifier: Id,
        accessors: Vec<Arc<Expression<Id>>>,
        expression: Arc<Expression<Id>>,
    },
    AssignmentAny {
        identifier: Id,
        accessors: Vec<Arc<Expression<Id>>>,
        type_: Arc<Type<Id>>,
    },
    Branch {
        arms: Vec<Vec<Statement<Id>>>,
    },
    BranchVar {
        identifier: Id,
        type_: Arc<Type<Id>>,
        body: Vec<Statement<Id>>,
    },
    Call {
        identifier: Id,
        args: Vec<Arc<Expression<Id>>>,
    },
    If {
        expression: Arc<Expression<Id>>,
        then: Vec<Statement<Id>>,
        else_: Option<Vec<Statement<Id>>>,
    },
    Loop {
        body: Vec<Statement<Id>>,
    },
    Repeat {
        count: usize,
        body: Vec<Statement<Id>>,
    },
    Tag {
        artificial: bool,
        symbol: Id,
    },
    TagVariable {
        identifier: Id,
    },
    While {
        expression: Arc<Expression<Id>>,
        body: Vec<Statement<Id>>,
    },
}

impl<Id: Clone + Ord> Statement<Id> {
    pub fn count_calls(&self, call_counts: &mut CallsCount<Id>, caller: &Id) {
        match self {
            Self::Assignment { .. }
            | Self::AssignmentAny { .. }
            | Self::Tag { .. }
            | Self::TagVariable { .. } => {}
            Self::Branch { arms } => {
                for statement in arms.iter().flatten() {
                    statement.count_calls(call_counts, caller);
                }
            }
            Self::Call { identifier, .. } => {
                *call_counts
                    .entry((caller.clone(), identifier.clone()))
                    .or_default() += 1;
            }
            Self::If {
                expression,
                then,
                else_,
            } => {
                expression.count_calls(call_counts, caller);
                for statement in then.iter().chain(else_.iter().flatten()) {
                    statement.count_calls(call_counts, caller);
                }
            }
            Self::BranchVar { body, .. } | Self::Loop { body } | Self::Repeat { body, .. } => {
                for statement in body {
                    statement.count_calls(call_counts, caller);
                }
            }
            Self::While { expression, body } => {
                expression.count_calls(call_counts, caller);
                for statement in body {
                    statement.count_calls(call_counts, caller);
                }
            }
        }
    }
}

impl<Id: Clone + PartialEq> Statement<Id> {
    pub fn substitute_var(&mut self, var: &Id, value: &Id) -> Result<(), Error<Id>> {
        match self {
            Self::Assignment { identifier, .. } if identifier == var => {
                return Err(Error::CannotSubstitute {
                    identifier: var.clone(),
                    context: "assigned variable",
                })
            }
            Self::Assignment {
                accessors,
                expression,
                ..
            } => {
                for accessor in accessors {
                    Arc::make_mut(accessor).substitute_var(var, value)?;
                }
                Arc::make_mut(expression).substitute_var(var, value)?;
            }
            Self::AssignmentAny { identifier, .. } if identifier == var => {
                return Err(Error::CannotSubstitute {
                    identifier: var.clone(),
                    context: "assigned variable",
                })
            }
            Self::AssignmentAny { accessors, .. } => {
                for accessor in accessors {
                    Arc::make_mut(accessor).substitute_var(var, value)?;
                }
            }
            Self::Branch { arms } => {
                for arm in arms {
                    for statement in arm {
                        statement.substitute_var(var, value)?;
                    }
                }
            }
            Self::BranchVar {
                identifier, body, ..
            } if identifier != var => {
                for statement in body {
                    statement.substitute_var(var, value)?;
                }
            }
            Self::Call { identifier, .. } if identifier == var => {
                return Err(Error::CannotSubstitute {
                    identifier: var.clone(),
                    context: "function name",
                })
            }
            Self::Call { args, .. } => {
                for arg in args {
                    Arc::make_mut(arg).substitute_var(var, value)?;
                }
            }
            Self::If {
                expression,
                then,
                else_,
            } => {
                Arc::make_mut(expression).substitute_var(var, value)?;
                for statement in then {
                    statement.substitute_var(var, value)?;
                }
                if let Some(else_) = else_ {
                    for statement in else_ {
                        statement.substitute_var(var, value)?;
                    }
                }
            }
            Self::Loop { body } => {
                for statement in body {
                    statement.substitute_var(var, value)?;
                }
            }
            Self::Repeat { body, .. } => {
                for statement in body {
                    statement.substitute_var(var, value)?;
                }
            }
            Self::Tag { symbol, .. } if symbol == var => {
                *symbol = value.clone();
            }
            Self::TagVariable { identifier } if identifier == var => {
                return Err(Error::CannotSubstitute {
                    identifier: var.clone(),
                    context: "tag variable",
                })
            }
            Self::While { expression, body } => {
                Arc::make_mut(expression).substitute_var(var, value)?;
                for statement in body {
                    statement.substitute_var(var, value)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Function<Id> {
    pub reusable: bool,
    pub name: Id,
    pub args: Vec<FunctionArg<Id>>,
    pub body: Vec<Statement<Id>>,
}

impl<Id: Clone + Ord> Function<Id> {
    pub fn count_calls(&self, call_counts: &mut CallsCount<Id>) {
        for statement in &self.body {
            statement.count_calls(call_counts, &self.name);
        }
    }
}

impl<Id: PartialEq> Function<Id> {
    pub fn arg_index(&self, identifier: &Id) -> Option<usize> {
        self.args
            .iter()
            .position(|arg| arg.identifier == *identifier)
    }
}

impl Function<Arc<str>> {
    pub fn nth_arg_variable(&self, index: usize) -> Arc<str> {
        Arc::from(format!(
            "{}_arg{index}_{}",
            self.name, self.args[index].identifier
        ))
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FunctionArg<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FunctionDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
    pub cases: Vec<FunctionCase<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FunctionCase<Id> {
    pub identifier: Id,
    pub args: Vec<Arc<Pattern<Id>>>,
    pub body: Arc<Expression<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DomainDeclaration<Id> {
    pub identifier: Id,
    pub elements: Vec<DomainElement<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DomainElement<Id> {
    Generator {
        identifier: Id,
        args: Vec<DomainElementPattern<Id>>,
        values: Vec<DomainValue<Id>>,
    },
    Literal {
        identifier: Id,
    },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DomainElementPattern<Id> {
    Literal { identifier: Id },
    Variable { identifier: Id },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DomainValue<Id> {
    Range {
        identifier: Id,
        min: usize,
        max: usize,
    },
    Set {
        identifier: Id,
        elements: Vec<Id>,
    },
}

impl<Id> DomainValue<Id> {
    pub fn identifier(&self) -> &Id {
        match self {
            Self::Range { identifier, .. } => identifier,
            Self::Set { identifier, .. } => identifier,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error<Id> {
    CannotSubstitute {
        identifier: Id,
        context: &'static str,
    },
    DuplicatedDomainValue {
        identifier: Id,
    },
    DuplicatedMapKey {
        key: Value<Id>,
    },
    EmptyMap,
    FunctionCaseNotCovered {
        identifier: Id,
        args: Vec<Value<Id>>,
    },
    IncomparableValues {
        lhs: Value<Id>,
        rhs: Value<Id>,
    },
    IncorrectNumberOfArguments {
        identifier: Id,
        expected: usize,
        got: usize,
    },
    InvalidCondition {
        expression: Expression<Id>,
    },
    NotImplemented {
        message: &'static str,
    },
    UnknownAutomatonFunction {
        identifier: Id,
    },
    UnknownFunction {
        identifier: Id,
    },
    UnknownType {
        identifier: Id,
    },
    UnknownVariable {
        identifier: Id,
    },
}

// TODO: Implement MapId for trivial enums
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Binop {
    Add,
    And,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    Mod,
    Ne,
    Or,
    Sub,
}

impl Binop {
    pub fn precedence(&self) -> usize {
        match self {
            Self::Or => 0,
            Self::And => 1,
            Self::Eq | Self::Gt | Self::Gte | Self::Lt | Self::Lte | Self::Ne => 2,
            Self::Add | Self::Mod | Self::Sub => 3,
        }
    }
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for Binop {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Expression<Id> {
    Access {
        lhs: Arc<Expression<Id>>,
        rhs: Arc<Expression<Id>>,
    },
    BinExpr {
        lhs: Arc<Expression<Id>>,
        op: Binop,
        rhs: Arc<Expression<Id>>,
    },
    Call {
        expression: Arc<Expression<Id>>,
        args: Vec<Arc<Expression<Id>>>,
    },
    Constructor {
        identifier: Id,
        args: Vec<Arc<Expression<Id>>>,
    },
    If {
        cond: Arc<Expression<Id>>,
        then: Arc<Expression<Id>>,
        else_: Arc<Expression<Id>>,
    },
    Literal {
        identifier: Id,
    },
    Map {
        default_value: Option<Arc<Expression<Id>>>,
        parts: Vec<ExpressionMapPart<Id>>,
    },
}

impl<Id: Clone + Ord> Expression<Id> {
    pub fn count_calls(&self, call_counts: &mut CallsCount<Id>, caller: &Id) {
        match self {
            Self::Access { lhs, rhs } | Self::BinExpr { lhs, rhs, .. } => {
                lhs.count_calls(call_counts, caller);
                rhs.count_calls(call_counts, caller);
            }
            Self::Call { expression, args } => {
                for expression in args.iter().chain([expression]) {
                    expression.count_calls(call_counts, caller);
                }
            }
            Self::Constructor { args, .. } => {
                for expression in args {
                    expression.count_calls(call_counts, caller);
                }
            }
            Self::If { cond, then, else_ } => {
                for expression in [cond, then, else_] {
                    expression.count_calls(call_counts, caller);
                }
            }
            Self::Literal { identifier } => {
                *call_counts
                    .entry((caller.clone(), identifier.clone()))
                    .or_default() += 1;
            }
            Self::Map {
                default_value,
                parts,
            } => {
                for expression in parts
                    .iter()
                    .map(|part| &part.expression)
                    .chain(default_value)
                {
                    expression.count_calls(call_counts, caller);
                }
            }
        }
    }
}

impl<Id: Clone + PartialEq> Expression<Id> {
    pub fn substitute_var(&mut self, var: &Id, value: &Id) -> Result<(), Error<Id>> {
        match self {
            Self::Access { lhs, rhs } => {
                Arc::make_mut(lhs).substitute_var(var, value)?;
                Arc::make_mut(rhs).substitute_var(var, value)?;
            }
            Self::BinExpr { lhs, rhs, .. } => {
                Arc::make_mut(lhs).substitute_var(var, value)?;
                Arc::make_mut(rhs).substitute_var(var, value)?;
            }
            Self::Call { expression, args } => {
                Arc::make_mut(expression).substitute_var(var, value)?;
                for arg in args {
                    Arc::make_mut(arg).substitute_var(var, value)?;
                }
            }
            Self::Constructor { identifier, .. } if identifier == var => {
                panic!("Cannot substitute constructor")
            }
            Self::Constructor { args, .. } => {
                for arg in args {
                    Arc::make_mut(arg).substitute_var(var, value)?;
                }
            }
            Self::If { cond, then, else_ } => {
                Arc::make_mut(cond).substitute_var(var, value)?;
                Arc::make_mut(then).substitute_var(var, value)?;
                Arc::make_mut(else_).substitute_var(var, value)?;
            }
            Self::Literal { identifier } if identifier == var => {
                *identifier = value.clone();
            }
            Self::Literal { .. } => {}
            Self::Map { .. } => {
                return Err(Error::NotImplemented {
                    message: "Expression::substitute_var",
                })
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ExpressionMapPart<Id> {
    pub pattern: Arc<Pattern<Id>>,
    pub expression: Arc<Expression<Id>>,
    pub domains: Vec<DomainValue<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Pattern<Id> {
    Constructor {
        identifier: Id,
        args: Vec<Arc<Pattern<Id>>>,
    },
    Literal {
        identifier: Id,
    },
    Variable {
        identifier: Id,
    },
    Wildcard,
}

impl<Id: std::fmt::Display> Pattern<Id> {
    /// Literals do not start with an uppercase letter.
    pub fn is_literal(id: &Id) -> bool {
        !id.to_string()
            .chars()
            .next()
            .is_some_and(char::is_uppercase)
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Type<Id> {
    Function {
        lhs: Arc<Type<Id>>,
        rhs: Arc<Type<Id>>,
    },
    Name {
        identifier: Id,
    },
}

impl<Id> Type<Id> {
    pub fn new(identifier: Id) -> Self {
        Self::Name { identifier }
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Value<Id> {
    Constructor {
        identifier: Id,
        args: Vec<Arc<Value<Id>>>,
    },
    Element {
        identifier: Id,
    },
    Map {
        default_value: Option<Arc<Value<Id>>>,
        entries: Vec<ValueMapEntry<Id>>,
    },
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ValueMapEntry<Id> {
    pub key: Arc<Value<Id>>,
    pub value: Arc<Value<Id>>,
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct VariableDeclaration<Id> {
    pub identifier: Id,
    pub type_: Arc<Type<Id>>,
    pub default_value: Option<Arc<Expression<Id>>>,
}

#[derive(Clone, Debug, Default, Eq, MapId, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Game<Id> {
    pub automaton: Vec<Function<Id>>,
    pub domains: Vec<DomainDeclaration<Id>>,
    pub functions: Vec<FunctionDeclaration<Id>>,
    pub variables: Vec<VariableDeclaration<Id>>,
}

impl<Id: Clone + Ord> Game<Id> {
    pub fn count_calls(&self) -> CallsCount<Id> {
        let mut call_counts = BTreeMap::new();
        for function in &self.automaton {
            function.count_calls(&mut call_counts);
        }

        call_counts
    }
}
