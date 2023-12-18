use std::collections::BTreeSet;
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

    fn atoms_into<'a>(&'a self, atoms: &mut BTreeSet<&'a Symbol>) {
        match self {
            Self::Atom(symbol) => {
                atoms.insert(symbol);
            }
            Self::Variable(_) => {}
        }
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }

    fn variables_into<'a>(&'a self, variables: &mut BTreeSet<&'a Symbol>) {
        match self {
            Self::Atom(_) => {}
            Self::Variable(symbol) => {
                variables.insert(symbol);
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Game<Symbol>(pub Vec<Rule<Symbol>>);

impl<Symbol: Clone + Ord> Game<Symbol> {
    pub fn atoms(&self) -> BTreeSet<&Symbol> {
        let mut atoms = BTreeSet::new();
        self.atoms_into(&mut atoms);
        atoms
    }

    fn atoms_into<'a>(&'a self, atoms: &mut BTreeSet<&'a Symbol>) {
        for term_with_predicate in &self.0 {
            term_with_predicate.atoms_into(atoms);
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rule<Symbol> {
    pub term: Rc<Term<Symbol>>,
    pub predicates: Option<Vec<(bool, Rc<Term<Symbol>>)>>,
}

impl<Symbol: Clone + Ord> Rule<Symbol> {
    fn atoms_into<'a>(&'a self, atoms: &mut BTreeSet<&'a Symbol>) {
        for term in self.subterms() {
            term.atoms_into(atoms);
        }
    }

    pub fn has_variable(&self) -> bool {
        self.subterms().into_iter().any(Term::has_variable)
    }

    pub fn subterms(&self) -> BTreeSet<&Term<Symbol>> {
        let mut subterms = BTreeSet::new();
        self.term.subterms_into(&mut subterms);
        self.predicates
            .iter()
            .flatten()
            .for_each(|(_, predicate)| predicate.subterms_into(&mut subterms));
        subterms
    }

    pub fn variables(&self) -> BTreeSet<&Symbol> {
        let mut variables = BTreeSet::new();
        self.variables_into(&mut variables);
        variables
    }

    fn variables_into<'a>(&'a self, variables: &mut BTreeSet<&'a Symbol>) {
        for term in self.subterms() {
            term.variables_into(variables);
        }
    }
}

impl<Symbol> From<(Rc<Term<Symbol>>, Option<Vec<(bool, Rc<Term<Symbol>>)>>)> for Rule<Symbol> {
    fn from((term, predicates): (Rc<Term<Symbol>>, Option<Vec<(bool, Rc<Term<Symbol>>)>>)) -> Self {
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
    fn atoms_into<'a>(&'a self, atoms: &mut BTreeSet<&'a Symbol>) {
        use Term::*;
        match self {
            Base(proposition) => proposition.atoms_into(atoms),
            Custom(name, None) => name.atoms_into(atoms),
            Custom(_, arguments) => arguments
                .iter()
                .flatten()
                .for_each(|argument| argument.atoms_into(atoms)),
            Does(role, action) => {
                role.atoms_into(atoms);
                action.atoms_into(atoms);
            }
            Goal(role, utility) => {
                role.atoms_into(atoms);
                utility.atoms_into(atoms);
            }
            Init(proposition) => proposition.atoms_into(atoms),
            Input(role, action) => {
                role.atoms_into(atoms);
                action.atoms_into(atoms);
            }
            Legal(role, action) => {
                role.atoms_into(atoms);
                action.atoms_into(atoms);
            }
            Next(proposition) => proposition.atoms_into(atoms),
            Role(role) => role.atoms_into(atoms),
            Terminal => {}
            True(proposition) => proposition.atoms_into(atoms),
        }
    }

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

    fn subterms_into<'a>(&'a self, subterms: &mut BTreeSet<&'a Term<Symbol>>) {
        use Term::*;

        if !matches!(self, Custom(AtomOrVariable::Variable(_), _)) {
            subterms.insert(self);
        }

        match self {
            Base(proposition) => proposition.subterms_into(subterms),
            Custom(_, arguments) => arguments
                .iter()
                .flatten()
                .for_each(|argument| argument.subterms_into(subterms)),
            Does(_, action) => action.subterms_into(subterms),
            Goal(_, _) => {}
            Init(proposition) => proposition.subterms_into(subterms),
            Input(_, action) => action.subterms_into(subterms),
            Legal(_, action) => action.subterms_into(subterms),
            Next(proposition) => proposition.subterms_into(subterms),
            Role(_) => {}
            Terminal => {}
            True(proposition) => proposition.subterms_into(subterms),
        }
    }

    fn variables_into<'a>(&'a self, variables: &mut BTreeSet<&'a Symbol>) {
        use Term::*;
        match self {
            Base(proposition) => proposition.variables_into(variables),
            Custom(name, None) => name.variables_into(variables),
            Custom(_, arguments) => arguments
                .iter()
                .flatten()
                .for_each(|argument| argument.variables_into(variables)),
            Does(role, action) => {
                role.variables_into(variables);
                action.variables_into(variables);
            }
            Goal(role, utility) => {
                role.variables_into(variables);
                utility.variables_into(variables);
            }
            Init(proposition) => proposition.variables_into(variables),
            Input(role, action) => {
                role.variables_into(variables);
                action.variables_into(variables);
            }
            Legal(role, action) => {
                role.variables_into(variables);
                action.variables_into(variables);
            }
            Next(proposition) => proposition.variables_into(variables),
            Role(role) => role.variables_into(variables),
            Terminal => {}
            True(proposition) => proposition.variables_into(variables),
        }
    }
}
