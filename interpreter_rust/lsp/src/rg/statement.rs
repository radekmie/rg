use super::symbol::Symbol;
use crate::completions::CompletionKind;
use rg::ast::*;
use rg::position::{Position, Positioned};

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
        } else if self.value.span().encloses_position(pos) || pos.is_after(&self.value.end()) {
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
            completion_kind_edge_name(pos, &self.lhs)
        } else if self.rhs.span().encloses_position(pos) {
            completion_kind_edge_name(pos, &self.rhs)
        } else if self.label.span().encloses_position(pos) || pos.is_after(&self.label.end()) {
            match self.label {
                EdgeLabel::Assignment { .. } => CompletionKind::Variable,
                EdgeLabel::Comparison { .. } => CompletionKind::Variable,
                EdgeLabel::Reachability { .. } => CompletionKind::Edge,
                EdgeLabel::Skip { .. } => CompletionKind::Variable,
                EdgeLabel::Tag { .. } => CompletionKind::Param,
            }
        } else {
            CompletionKind::Edge
        }
    }

    fn keyword(&self) -> &'static str {
        ""
    }

    fn symbol_type(&self, symbol: &Symbol) -> Option<String> {
        for edge_name in [&self.lhs, &self.rhs] {
            for part in &edge_name.parts {
                if let EdgeNamePart::Binding { span, type_, .. } = part {
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
        completion_kind_edge_name(pos, self.edge_name())
    }

    fn keyword(&self) -> &'static str {
        match self {
            Pragma::Any { .. } => "@any",
            Pragma::Disjoint { .. } => "@disjoint",
            Pragma::MultiAny { .. } => "@multiAny",
            Pragma::Unique { .. } => "@unique",
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
        } else if self.type_.span().encloses_position(pos) || pos.is_after(&self.type_.end()) {
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
            || pos.is_after(&self.default_value.end())
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

fn completion_kind_edge_name(pos: &Position, edge_name: &EdgeName<Identifier>) -> CompletionKind {
    edge_name
        .parts
        .iter()
        .find(|part| part.span().encloses_position(pos))
        .map(|part| {
            match part {
                EdgeNamePart::Binding {
                    identifier, type_, ..
                } if identifier.span().encloses_position(pos)
                    || !type_.span().start.is_after(&identifier.span().end) =>
                {
                    CompletionKind::Param
                }
                EdgeNamePart::Binding { .. } => CompletionKind::Type,
                // We can been on toplevel before an edge
                EdgeNamePart::Literal { .. } => CompletionKind::Toplevel,
            }
        })
        .unwrap_or(CompletionKind::Edge)
}
