use crate::ast::{
    Constant, Edge, Expression, Label, Node, Pragma, Type, Typedef, Value, ValueEntry, Variable,
};
use utils::position::{Positioned, Span};

impl<Id> Positioned for Constant<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id> Positioned for Edge<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id: Positioned> Positioned for Label<Id> {
    fn span(&self) -> Span {
        match self {
            Self::Assignment { lhs, rhs } | Self::Comparison { lhs, rhs, .. } => {
                lhs.span().with_end(rhs.span().end)
            }
            Self::AssignmentAny { lhs, rhs } => lhs.span().with_end(rhs.span().end),
            Self::Reachability { span, .. } | Self::Skip { span } => *span,
            Self::Tag { symbol } => symbol.span(),
            Self::TagVariable { identifier } => identifier.span(),
        }
    }
}

impl<Id: Positioned> Positioned for Node<Id> {
    fn span(&self) -> Span {
        self.identifier.span()
    }
}

impl<Id: Positioned> Positioned for Expression<Id> {
    fn span(&self) -> Span {
        match &self {
            Self::Access { span, .. } | Self::Cast { span, .. } => *span,
            Self::Reference { identifier } => identifier.span(),
        }
    }
}

impl<Id> Positioned for Pragma<Id> {
    fn span(&self) -> Span {
        match self {
            Self::ArtificialTag { span, .. }
            | Self::Disjoint { span, .. }
            | Self::DisjointExhaustive { span, .. }
            | Self::Repeat { span, .. }
            | Self::SimpleApply { span, .. }
            | Self::SimpleApplyExhaustive { span, .. }
            | Self::TagIndex { span, .. }
            | Self::TagMaxIndex { span, .. }
            | Self::TranslatedFromRbg { span }
            | Self::Unique { span, .. } => *span,
        }
    }
}

impl<Id: Positioned> Positioned for Type<Id> {
    fn span(&self) -> Span {
        match &self {
            Self::Arrow { lhs, rhs } => lhs.span().with_end(rhs.span().end),
            Self::Set { span, .. } => *span,
            Self::TypeReference { identifier } => identifier.span(),
        }
    }
}

impl<Id> Positioned for Typedef<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id: Positioned> Positioned for Value<Id> {
    fn span(&self) -> Span {
        match &self {
            Self::Element { identifier } => identifier.span(),
            Self::Map { span, .. } => *span,
        }
    }
}

impl<Id> Positioned for ValueEntry<Id> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<Id> Positioned for Variable<Id> {
    fn span(&self) -> Span {
        self.span
    }
}
