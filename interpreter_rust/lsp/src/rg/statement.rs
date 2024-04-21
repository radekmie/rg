use crate::common::symbol::Symbol;
use crate::completions::CompletionKind;
use rg::ast::{Constant, Edge, Label, Node, NodePart, Pragma, Type, Typedef, Variable};
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
        if self.lhs.span().encloses_position(pos) {
            completion_kind_node(pos, &self.lhs)
        } else if self.rhs.span().encloses_position(pos) {
            completion_kind_node(pos, &self.rhs)
        } else if self.label.span().encloses_position(pos) || pos > &self.label.end() {
            match self.label {
                Label::Assignment { .. } => CompletionKind::Variable,
                Label::Comparison { .. } => CompletionKind::Variable,
                Label::Reachability { .. } => CompletionKind::Edge,
                Label::Skip { .. } => CompletionKind::Variable,
                Label::Tag { .. } => CompletionKind::Param,
            }
        } else {
            CompletionKind::Edge
        }
    }

    fn keyword(&self) -> &'static str {
        ""
    }

    fn symbol_type(&self, symbol: &Symbol) -> Option<String> {
        for node in [&self.lhs, &self.rhs] {
            for node_part in &node.parts {
                if let NodePart::Binding { span, type_, .. } = node_part {
                    if span.encloses_span(&symbol.pos) {
                        return Some(type_.to_string());
                    }
                }
            }
        }

        None
    }
}

impl Statement for Pragma<Identifier> {
    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        for node in self.nodes() {
            if node.span.encloses_position(pos) {
                return completion_kind_node(pos, node);
            }
        }

        // TODO: Handle `CompletionKind::Variable` for `identifiers`.
        CompletionKind::None
    }

    fn keyword(&self) -> &'static str {
        match self {
            Self::Disjoint { .. } => "@disjoint",
            Self::DisjointExhaustive { .. } => "@disjointExhaustive",
            Self::Repeat { .. } => "@repeat",
            Self::SimpleApply { .. } => "@simpleApply",
            Self::TagIndex { .. } => "@tagIndex",
            Self::TagMaxIndex { .. } => "@tagMaxIndex",
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

fn completion_kind_node(pos: &Position, node: &Node<Identifier>) -> CompletionKind {
    node.parts
        .iter()
        .find(|node_part| node_part.span().encloses_position(pos))
        .map_or(CompletionKind::Edge, |node_part| {
            match node_part {
                NodePart::Binding {
                    identifier, type_, ..
                } if identifier.span().encloses_position(pos)
                    || type_.span().start <= identifier.span().end =>
                {
                    CompletionKind::Param
                }
                NodePart::Binding { .. } => CompletionKind::Type,
                // We can been on toplevel before an edge
                NodePart::Literal { .. } => CompletionKind::Toplevel,
            }
        })
}
