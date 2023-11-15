use rg::{
    ast::*,
    position::{Positioned, Span},
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Stat<'a> {
    Constant(&'a Constant<Identifier>),
    Edge(&'a Edge<Identifier>),
    Pragma(&'a Pragma<Identifier>),
    Typedef(&'a Typedef<Identifier>),
    Variable(&'a Variable<Identifier>),
}

impl Stat<'_> {
    pub fn from_game(game: &Game<Identifier>) -> Vec<Stat> {
        let mut stats = Vec::new();
        for typedef in game.typedefs.iter() {
            stats.push(Stat::Typedef(typedef));
        }
        for variable in game.variables.iter() {
            stats.push(Stat::Variable(variable));
        }
        for constant in game.constants.iter() {
            stats.push(Stat::Constant(constant));
        }
        for edge in game.edges.iter() {
            stats.push(Stat::Edge(edge));
        }
        for pragma in game.pragmas.iter() {
            stats.push(Stat::Pragma(pragma));
        }
        stats
    }
}

impl Positioned for Stat<'_> {
    fn span(&self) -> Span {
        match self {
            Stat::Constant(constant) => constant.span(),
            Stat::Edge(edge) => edge.span(),
            Stat::Pragma(pragma) => pragma.span(),
            Stat::Typedef(typedef) => typedef.span(),
            Stat::Variable(variable) => variable.span(),
        }
    }
}
