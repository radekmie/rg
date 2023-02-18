use crate::ast::{
    ConstantDeclaration, EdgeDeclaration, EdgeLabel, EdgeName, EdgeNamePart, Expression, Pragma,
    Type, TypeDeclaration, Value, ValueEntry, VariableDeclaration,
};
use std::rc::Rc;

impl<Id> From<(Id, Rc<Type<Id>>, Rc<Value<Id>>)> for ConstantDeclaration<Id> {
    fn from((identifier, type_, value): (Id, Rc<Type<Id>>, Rc<Value<Id>>)) -> Self {
        Self {
            identifier,
            type_,
            value,
        }
    }
}

impl<Id> From<(Rc<EdgeName<Id>>, Rc<EdgeName<Id>>, Rc<EdgeLabel<Id>>)> for EdgeDeclaration<Id> {
    fn from((lhs, rhs, label): (Rc<EdgeName<Id>>, Rc<EdgeName<Id>>, Rc<EdgeLabel<Id>>)) -> Self {
        Self { label, lhs, rhs }
    }
}

impl<Id> From<(Rc<Expression<Id>>, Rc<Expression<Id>>)> for EdgeLabel<Id> {
    fn from((lhs, rhs): (Rc<Expression<Id>>, Rc<Expression<Id>>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl<Id> From<(Rc<Expression<Id>>, bool, Rc<Expression<Id>>)> for EdgeLabel<Id> {
    fn from((lhs, negated, rhs): (Rc<Expression<Id>>, bool, Rc<Expression<Id>>)) -> Self {
        Self::Comparison { lhs, rhs, negated }
    }
}

impl<Id> From<(bool, Rc<EdgeName<Id>>, Rc<EdgeName<Id>>)> for EdgeLabel<Id> {
    fn from((negated, lhs, rhs): (bool, Rc<EdgeName<Id>>, Rc<EdgeName<Id>>)) -> Self {
        Self::Reachability { lhs, rhs, negated }
    }
}

impl<Id> From<()> for EdgeLabel<Id> {
    fn from(_: ()) -> Self {
        Self::Skip
    }
}

impl<Id> From<(Id, Rc<Type<Id>>)> for EdgeNamePart<Id> {
    fn from((identifier, type_): (Id, Rc<Type<Id>>)) -> Self {
        Self::Binding { identifier, type_ }
    }
}

impl<Id> From<Id> for EdgeNamePart<Id> {
    fn from(identifier: Id) -> Self {
        Self::Literal { identifier }
    }
}

impl<Id> From<(Rc<Expression<Id>>, Rc<Expression<Id>>)> for Expression<Id> {
    fn from((lhs, rhs): (Rc<Expression<Id>>, Rc<Expression<Id>>)) -> Self {
        Self::Access { lhs, rhs }
    }
}

impl<Id> From<(Id, Rc<Expression<Id>>)> for Expression<Id> {
    fn from((identifier, rhs): (Id, Rc<Expression<Id>>)) -> Self {
        Self::Cast {
            lhs: Type::TypeReference { identifier }.into(),
            rhs,
        }
    }
}

impl<Id> From<Id> for Expression<Id> {
    fn from(identifier: Id) -> Self {
        Self::Reference { identifier }
    }
}

impl<Id> From<Rc<EdgeName<Id>>> for Pragma<Id> {
    fn from(edge_name: Rc<EdgeName<Id>>) -> Self {
        Self::Disjoint { edge_name }
    }
}

impl<Id> From<(Id, Rc<Type<Id>>)> for Type<Id> {
    fn from((lhs, rhs): (Id, Rc<Type<Id>>)) -> Self {
        Self::Arrow { lhs, rhs }
    }
}

impl<Id> From<Vec<Id>> for Type<Id> {
    fn from(identifiers: Vec<Id>) -> Self {
        Self::Set { identifiers }
    }
}

impl<Id> From<Id> for Type<Id> {
    fn from(identifier: Id) -> Self {
        Self::TypeReference { identifier }
    }
}

impl<Id> From<(Id, Rc<Type<Id>>)> for TypeDeclaration<Id> {
    fn from((identifier, type_): (Id, Rc<Type<Id>>)) -> Self {
        Self { identifier, type_ }
    }
}

impl<Id> From<Id> for Value<Id> {
    fn from(identifier: Id) -> Self {
        Self::Element { identifier }
    }
}

impl<Id> From<Vec<Rc<ValueEntry<Id>>>> for Value<Id> {
    fn from(entries: Vec<Rc<ValueEntry<Id>>>) -> Self {
        Self::Map { entries }
    }
}

impl<Id> From<(Option<Id>, Rc<Value<Id>>)> for ValueEntry<Id> {
    fn from((identifier, value): (Option<Id>, Rc<Value<Id>>)) -> Self {
        match identifier {
            None => Self::DefaultEntry { value },
            Some(identifier) => Self::NamedEntry { identifier, value },
        }
    }
}

impl<Id> From<(Id, Rc<Type<Id>>, Rc<Value<Id>>)> for VariableDeclaration<Id> {
    fn from((identifier, type_, default_value): (Id, Rc<Type<Id>>, Rc<Value<Id>>)) -> Self {
        Self {
            default_value,
            identifier,
            type_,
        }
    }
}
