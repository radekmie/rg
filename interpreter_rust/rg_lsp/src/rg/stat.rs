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
        game.typedefs.iter().for_each(|typedef| {
            stats.push(Stat::Typedef(typedef));
        });
        game.constants.iter().for_each(|constant| {
            stats.push(Stat::Constant(constant));
        });
        game.variables.iter().for_each(|variable| {
            stats.push(Stat::Variable(variable));
        });
        game.edges.iter().for_each(|edge| {
            stats.push(Stat::Edge(edge));
        });
        game.pragmas.iter().for_each(|pragma| {
            stats.push(Stat::Pragma(pragma));
        });
        stats
    }

    pub fn keyword(&self) -> &'static str {
        match self {
            Stat::Constant(_) => "const",
            Stat::Edge(_) => "",
            Stat::Typedef(_) => "typ",
            Stat::Variable(_) => "var",
            Stat::Pragma(Pragma::Any { .. }) => "@any",
            Stat::Pragma(Pragma::Disjoint { .. }) => "@disjoint",
            Stat::Pragma(Pragma::MultiAny { .. }) => "@multiAny",
            Stat::Pragma(Pragma::Unique { .. }) => "@unique",
        }
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
