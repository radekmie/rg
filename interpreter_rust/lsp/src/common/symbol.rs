use std::fmt::Display;

use utils::{
    position::{Positioned, Span},
    Identifier,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Symbol {
    pub flag: Flag,
    pub id: String,
    pub owners: Option<Vec<usize>>,
    pub pos: Span,
}

impl Symbol {
    pub fn new(id: String, pos: Span, flag: Flag, owners: Option<Vec<usize>>) -> Self {
        Self {
            flag,
            id,
            owners,
            pos,
        }
    }

    pub fn from_id(identifier: &Identifier, flag: Flag) -> Option<Self> {
        if identifier.is_none() {
            None
        } else {
            let id = identifier.identifier.clone();
            let pos = identifier.span();
            Some(Self::new(id, pos, flag, None))
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
pub enum Flag {
    Type,
    Member,
    Constant,
    Variable,
    Edge,
    Param,
}

impl Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type => write!(f, "#"),
            Self::Member => write!(f, "."),
            Self::Constant => write!(f, "!"),
            Self::Variable => write!(f, "?"),
            Self::Edge => write!(f, "()"),
            Self::Param => write!(f, "$"),
        }
    }
}
