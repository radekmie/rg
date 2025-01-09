use crate::ast::{
    DomainDeclaration, DomainElement, DomainElementPattern, DomainValue, Expression,
    ExpressionMapPart, Function, FunctionArg, FunctionCase, FunctionDeclaration, Pattern,
    Statement, Type, TypeDeclaration, VariableDeclaration,
};
use std::sync::Arc;

impl<Id> From<(Id, Vec<Arc<Expression<Id>>>, Arc<Expression<Id>>)> for Statement<Id> {
    fn from(
        (identifier, accessors, expression): (Id, Vec<Arc<Expression<Id>>>, Arc<Expression<Id>>),
    ) -> Self {
        Self::Assignment {
            identifier,
            accessors,
            expression,
        }
    }
}

impl<Id> From<(Id, Vec<Arc<Expression<Id>>>)> for Statement<Id> {
    fn from((identifier, args): (Id, Vec<Arc<Expression<Id>>>)) -> Self {
        Self::Call { identifier, args }
    }
}

impl<Id> From<((Id, Arc<Type<Id>>), Vec<Self>)> for Statement<Id> {
    fn from(((identifier, type_), body): ((Id, Arc<Type<Id>>), Vec<Self>)) -> Self {
        Self::Forall {
            identifier,
            type_,
            body,
        }
    }
}

impl<Id> From<Id> for DomainElement<Id> {
    fn from(identifier: Id) -> Self {
        Self::Literal { identifier }
    }
}

impl<Id> From<(Id, Vec<DomainElementPattern<Id>>, Vec<DomainValue<Id>>)> for DomainElement<Id> {
    fn from(
        (identifier, args, values): (Id, Vec<DomainElementPattern<Id>>, Vec<DomainValue<Id>>),
    ) -> Self {
        Self::Generator {
            identifier,
            args,
            values,
        }
    }
}

impl<Id> From<(Id, (usize, usize))> for DomainValue<Id> {
    fn from((identifier, (min, max)): (Id, (usize, usize))) -> Self {
        Self::Range {
            identifier,
            min,
            max,
        }
    }
}

impl<Id> From<(Id, Vec<Id>)> for DomainValue<Id> {
    fn from((identifier, elements): (Id, Vec<Id>)) -> Self {
        Self::Set {
            identifier,
            elements,
        }
    }
}

impl<Id> From<Id> for Expression<Id> {
    fn from(identifier: Id) -> Self {
        Self::Literal { identifier }
    }
}

impl<Id> From<(Arc<Self>, Arc<Self>, Arc<Self>)> for Expression<Id> {
    fn from((cond, then, else_): (Arc<Self>, Arc<Self>, Arc<Self>)) -> Self {
        Self::If { cond, then, else_ }
    }
}

impl<Id> From<(Id, Vec<Arc<Self>>)> for Expression<Id> {
    fn from((identifier, args): (Id, Vec<Arc<Self>>)) -> Self {
        Self::Constructor { identifier, args }
    }
}

impl<Id> From<(Option<Arc<Self>>, Vec<ExpressionMapPart<Id>>)> for Expression<Id> {
    fn from((default_value, parts): (Option<Arc<Self>>, Vec<ExpressionMapPart<Id>>)) -> Self {
        Self::Map {
            default_value,
            parts,
        }
    }
}

impl<Id>
    From<(
        Arc<Pattern<Id>>,
        Arc<Expression<Id>>,
        Option<Vec<DomainValue<Id>>>,
    )> for ExpressionMapPart<Id>
{
    fn from(
        (pattern, expression, domains): (
            Arc<Pattern<Id>>,
            Arc<Expression<Id>>,
            Option<Vec<DomainValue<Id>>>,
        ),
    ) -> Self {
        Self {
            pattern,
            expression,
            domains: domains.unwrap_or_default(),
        }
    }
}

impl<Id> From<(Id, Vec<Arc<Self>>)> for Pattern<Id> {
    fn from((identifier, args): (Id, Vec<Arc<Self>>)) -> Self {
        Self::Constructor { identifier, args }
    }
}

impl<Id> From<Id> for Type<Id> {
    fn from(identifier: Id) -> Self {
        Self::Name { identifier }
    }
}

impl<Id> From<(Id, Vec<FunctionArg<Id>>, Vec<Statement<Id>>)> for Function<Id> {
    fn from((name, args, body): (Id, Vec<FunctionArg<Id>>, Vec<Statement<Id>>)) -> Self {
        Self { name, args, body }
    }
}

impl<Id> From<(Id, Arc<Type<Id>>)> for FunctionArg<Id> {
    fn from((identifier, type_): (Id, Arc<Type<Id>>)) -> Self {
        Self { identifier, type_ }
    }
}

impl<Id> From<(Id, Arc<Type<Id>>)> for TypeDeclaration<Id> {
    fn from((identifier, type_): (Id, Arc<Type<Id>>)) -> Self {
        Self { identifier, type_ }
    }
}

impl<Id> From<(Id, Vec<DomainElement<Id>>)> for DomainDeclaration<Id> {
    fn from((identifier, elements): (Id, Vec<DomainElement<Id>>)) -> Self {
        Self {
            identifier,
            elements,
        }
    }
}

impl<Id> From<(Id, Arc<Type<Id>>, Option<Arc<Expression<Id>>>)> for VariableDeclaration<Id> {
    fn from(
        (identifier, type_, default_value): (Id, Arc<Type<Id>>, Option<Arc<Expression<Id>>>),
    ) -> Self {
        Self {
            identifier,
            type_,
            default_value,
        }
    }
}

impl<Id: Clone>
    From<(
        Id,
        Arc<Type<Id>>,
        Vec<((Id, Vec<Arc<Pattern<Id>>>), Arc<Expression<Id>>)>,
    )> for FunctionDeclaration<Id>
{
    fn from(
        (identifier, type_, cases): (
            Id,
            Arc<Type<Id>>,
            Vec<((Id, Vec<Arc<Pattern<Id>>>), Arc<Expression<Id>>)>,
        ),
    ) -> Self {
        let cases = cases
            .into_iter()
            .map(|((identifier, args), body)| FunctionCase {
                identifier,
                args,
                body,
            })
            .collect();
        Self {
            identifier,
            type_,
            cases,
        }
    }
}
