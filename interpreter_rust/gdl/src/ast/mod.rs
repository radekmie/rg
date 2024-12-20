mod eval_distinct;
mod expand_ors;
mod ground;
mod simplify;
mod substitute;
mod symbolify;
mod unify;

use map_id::MapId;
use map_id_macro::MapId;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd)]
pub enum AtomOrVariable<Id> {
    Atom(Id),
    Variable(Id),
}

impl<Id> AtomOrVariable<Id> {
    pub fn as_term(self) -> Term<Id> {
        Term::Custom0(self)
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd)]
pub struct Game<Id>(pub Vec<Rule<Id>>);

impl<Id: PartialEq> Game<Id> {
    pub fn subterms(&self) -> TermIterator<Id> {
        let mut iterator = TermIterator::new();
        iterator.add_game(self);
        iterator
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd)]
pub struct Predicate<Id> {
    pub term: Arc<Term<Id>>,
    pub is_negated: bool,
}

impl<Id> Predicate<Id> {
    pub fn has_variable(&self) -> bool {
        self.term.has_variable()
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd)]
pub struct Rule<Id> {
    pub term: Arc<Term<Id>>,
    pub predicates: Vec<Predicate<Id>>,
}

impl<Id> Rule<Id> {
    pub fn has_variable(&self) -> bool {
        self.term.has_variable() || self.predicates.iter().any(Predicate::has_variable)
    }
}

impl<Id: PartialEq> Rule<Id> {
    pub fn subterms(&self) -> TermIterator<Id> {
        let mut iterator = TermIterator::new();
        iterator.add_rule(self);
        iterator
    }
}

/// As defined in <http://logic.stanford.edu/ggp/notes/gdl.html>.
#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd)]
pub enum Term<Id> {
    /// `base(p)` means that `p` is a base proposition in the game.
    Base(Arc<Term<Id>>),

    /// `name` is a custom, game-specific term.
    Custom0(AtomOrVariable<Id>),

    /// `name(...arguments)` is a custom, game-specific term.
    CustomN(AtomOrVariable<Id>, Vec<Arc<Term<Id>>>),

    /// `does(r, a)` means that player `r` performs action `a` in the current state.
    Does(AtomOrVariable<Id>, Arc<Term<Id>>),

    /// `goal(r, u)` means that player the current state has utility `u` for player `r`.
    Goal(AtomOrVariable<Id>, AtomOrVariable<Id>),

    /// `init(p)` means that the proposition `p` is true in the initial state.
    Init(Arc<Term<Id>>),

    /// `input(r, a)` means that `a` is an action for role `r`.
    Input(AtomOrVariable<Id>, Arc<Term<Id>>),

    /// `legal(r, a)` means it is legal for role `r` to play action `a` in the current state.
    Legal(AtomOrVariable<Id>, Arc<Term<Id>>),

    /// `next(p)` means that the proposition `p` is true in the next state.
    Next(Arc<Term<Id>>),

    /// `role(a)` means that `a` is a role in the game.
    Role(AtomOrVariable<Id>),

    /// `terminal` means that the current state is a terminal state.
    Terminal,

    /// `true(p)` means that the proposition `p` is true in the current state.
    True(Arc<Term<Id>>),
}

impl<Id> Term<Id> {
    pub fn as_custom_atom(&self) -> Option<&Id> {
        match self {
            Self::Custom0(AtomOrVariable::Atom(atom))
            | Self::CustomN(AtomOrVariable::Atom(atom), _) => Some(atom),
            _ => None,
        }
    }

    pub fn has_variable(&self) -> bool {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };
        match self {
            Base(proposition) => proposition.has_variable(),
            Custom0(name) => name.is_variable(),
            CustomN(name, arguments) => {
                name.is_variable() || arguments.iter().any(|argument| argument.has_variable())
            }
            Does(role, action) => role.is_variable() || action.has_variable(),
            Goal(role, utility) => role.is_variable() || utility.is_variable(),
            Init(proposition) => proposition.has_variable(),
            Input(role, action) => role.is_variable() || action.has_variable(),
            Legal(role, action) => role.is_variable() || action.has_variable(),
            Next(proposition) => proposition.has_variable(),
            Role(role) => role.is_variable(),
            Terminal => false,
            True(proposition) => proposition.has_variable(),
        }
    }

    pub fn is_init(&self) -> bool {
        matches!(self, Self::Init(_))
    }

    pub fn new_custom(name: AtomOrVariable<Id>, arguments: Vec<Arc<Self>>) -> Self {
        match arguments.is_empty() {
            true => Self::Custom0(name),
            false => Self::CustomN(name, arguments),
        }
    }

    pub fn order(&self) -> usize {
        match self {
            Self::Base(_) => 0,
            Self::Custom0(_) => 1,
            Self::CustomN(_, _) => 2,
            Self::Does(_, _) => 3,
            Self::Goal(_, _) => 4,
            Self::Init(_) => 5,
            Self::Input(_, _) => 6,
            Self::Legal(_, _) => 7,
            Self::Next(_) => 8,
            Self::Role(_) => 9,
            Self::Terminal => 10,
            Self::True(_) => 11,
        }
    }
}

impl<Id: PartialEq> Term<Id> {
    pub fn subterms(&self) -> TermIterator<Id> {
        let mut iterator = TermIterator::new();
        iterator.add_term(self);
        iterator
    }
}

pub struct TermIterator<'a, Id> {
    index: usize,
    queue: Vec<&'a Term<Id>>,
}

impl<'a, Id: PartialEq> TermIterator<'a, Id> {
    fn add_game(&mut self, game: &'a Game<Id>) {
        for rule in &game.0 {
            self.add_rule(rule);
        }
    }

    fn add_rule(&mut self, rule: &'a Rule<Id>) {
        self.add_term(&rule.term);
        for predicate in &rule.predicates {
            self.add_term(&predicate.term);
        }
    }

    fn add_term(&mut self, term: &'a Term<Id>) {
        if matches!(
            term,
            Term::Custom0(AtomOrVariable::Variable(_))
                | Term::CustomN(AtomOrVariable::Variable(_), _)
        ) {
            return;
        }

        if !self.queue.contains(&term) {
            self.queue.push(term);
        }
    }

    fn new() -> Self {
        Self {
            index: 0,
            queue: vec![],
        }
    }
}

impl<'a, Id: PartialEq> Iterator for TermIterator<'a, Id> {
    type Item = &'a Term<Id>;

    fn next(&mut self) -> Option<Self::Item> {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };

        let maybe_term = self.queue.get(self.index).copied();
        if let Some(term) = maybe_term {
            self.index += 1;
            match term {
                Base(proposition) => self.add_term(proposition),
                Custom0(_) => {}
                CustomN(_, arguments) => arguments
                    .iter()
                    .for_each(|argument| self.add_term(argument)),
                Does(_, action) => self.add_term(action),
                Goal(_, _) => {}
                Init(proposition) => self.add_term(proposition),
                Input(_, action) => self.add_term(action),
                Legal(_, action) => self.add_term(action),
                Next(proposition) => self.add_term(proposition),
                Role(_) => {}
                Terminal => {}
                True(proposition) => self.add_term(proposition),
            }
        }

        maybe_term
    }
}
