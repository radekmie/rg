use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::fmt::{Display, Formatter, Result};
use std::sync::Arc;

impl<Id: Display> Game<Id> {
    pub fn as_prefix(&self) -> GamePrefix<Id> {
        GamePrefix(self)
    }
}

struct AtomOrVariablePrefix<'a, Id: Display>(&'a AtomOrVariable<Id>);

impl<Id: Display> Display for AtomOrVariablePrefix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            AtomOrVariable::Atom(symbol) => write!(f, "{symbol}"),
            AtomOrVariable::Variable(symbol) => write!(f, "?{symbol}"),
        }
    }
}

pub struct GamePrefix<'a, Id: Display>(&'a Game<Id>);

impl<Id: Display> Display for GamePrefix<'_, Id> {
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

struct PredicatePrefix<'a, Id: Display>(&'a Predicate<Id>);

impl<Id: Display> Display for PredicatePrefix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.0.is_negated {
            write!(f, "(not {})", TermPrefix(&self.0.term))
        } else {
            write!(f, "{}", TermPrefix(&self.0.term))
        }
    }
}

struct RulePrefix<'a, Id: Display>(&'a Rule<Id>);

impl<Id: Display> Display for RulePrefix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.0.predicates.is_empty() {
            write!(f, "{}", TermPrefix(&self.0.term))
        } else {
            write!(f, "(<= {}", TermPrefix(&self.0.term))?;
            self.0
                .predicates
                .iter()
                .try_for_each(|predicate| write!(f, " {}", PredicatePrefix(predicate)))?;
            write!(f, ")")
        }
    }
}

struct TermPrefix<'a, Id: Display>(&'a Arc<Term<Id>>);

impl<Id: Display> Display for TermPrefix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &**self.0 {
            Term::Base(proposition) => write!(f, "(base {})", TermPrefix(proposition)),
            Term::Custom0(name) => write!(f, "{}", AtomOrVariablePrefix(name)),
            Term::CustomN(name, arguments) => {
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
