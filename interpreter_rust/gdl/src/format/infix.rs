use crate::ast::{AtomOrVariable, Game, Rule, Term};
use std::fmt::{Display, Formatter, Result};
use std::ops::Deref;
use std::rc::Rc;

impl<Symbol: Display> Game<Symbol> {
    pub fn as_infix(&self) -> GameInfix<Symbol> {
        GameInfix(self)
    }
}

struct AtomOrVariableInfix<'a, Symbol: Display>(&'a AtomOrVariable<Symbol>);

impl<Symbol: Display> Display for AtomOrVariableInfix<'_, Symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0 {
            AtomOrVariable::Atom(symbol) => write!(f, "{symbol}"),
            AtomOrVariable::Variable(symbol) => {
                let mut symbol = format!("{}", symbol);
                write!(f, "{}{}", symbol.remove(0).to_uppercase(), symbol)
            }
        }
    }
}

pub struct GameInfix<'a, Symbol: Display>(&'a Game<Symbol>);

impl<Symbol: Display> Display for GameInfix<'_, Symbol> {
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

struct RuleInfix<'a, Symbol: Display>(&'a Rule<Symbol>);

impl<Symbol: Display> Display for RuleInfix<'_, Symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", TermInfix(&self.0.term))?;
        self.0
            .predicates
            .iter()
            .enumerate()
            .try_for_each(|(index, (is_negated, predicate))| {
                let separator = if index == 0 { " :- " } else { " & " };
                let negation = if *is_negated { "~" } else { "" };
                write!(f, "{separator}{negation}{}", TermInfix(predicate))
            })
    }
}

struct TermInfix<'a, Symbol: Display>(&'a Rc<Term<Symbol>>);

impl<Symbol: Display> Display for TermInfix<'_, Symbol> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.0.deref() {
            Term::Base(proposition) => write!(f, "base({})", TermInfix(proposition)),
            Term::Custom(name, arguments) => {
                write!(f, "{}", AtomOrVariableInfix(name))?;
                if let Some(arguments) = arguments {
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
