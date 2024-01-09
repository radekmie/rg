use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use std::fmt::{Display, Formatter, Result};
use std::rc::Rc;

impl<Id: Display> Game<Id> {
    pub fn as_infix(&self) -> GameInfix<Id> {
        GameInfix(self)
    }
}

struct AtomOrVariableInfix<'a, Id: Display>(&'a AtomOrVariable<Id>);

impl<Id: Display> Display for AtomOrVariableInfix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            AtomOrVariable::Atom(symbol) => write!(f, "{symbol}"),
            AtomOrVariable::Variable(symbol) => {
                let mut symbol = format!("{symbol}");
                write!(f, "{}{}", symbol.remove(0).to_uppercase(), symbol)
            }
        }
    }
}

pub struct GameInfix<'a, Id: Display>(&'a Game<Id>);

impl<Id: Display> Display for GameInfix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0
             .0
            .iter()
            .map(RuleInfix)
            .enumerate()
            .try_for_each(|(index, rule)| {
                let separator = if index == 0 { "" } else { " " };
                write!(f, "{separator}{rule}")
            })
    }
}

struct PredicateInfix<'a, Id: Display>(&'a Predicate<Id>);

impl<Id: Display> Display for PredicateInfix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let negation = if self.0.is_negated { "~" } else { "" };
        write!(f, "{negation}{}", TermInfix(&self.0.term))
    }
}

struct RuleInfix<'a, Id: Display>(&'a Rule<Id>);

impl<Id: Display> Display for RuleInfix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", TermInfix(&self.0.term))?;
        self.0
            .predicates
            .iter()
            .enumerate()
            .try_for_each(|(index, predicate)| {
                let separator = if index == 0 { " :- " } else { " & " };
                write!(f, "{separator}{}", PredicateInfix(predicate))
            })
    }
}

struct TermInfix<'a, Id: Display>(&'a Rc<Term<Id>>);

impl<Id: Display> Display for TermInfix<'_, Id> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &**self.0 {
            Term::Base(proposition) => write!(f, "base({})", TermInfix(proposition)),
            Term::Custom(name, arguments) => {
                write!(f, "{}", AtomOrVariableInfix(name))?;
                if !arguments.is_empty() {
                    write!(f, "(")?;
                    arguments
                        .iter()
                        .enumerate()
                        .try_for_each(|(index, argument)| {
                            let separator = if index == 0 { "" } else { ", " };
                            write!(f, "{separator}{}", TermInfix(argument))
                        })?;
                    write!(f, ")")?;
                }

                Ok(())
            }
            Term::Does(role, action) => {
                write!(
                    f,
                    "does({}, {})",
                    AtomOrVariableInfix(role),
                    TermInfix(action)
                )
            }
            Term::Goal(role, utility) => write!(
                f,
                "goal({}, {})",
                AtomOrVariableInfix(role),
                AtomOrVariableInfix(utility),
            ),
            Term::Init(proposition) => write!(f, "init({})", TermInfix(proposition)),
            Term::Input(role, action) => write!(
                f,
                "input({}, {})",
                AtomOrVariableInfix(role),
                TermInfix(action),
            ),
            Term::Legal(role, action) => write!(
                f,
                "legal({}, {})",
                AtomOrVariableInfix(role),
                TermInfix(action),
            ),
            Term::Next(proposition) => write!(f, "next({})", TermInfix(proposition)),
            Term::Role(role) => write!(f, "role({})", AtomOrVariableInfix(role)),
            Term::Terminal => write!(f, "terminal"),
            Term::True(proposition) => write!(f, "true({})", TermInfix(proposition)),
        }
    }
}
