use crate::ast::*;
use crate::position::Span;

pub struct DocumentSymbols {
    pub symbols: Vec<Symbol>,
    pub occurences: Vec<Occurrence>,
}

pub struct Symbol {
    pub id: String,
    pub pos: Span,
    pub flags: u32,
    pub owner: Option<String>,
}

impl Symbol {
    fn new(id: String, pos: Span, flags: u32, owner: Option<String>) -> Self {
        Self {
            id,
            pos,
            flags,
            owner,
        }
    }

    fn from_id(identifier: &Identifier) -> Self {
        let id = identifier.identifier.clone();
        let pos = identifier.span.clone();
        let flags = 0;
        let owner = None;
        Self::new(id, pos, flags, owner)
    }

    fn from_type(type_: &Type) -> Vec<Self> {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                let mut symbols = Self::from_type(lhs);
                symbols.append(&mut Self::from_type(rhs));
                symbols
            }
            Type::TypeReference { .. } => vec![],
            Type::Set { identifiers, .. } => identifiers
                .iter()
                .map(|id| Self::from_id(id))
                .collect::<Vec<Self>>(),
        }
    }

    fn from_edge_name(edge_name: &EdgeName) -> Vec<Self> {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => vec![Self::from_id(identifier)],
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                let symbol = Self::from_id(identifier);
                let mut bindings = bindings
                    .iter()
                    .map(|binding| Self::from_name_part(binding, &symbol.id))
                    .collect::<Vec<Self>>();
                let mut symbols = vec![symbol];
                symbols.append(&mut bindings);
                symbols
            }
            _ => vec![],
        }
    }

    fn from_name_part(name_part: &EdgeNamePart, owner: &str) -> Self {
        match name_part {
            EdgeNamePart::Binding { identifier, .. } => {
                let mut symbol = Symbol::from_id(identifier);
                symbol.owner = Some(owner.to_string());
                symbol
            }
            EdgeNamePart::Literal { identifier } => Symbol::from_id(identifier),
        }
    }
}

pub struct Occurrence {
    pub id: String,
    pub pos: Span,
}

impl Occurrence {
    fn new(id: String, pos: Span) -> Self {
        Self { id, pos }
    }

    fn from_id(identifier: &Identifier) -> Self {
        let id = identifier.identifier.clone();
        let pos = identifier.span.clone();
        Self::new(id, pos)
    }

    fn from_type(type_: &Type) -> Vec<Self> {
        match type_ {
            Type::Arrow { lhs, rhs } => {
                let mut occurrences = Self::from_type(lhs);
                occurrences.append(&mut Self::from_type(rhs));
                occurrences
            }
            Type::TypeReference { identifier } => vec![Self::from_id(identifier)],
            Type::Set { identifiers, .. } => identifiers
                .iter()
                .map(|id| Self::from_id(id))
                .collect::<Vec<Self>>(),
        }
    }

    fn from_edge(edge: &Edge) -> Vec<Self> {
        let mut occurrences = Self::from_edge_name(&edge.lhs);
        occurrences.append(&mut Self::from_edge_name(&edge.rhs));
        occurrences.append(&mut Self::from_edge_label(&edge.label));
        occurrences
    }

    fn from_edge_label(label: &EdgeLabel) -> Vec<Self> {
        match label {
            EdgeLabel::Assignment { lhs, rhs } => {
                let mut occurrences = Self::from_expression(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
            EdgeLabel::Comparison { lhs, rhs, .. } => {
                let mut occurrences = Self::from_expression(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
            EdgeLabel::Skip { .. } => vec![],
            EdgeLabel::Tag { symbol } => vec![Self::from_id(symbol)],
            EdgeLabel::Reachability { lhs, rhs, .. } => {
                let mut occurrences = Self::from_edge_name(lhs);
                occurrences.append(&mut Self::from_edge_name(rhs));
                occurrences
            }
        }
    }

    fn from_expression(expr: &Expression) -> Vec<Self> {
        match expr {
            Expression::Reference { identifier } => vec![Self::from_id(identifier)],
            Expression::Access { lhs, rhs, .. } => {
                let mut occurrences = Self::from_expression(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
            Expression::Cast { lhs, rhs, .. } => {
                let mut occurrences = Self::from_type(lhs);
                occurrences.append(&mut Self::from_expression(rhs));
                occurrences
            }
        }
    }

    fn from_edge_name(edge_name: &EdgeName) -> Vec<Self> {
        match edge_name.parts.as_slice() {
            [EdgeNamePart::Literal { identifier }] => vec![Self::from_id(identifier)],
            [EdgeNamePart::Literal { identifier }, bindings @ ..] => {
                let symbol = Self::from_id(identifier);
                let mut bindings = bindings
                    .iter()
                    .map(|binding| Self::from_name_part(binding))
                    .collect::<Vec<Self>>();
                let mut occurrences = vec![symbol];
                occurrences.append(&mut bindings);
                occurrences
            }
            _ => vec![],
        }
    }

    fn from_name_part(name_part: &EdgeNamePart) -> Self {
        match name_part {
            EdgeNamePart::Binding { identifier, .. } => Self::from_id(identifier),
            EdgeNamePart::Literal { identifier } => Self::from_id(identifier),
        }
    }
}

impl DocumentSymbols {
    pub fn new(game: &Game) -> Self {
        let symbols = Self::get_definitions(game);
        let occurences = Self::get_occurences(game);
        Self {
            symbols,
            occurences,
        }
    }

    pub fn get_definitions(game: &Game) -> Vec<Symbol> {
        let mut symbols: Vec<Symbol> = Vec::new();
        for typedef in game.typedefs.iter() {
            let id = &typedef.identifier;
            let symbol = Symbol::from_id(id);
            symbols.push(symbol);
            symbols.append(&mut Symbol::from_type(&typedef.type_));
        }
        for constant in game.constants.iter() {
            let id = &constant.identifier;
            let symbol = Symbol::from_id(id);
            symbols.push(symbol);
        }
        for variable in game.variables.iter() {
            let id = &variable.identifier;
            let symbol = Symbol::from_id(id);
            symbols.push(symbol);
        }

        for edge in game.edges.iter() {
            let mut l_symbols = Symbol::from_edge_name(&edge.lhs);
            let mut r_symbols = Symbol::from_edge_name(&edge.rhs);
            symbols.append(&mut l_symbols);
            symbols.append(&mut r_symbols);
        }
        symbols
    }

    pub fn get_occurences(game: &Game) -> Vec<Occurrence> {
        let mut occurrences: Vec<Occurrence> = Vec::new();
        for typedef in game.typedefs.iter() {
            let id = &typedef.identifier;
            let occ = Occurrence::from_id(id);
            occurrences.push(occ);
            occurrences.append(&mut Occurrence::from_type(&typedef.type_));
        }
        for constant in game.constants.iter() {
            let id = &constant.identifier;
            let symbol = Occurrence::from_id(id);
            occurrences.push(symbol);
        }
        for variable in game.variables.iter() {
            let id = &variable.identifier;
            let symbol = Occurrence::from_id(id);
            occurrences.push(symbol);
        }
        for edge in game.edges.iter() {
            occurrences.append(&mut Occurrence::from_edge(edge));
        }
        for pragma in game.pragmas.iter() {
            //TODO: Impl accessor methods on pragmas (and everything else actually)
            // At least `identifier` would be nice
            todo!()
        }
        occurrences
    }
}
