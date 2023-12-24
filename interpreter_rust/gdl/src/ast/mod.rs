pub mod ground;
pub mod simplify;
pub mod substitute;
pub mod unify;

use map_id::MapId;
use map_id_macro::MapId;
use std::rc::Rc;

#[derive(Clone, Debug, MapId, PartialEq)]
pub enum AtomOrVariable<Id> {
    Atom(Id),
    Variable(Id),
}

impl<Id> AtomOrVariable<Id> {
    pub fn as_term(self) -> Term<Id> {
        Term::Custom(self, None)
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }
}

#[derive(Clone, Debug, MapId, PartialEq)]
pub struct Game<Id>(pub Vec<Rule<Id>>);

#[derive(Clone, Debug, MapId, PartialEq)]
pub struct Predicate<Id> {
    pub is_negated: bool,
    pub term: Rc<Term<Id>>,
}

#[derive(Clone, Debug, MapId, PartialEq)]
pub struct Rule<Id> {
    pub term: Rc<Term<Id>>,
    pub predicates: Vec<Predicate<Id>>,
}

impl<Id: PartialEq> Rule<Id> {
    pub fn has_variable(&self) -> bool {
        self.subterms().any(Term::has_variable)
    }

    pub fn subterms(&self) -> impl Iterator<Item = &Term<Id>> {
        let mut iterator = TermIterator::new();
        iterator.add(&self.term);
        for predicate in &self.predicates {
            iterator.add(&predicate.term);
        }
        iterator
    }
}

/// As defined in http://logic.stanford.edu/ggp/notes/gdl.html.
#[derive(Clone, Debug, MapId, PartialEq)]
pub enum Term<Id> {
    /// `base(p)` means that `p` is a base proposition in the game.
    Base(Rc<Term<Id>>),

    /// `name(...arguments)` is a custom, game-specific term.
    Custom(AtomOrVariable<Id>, Option<Vec<Rc<Term<Id>>>>),

    /// `does(r, a)` means that player `r` performs action `a` in the current state.
    Does(AtomOrVariable<Id>, Rc<Term<Id>>),

    /// `goal(r, u)` means that player the current state has utility `u` for player `r`.
    Goal(AtomOrVariable<Id>, AtomOrVariable<Id>),

    /// `init(p)` means that the proposition `p` is true in the initial state.
    Init(Rc<Term<Id>>),

    /// `input(r, a)` means that `a` is an action for role `r`.
    Input(AtomOrVariable<Id>, Rc<Term<Id>>),

    /// `legal(r, a)` means it is legal for role `r` to play action `a` in the current state.
    Legal(AtomOrVariable<Id>, Rc<Term<Id>>),

    /// `next(p)` means that the proposition `p` is true in the next state.
    Next(Rc<Term<Id>>),

    /// `role(a)` means that `a` is a role in the game.
    Role(AtomOrVariable<Id>),

    /// `terminal` means that the current state is a terminal state.
    Terminal,

    /// `true(p)` means that the proposition `p` is true in the current state.
    True(Rc<Term<Id>>),
}

impl<Id> Term<Id> {
    pub fn has_variable(&self) -> bool {
        use Term::*;
        match self {
            Base(proposition) => proposition.has_variable(),
            Custom(name, None) => name.is_variable(),
            Custom(_, arguments) => arguments
                .iter()
                .flatten()
                .any(|argument| argument.has_variable()),
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
}

struct TermIterator<'a, Id> {
    index: usize,
    queue: Vec<&'a Term<Id>>,
}

impl<'a, Id: PartialEq> TermIterator<'a, Id> {
    fn add(&mut self, term: &'a Term<Id>) {
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
            queue: Vec::new(),
        }
    }
}

impl<'a, Id: PartialEq> Iterator for TermIterator<'a, Id> {
    type Item = &'a Term<Id>;

    fn next(&mut self) -> Option<Self::Item> {
        let maybe_term = self.queue.get(self.index).copied();
        if let Some(term) = maybe_term {
            self.index += 1;

            use Term::*;
            match term {
                Base(proposition) => self.add(proposition),
                Custom(_, arguments) => arguments
                    .iter()
                    .flatten()
                    .for_each(|argument| self.add(argument)),
                Does(_, action) => self.add(action),
                Goal(_, _) => {}
                Init(proposition) => self.add(proposition),
                Input(_, action) => self.add(action),
                Legal(_, action) => self.add(action),
                Next(proposition) => self.add(proposition),
                Role(_) => {}
                Terminal => {}
                True(proposition) => self.add(proposition),
            }
        }

        maybe_term
    }
}
