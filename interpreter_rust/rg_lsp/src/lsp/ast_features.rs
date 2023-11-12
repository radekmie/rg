use crate::rg::{ast::*, position::*, symbol::Symbol};

impl Game {
    pub fn enclosing_stat(&self, span: Span) -> Option<&Stat> {
        self.stats
            .iter()
            .find(|stat| stat.span().encloses_span(&span))
    }

    pub fn enclosing_stat_pos(&self, pos: Position) -> Option<&Stat> {
        self.stats
            .iter()
            .find(|stat| stat.span().encloses_pos(&pos))
    }

    pub fn hover_signature(&self, symbol: &Symbol) -> Option<String> {
        self.enclosing_stat(symbol.span())
            .and_then(|stat| match stat {
                Stat::Constant(Constant { type_, .. }) => {
                    Some(format!("const {}: {}", symbol.id, type_))
                }
                Stat::Variable(Variable { type_, .. }) => {
                    Some(format!("var {}: {}", symbol.id, type_))
                }
                Stat::Typedef(Typedef {
                    identifier, type_, ..
                }) => {
                    if type_.as_ref().span().encloses_span(&symbol.pos) {
                        Some(format!("{}: {}", symbol.id, identifier))
                    } else {
                        Some(format!("type {}", symbol.id))
                    }
                }
                Stat::Edge(Edge { lhs, rhs, .. }) => {
                    let left = hover_sig_edge_name(lhs, symbol);
                    left.or_else(|| hover_sig_edge_name(rhs, symbol))
                }
                _ => None,
            })
    }
}

fn hover_sig_edge_name(edge_name: &EdgeName, symbol: &Symbol) -> Option<String> {
    match edge_name.parts.as_slice() {
        [_, bindings @ ..] => bindings.iter().find_map(|binding| match binding {
            EdgeNamePart::Binding { type_, .. } => {
                if binding.span().encloses_span(&symbol.pos) {
                    Some(format!("{}: {}", symbol.id, type_))
                } else {
                    None
                }
            }
            _ => None,
        }),
        _ => None,
    }
}
