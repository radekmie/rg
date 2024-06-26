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
        Term::Custom(self, vec![])
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }
}

#[derive(Clone, Debug, Eq, MapId, Ord, PartialEq, PartialOrd)]
pub struct Game<Id>(pub Vec<Rule<Id>>);

impl<Id: PartialEq> Game<Id> {
    pub fn subterms(&self) -> impl Iterator<Item = &Term<Id>> {
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
    pub fn subterms(&self) -> impl Iterator<Item = &Term<Id>> {
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

    /// `name(...arguments)` is a custom, game-specific term.
    Custom(AtomOrVariable<Id>, Vec<Arc<Term<Id>>>),

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
    pub fn has_variable(&self) -> bool {
        use Term::{Base, Custom, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True};
        match self {
            Base(proposition) => proposition.has_variable(),
            Custom(name, arguments) => {
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
}

impl<Id: PartialEq> Term<Id> {
    pub fn subterms(&self) -> impl Iterator<Item = &Self> {
        let mut iterator = TermIterator::new();
        iterator.add_term(self);
        iterator
    }
}

struct TermIterator<'a, Id> {
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
        if matches!(term, Term::Custom(AtomOrVariable::Variable(_), _)) {
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
        use Term::{Base, Custom, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True};

        let maybe_term = self.queue.get(self.index).copied();
        if let Some(term) = maybe_term {
            self.index += 1;
            match term {
                Base(proposition) => self.add_term(proposition),
                Custom(_, arguments) => arguments
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
