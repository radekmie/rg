use crate::ast::{AtomOrVariable, Game, Rule, Term};
use std::fmt::{Display, Formatter, Result};
use std::ops::Deref;
use std::rc::Rc;

impl<Symbol: Display> Game<Symbol> {
    pub fn as_prefix(&self) -> GamePrefix<Symbol> {
        GamePrefix(self)
    }
}

struct AtomOrVariablePrefix<'a, Symbol: Display>(&'a AtomOrVariable<Symbol>);

impl<Symbol: Display> Display for AtomOrVariablePrefix<'_, Symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            AtomOrVariable::Atom(symbol) => write!(f, "{symbol}"),
            AtomOrVariable::Variable(symbol) => write!(f, "?{symbol}"),
        }
    }
}

pub struct GamePrefix<'a, Symbol: Display>(&'a Game<Symbol>);

impl<Symbol: Display> Display for GamePrefix<'_, Symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0
             .0
            .iter()
            .map(RulePrefix)
            .enumerate()
            .try_for_each(|(index, rule)| {
                let separator = if index == 0 { "" } else { " " };
                write!(f, "{separator}{rule}")
            })
    }
}

struct RulePrefix<'a, Symbol: Display>(&'a Rule<Symbol>);

impl<Symbol: Display> Display for RulePrefix<'_, Symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.0.predicates.is_empty() {
            write!(f, "{}", TermPrefix(&self.0.term))
        } else {
            write!(f, "(<= {}", TermPrefix(&self.0.term))?;
            self.0
                .predicates
                .iter()
                .try_for_each(|(is_negated, predicate)| {
                    if *is_negated {
                        write!(f, " (not {})", TermPrefix(predicate))
                    } else {
                        write!(f, " {}", TermPrefix(predicate))
                    }
                })?;
            write!(f, ")")
        }
    }
}

struct TermPrefix<'a, Symbol: Display>(&'a Rc<Term<Symbol>>);

impl<Symbol: Display> Display for TermPrefix<'_, Symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0.deref() {
            Term::Base(proposition) => write!(f, "(base {})", TermPrefix(proposition)),
            Term::Custom(name, None) => write!(f, "{}", AtomOrVariablePrefix(name)),
            Term::Custom(name, Some(arguments)) => {
                write!(f, "({}", AtomOrVariablePrefix(name))?;
                arguments
                    .iter()
                    .map(TermPrefix)
                    .try_for_each(|argument| write!(f, " {argument}"))?;
                write!(f, ")")
            }
            Term::Does(role, action) => {
                write!(
                    f,
                    "(does {} {})",
                    AtomOrVariablePrefix(role),
                    TermPrefix(action)
                )
            }
            Term::Goal(role, utility) => write!(
                f,
                "(goal {} {})",
                AtomOrVariablePrefix(role),
                AtomOrVariablePrefix(utility),
            ),
            Term::Init(proposition) => write!(f, "init({})", TermPrefix(proposition)),
            Term::Input(role, action) => write!(
                f,
                "(input {} {})",
                AtomOrVariablePrefix(role),
                TermPrefix(action),
            ),
            Term::Legal(role, action) => write!(
                f,
                "(legal {} {})",
                AtomOrVariablePrefix(role),
                TermPrefix(action),
            ),
            Term::Next(proposition) => write!(f, "(next {})", TermPrefix(proposition)),
            Term::Role(role) => write!(f, "(role {})", AtomOrVariablePrefix(role)),
            Term::Terminal => write!(f, "terminal"),
            Term::True(proposition) => write!(f, "(true {})", TermPrefix(proposition)),
        }
    }
}
