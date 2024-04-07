use crate::ast::{
    Constant, Edge, Expression, Identifier, Label, Node, NodePart, Type, Typedef, Value,
    ValueEntry, Variable,
};
use crate::parsing::parser::Input;
use crate::position::{Positioned, Span};
use std::sync::Arc;

impl<Id: Positioned> From<(Input<'_>, (Id, Arc<Type<Id>>, Arc<Value<Id>>), Span)> for Constant<Id> {
    fn from(
        (start, (identifier, type_, value), end): (
            Input,
            (Id, Arc<Type<Id>>, Arc<Value<Id>>),
            Span,
        ),
    ) -> Self {
        Self {
            span: end.with_start((&start).into()),
            identifier,
            type_,
            value,
        }
    }
}

impl<Id: Positioned> From<(Node<Id>, (Node<Id>, Label<Id>), Span)> for Edge<Id> {
    fn from((lhs, (rhs, label), end): (Node<Id>, (Node<Id>, Label<Id>), Span)) -> Self {
        Self {
            span: end.with_start(lhs.start()),
            label,
            lhs,
            rhs,
        }
    }
}

impl<Id> From<Id> for Label<Id> {
    fn from(symbol: Id) -> Self {
        Self::Tag { symbol }
    }
}

impl<Id> From<(Arc<Expression<Id>>, Arc<Expression<Id>>)> for Label<Id> {
    fn from((lhs, rhs): (Arc<Expression<Id>>, Arc<Expression<Id>>)) -> Self {
        Self::Assignment { lhs, rhs }
    }
}

impl<Id> From<(Arc<Expression<Id>>, &str, Arc<Expression<Id>>)> for Label<Id> {
    fn from((lhs, separator, rhs): (Arc<Expression<Id>>, &str, Arc<Expression<Id>>)) -> Self {
        match separator {
            "!=" => Self::Comparison {
                lhs,
                rhs,
                negated: true,
            },
            "==" => Self::Comparison {
                lhs,
                rhs,
                negated: false,
            },
            "=" => Self::Assignment { lhs, rhs },
            _ => unreachable!(),
        }
    }
}

impl<Id: Positioned> From<(Input<'_>, Node<Id>, Node<Id>)> for Label<Id> {
    fn from((tag, lhs, rhs): (Input, Node<Id>, Node<Id>)) -> Self {
        let negated = *tag.fragment() == "!";
        Self::Reachability {
            span: Span::from(&tag).with_end(rhs.span().end),
            lhs,
            rhs,
            negated,
        }
    }
}

impl<Id: Positioned> From<Id> for Expression<Id> {
    fn from(identifier: Id) -> Self {
        Self::Reference { identifier }
    }
}

impl<Id: Positioned> From<(Id, Vec<NodePart<Id>>)> for Node<Id> {
    fn from((identifier, bindings): (Id, Vec<NodePart<Id>>)) -> Self {
        let first = NodePart::from(identifier);
        let last = bindings.last().unwrap_or(&first);
        let span = first.span().with_end(last.end());
        let mut parts = vec![first];
        parts.extend(bindings);
        Self { span, parts }
    }
}

impl<Id: Positioned> From<Id> for Node<Id> {
    fn from(identifier: Id) -> Self {
        Self::from((identifier, vec![]))
    }
}

impl<Id: Positioned> From<(Input<'_>, Id, Arc<Type<Id>>, Input<'_>)> for NodePart<Id> {
    fn from((start, identifier, type_, end): (Input, Id, Arc<Type<Id>>, Input)) -> Self {
        Self::Binding {
            span: Span::from((&start, &end)),
            identifier,
            type_,
        }
    }
}

impl<Id> From<Id> for NodePart<Id> {
    fn from(identifier: Id) -> Self {
        Self::Literal { identifier }
    }
}

impl From<Input<'_>> for Identifier {
    fn from(value: Input) -> Self {
        Self::new(Span::from(&value), (*value.fragment()).to_string())
    }
}

impl<Id: Positioned> From<(Input<'_>, Vec<Id>, Input<'_>)> for Type<Id> {
    fn from((start, identifiers, end): (Input, Vec<Id>, Input)) -> Self {
        let span = Span::from((&start, &end));
        Self::Set { span, identifiers }
    }
}

impl<Id> From<Id> for Type<Id> {
    fn from(identifier: Id) -> Self {
        Self::TypeReference { identifier }
    }
}

impl<Id: Positioned> From<(Input<'_>, (Id, Arc<Type<Id>>), Span)> for Typedef<Id> {
    fn from((start, (identifier, type_), end): (Input, (Id, Arc<Type<Id>>), Span)) -> Self {
        Self {
            span: end.with_start((&start).into()),
            identifier,
            type_,
        }
    }
}

impl From<(Input<'_>, Input<'_>)> for Value<Identifier> {
    fn from((start, end): (Input<'_>, Input<'_>)) -> Self {
        Self::Map {
            span: Span::from((&start, &end)),
            entries: vec![],
        }
    }
}

impl<Id> From<Id> for Value<Id> {
    fn from(identifier: Id) -> Self {
        Self::Element { identifier }
    }
}

impl<Id> From<(Input<'_>, Vec<Option<ValueEntry<Id>>>, Input<'_>)> for Value<Id> {
    fn from((start, entries, end): (Input<'_>, Vec<Option<ValueEntry<Id>>>, Input<'_>)) -> Self {
        let span = Span::from((&start, &end));
        let entries = entries.into_iter().flatten().collect();
        Self::Map { span, entries }
    }
}

impl<Id> From<(Span, Option<Id>, Arc<Value<Id>>)> for ValueEntry<Id> {
    fn from((span, identifier, value): (Span, Option<Id>, Arc<Value<Id>>)) -> Self {
        Self {
            span,
            identifier,
            value,
        }
    }
}

impl<Id: Positioned> From<(Input<'_>, (Id, Arc<Type<Id>>, Arc<Value<Id>>), Span)> for Variable<Id> {
    fn from(
        (start, (identifier, type_, default_value), end): (
            Input,
            (Id, Arc<Type<Id>>, Arc<Value<Id>>),
            Span,
        ),
    ) -> Self {
        Self {
            span: end.with_start((&start).into()),
            default_value,
            identifier,
            type_,
        }
    }
}
