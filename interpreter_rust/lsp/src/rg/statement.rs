use crate::common::symbol::Symbol;
use crate::completions::CompletionKind;
use rg::ast::{Constant, Edge, Label, Pragma, Type, Typedef, Variable};
use utils::{
    position::{Position, Positioned},
    Identifier,
};

pub trait Statement: Positioned {
    fn completion_kind(&self, pos: &Position) -> CompletionKind;
    fn keyword(&self) -> &'static str;
    fn symbol_type(&self, symbol: &Symbol) -> Option<String>;
    fn token_type(&self) -> &'static str {
        "keyword"
    }
}

impl Statement for Constant<Identifier> {
    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.identifier.span().encloses_position(pos) {
            CompletionKind::None
        } else if self.type_.span().encloses_position(pos) {
            CompletionKind::Type
        } else if self.value.span().encloses_position(pos) || pos > &self.value.end() {
            CompletionKind::Value
        } else {
            CompletionKind::Any
        }
    }

    fn keyword(&self) -> &'static str {
        "const"
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        Some(self.type_.to_string())
    }
}

impl Statement for Edge<Identifier> {
    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.label.span().encloses_position(pos) || pos > &self.label.end() {
            match self.label {
                Label::Assignment { .. } => CompletionKind::Variable,
                Label::AssignmentAny { .. } => CompletionKind::Variable,
                Label::Comparison { .. } => CompletionKind::Variable,
                Label::Reachability { .. } => CompletionKind::Edge,
                Label::Skip { .. } => CompletionKind::Variable,
                Label::Tag { .. } => CompletionKind::Param,
                Label::TagVariable { .. } => CompletionKind::Param,
            }
        } else {
            CompletionKind::Edge
        }
    }

    fn keyword(&self) -> &'static str {
        ""
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        None
    }
}

impl Statement for Pragma<Identifier> {
    fn completion_kind(&self, _pos: &Position) -> CompletionKind {
        CompletionKind::None
    }

    fn keyword(&self) -> &'static str {
        match self {
            Self::ArtificialTag { .. } => "@artificialTag",
            Self::Disjoint { .. } => "@disjoint",
            Self::DisjointExhaustive { .. } => "@disjointExhaustive",
            Self::Repeat { .. } => "@repeat",
            Self::SimpleApply { .. } => "@simpleApply",
            Self::SimpleApplyExhaustive { .. } => "@simpleApplyExhaustive",
            Self::TagIndex { .. } => "@tagIndex",
            Self::TagMaxIndex { .. } => "@tagMaxIndex",
            Self::TranslatedFromRbg { .. } => "@translatedFromRbg",
            Self::Unique { .. } => "@unique",
        }
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        None
    }

    fn token_type(&self) -> &'static str {
        "macro"
    }
}

impl Statement for Typedef<Identifier> {
    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.identifier.span().encloses_position(pos) {
            CompletionKind::None
        } else if self.type_.span().encloses_position(pos) || pos > &self.type_.end() {
            match self.type_.as_ref() {
                Type::Set { .. } => CompletionKind::None,
                _ => CompletionKind::Type,
            }
        } else {
            CompletionKind::Any
        }
    }

    fn keyword(&self) -> &'static str {
        "type"
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        Some(self.identifier.to_string())
    }
}

impl Statement for Variable<Identifier> {
    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.identifier.span().encloses_position(pos) {
            CompletionKind::None
        } else if self.type_.span().encloses_position(pos) {
            CompletionKind::Type
        } else if self.default_value.span().encloses_position(pos)
            || pos > &self.default_value.end()
        {
            CompletionKind::Value
        } else {
            CompletionKind::Any
        }
    }

    fn keyword(&self) -> &'static str {
        "var"
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        Some(self.type_.to_string())
    }
}
