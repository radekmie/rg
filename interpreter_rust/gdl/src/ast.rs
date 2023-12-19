use std::rc::Rc;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AtomOrVariable<Symbol> {
    Atom(Symbol),
    Variable(Symbol),
}

impl<Symbol: Clone + Ord> AtomOrVariable<Symbol> {
    pub fn as_term(self) -> Term<Symbol> {
        Term::Custom(self, None)
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game<Symbol>(pub Vec<Rule<Symbol>>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule<Symbol> {
    pub term: Rc<Term<Symbol>>,
    pub predicates: Vec<(bool, Rc<Term<Symbol>>)>,
}

impl<Symbol: Clone + Ord> Rule<Symbol> {
    pub fn has_variable(&self) -> bool {
        self.subterms().any(Term::has_variable)
    }

    pub fn subterms(&self) -> TermIterator<Symbol> {
        let mut iterator = TermIterator::new();
        iterator.add(&self.term);
        for (_, predicate) in &self.predicates {
            iterator.add(predicate);
        }
        iterator
    }
}

impl<Symbol> From<(Rc<Term<Symbol>>, Vec<(bool, Rc<Term<Symbol>>)>)> for Rule<Symbol> {
    fn from((term, predicates): (Rc<Term<Symbol>>, Vec<(bool, Rc<Term<Symbol>>)>)) -> Self {
        Self { term, predicates }
    }
}

/// As defined in http://logic.stanford.edu/ggp/notes/gdl.html.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Term<Symbol> {
    /// `base(p)` means that `p` is a base proposition in the game.
    Base(Rc<Term<Symbol>>),

    /// `name(...arguments)` is a custom, game-specific term.
    Custom(AtomOrVariable<Symbol>, Option<Vec<Rc<Term<Symbol>>>>),

    /// `does(r, a)` means that player `r` performs action `a` in the current state.
    Does(AtomOrVariable<Symbol>, Rc<Term<Symbol>>),

    /// `goal(r, u)` means that player the current state has utility `u` for player `r`.
    Goal(AtomOrVariable<Symbol>, AtomOrVariable<Symbol>),

    /// `init(p)` means that the proposition `p` is true in the initial state.
    Init(Rc<Term<Symbol>>),

    /// `input(r, a)` means that `a` is an action for role `r`.
    Input(AtomOrVariable<Symbol>, Rc<Term<Symbol>>),

    /// `legal(r, a)` means it is legal for role `r` to play action `a` in the current state.
    Legal(AtomOrVariable<Symbol>, Rc<Term<Symbol>>),

    /// `next(p)` means that the proposition `p` is true in the next state.
    Next(Rc<Term<Symbol>>),

    /// `role(a)` means that `a` is a role in the game.
    Role(AtomOrVariable<Symbol>),

    /// `terminal` means that the current state is a terminal state.
    Terminal,

    /// `true(p)` means that the proposition `p` is true in the current state.
    True(Rc<Term<Symbol>>),
}

impl<Symbol: Clone + Ord> Term<Symbol> {
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

pub struct TermIterator<'a, Symbol> {
    index: usize,
    queue: Vec<&'a Term<Symbol>>,
}

impl<'a, Symbol: PartialEq> TermIterator<'a, Symbol> {
    fn add(&mut self, term: &'a Term<Symbol>) {
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

impl<'a, Symbol: PartialEq> Iterator for TermIterator<'a, Symbol> {
    type Item = &'a Term<Symbol>;

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
