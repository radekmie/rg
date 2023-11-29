use rg::{
    ast::*,
    position::{Position, Positioned},
};

use crate::completions::CompletionKind;

use super::symbol::Symbol;

pub trait Stat: Positioned {
    fn keyword(&self) -> &'static str;
    fn symbol_type(&self, symbol: &Symbol) -> Option<String>;
    fn token_type(&self) -> &'static str {
        "keyword"
    }
    fn completion_kind(&self, pos: &Position) -> CompletionKind;
}

impl Stat for Constant<Identifier> {
    fn keyword(&self) -> &'static str {
        "const"
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        Some(self.type_.to_string())
    }

    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.identifier.span().encloses_pos(pos) {
            CompletionKind::None
        } else if self.type_.span().encloses_pos(pos) {
            CompletionKind::Type
        } else if self.value.span().encloses_pos(pos) || pos.is_after(&self.value.end()) {
            CompletionKind::Value
        } else {
            CompletionKind::Any
        }
    }
}

impl Stat for Variable<Identifier> {
    fn keyword(&self) -> &'static str {
        "var"
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        Some(self.type_.to_string())
    }

    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.identifier.span().encloses_pos(pos) {
            CompletionKind::None
        } else if self.type_.span().encloses_pos(pos) {
            CompletionKind::Type
        } else if self.default_value.span().encloses_pos(pos)
            || pos.is_after(&self.default_value.end())
        {
            CompletionKind::Value
        } else {
            CompletionKind::Any
        }
    }
}

impl Stat for Edge<Identifier> {
    fn keyword(&self) -> &'static str {
        ""
    }

    fn symbol_type(&self, symbol: &Symbol) -> Option<String> {
        let left = edge_name_label(&self.lhs, symbol);
        left.or(edge_name_label(&self.rhs, symbol))
    }

    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.lhs.span().encloses_pos(pos) {
            completion_kind_edge_name(pos, &self.lhs)
        } else if self.rhs.span().encloses_pos(pos) {
            completion_kind_edge_name(pos, &self.rhs)
        } else if self.label.span().encloses_pos(pos) || pos.is_after(&self.label.end()) {
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
}

fn edge_name_label(edge_name: &EdgeName<Identifier>, symbol: &Symbol) -> Option<String> {
    edge_name.parts.iter().find_map(|part| match part {
        EdgeNamePart::Binding { span, type_, .. } if span.encloses_span(&symbol.pos) => {
            Some(format!("{}", type_))
        }
        _ => None,
    })
}

impl Stat for Typedef<Identifier> {
    fn keyword(&self) -> &'static str {
        "type"
    }

    fn symbol_type(&self, _symbol: &Symbol) -> Option<String> {
        Some(self.identifier.to_string())
    }

    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        if self.identifier.span().encloses_pos(pos) {
            CompletionKind::None
        } else if self.type_.span().encloses_pos(pos) || pos.is_after(&self.type_.end()) {
            match self.type_.as_ref() {
                Type::Set { .. } => CompletionKind::None,
                _ => CompletionKind::Type,
            }
        } else {
            CompletionKind::Any
        }
    }
}

impl Stat for Pragma<Identifier> {
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

    fn completion_kind(&self, pos: &Position) -> CompletionKind {
        completion_kind_edge_name(pos, self.edge_name())
    }
}

fn completion_kind_edge_name(pos: &Position, edge_name: &EdgeName<Identifier>) -> CompletionKind {
    edge_name
        .parts
        .iter()
        .find(|part| part.span().encloses_pos(pos))
        .map(|part| {
            match part {
                // We can been on toplevel before an edge
                EdgeNamePart::Literal { .. } => CompletionKind::Toplevel,
                EdgeNamePart::Binding {
                    type_, identifier, ..
                } => {
                    if identifier.span().encloses_pos(pos)
                        || !type_.span().start.is_after(&identifier.span().end)
                    {
                        CompletionKind::Param
                    } else {
                        CompletionKind::Type
                    }
                }
            }
        })
        .unwrap_or(CompletionKind::Edge)
}
