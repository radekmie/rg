use std::{fmt::Display, sync::Arc};
use utils::{
    position::{Positioned, Span},
    Identifier,
};

type RgType = rg::ast::Type<Identifier>;
type HrgType = hrg::ast::Type<Identifier>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    pub flag: Flag,
    pub id: String,
    pub owners: Option<Vec<usize>>,
    pub pos: Span,
    pub type_: Type,
}

impl Symbol {
    pub fn new(id: String, pos: Span, flag: Flag, owners: Option<Vec<usize>>, type_: Type) -> Self {
        Self {
            flag,
            id,
            owners,
            pos,
            type_,
        }
    }

    pub fn from_id(identifier: &Identifier, flag: Flag, type_: Type) -> Option<Symbol> {
        if identifier.is_none() {
            None
        } else {
            let id = identifier.identifier.clone();
            let pos = identifier.span();
            Some(Self::new(id, pos, flag, None, type_))
        }
    }

    pub fn is_owned_by(&self, owner: usize) -> bool {
        self.owners
            .as_ref()
            .is_some_and(|owners| owners.contains(&owner))
    }

    pub fn safe_pos(&self) -> Option<Span> {
        if self.pos.is_none() {
            None
        } else {
            Some(self.pos)
        }
    }
}

pub fn defined(symbols: &[Symbol], name: &str, flag: &Flag) -> Option<usize> {
    symbols
        .iter()
        .enumerate()
        .find(|(_, symbol)| symbol.id == name && symbol.flag == *flag)
        .map(|(idx, _)| idx)
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.id, self.flag)
    }
}

impl Positioned for Symbol {
    fn span(&self) -> Span {
        self.pos
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Type {
    Hrg(Arc<HrgType>),
    Rg(Arc<RgType>),
    NoType,
}

impl Type {
    pub fn to_option(&self) -> Option<&Self> {
        match self {
            Type::Hrg(_) | Type::Rg(_) => Some(self),
            Type::NoType => None,
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rg(type_) => write!(f, "{type_}"),
            Self::Hrg(type_) => write!(f, "{type_}"),
            Self::NoType => write!(f, "<notype>"),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Flag {
    Type,
    Member,
    Constant,
    Variable,
    Function,
    Param,
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type => write!(f, "#"),
            Self::Member => write!(f, "."),
            Self::Constant => write!(f, "!"),
            Self::Variable => write!(f, "?"),
            Self::Function => write!(f, "()"),
            Self::Param => write!(f, "$"),
        }
    }
}

pub fn make_builtin(symbol: &str, flag: Flag) -> Symbol {
    Symbol::new(symbol.to_string(), Span::none(), flag, None, Type::NoType)
}
