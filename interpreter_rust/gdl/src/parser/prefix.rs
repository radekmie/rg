use super::utils::{in_parens, separated, symbol, Result};
use crate::ast::{AtomOrVariable, Game, Predicate, Rule, Term};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{opt, success, value};
use nom::error::Error;
use nom::multi::{many0, many1};
use nom::sequence::preceded;
use nom::Parser;
use std::convert::identity;
use std::sync::Arc;

pub fn atom_or_variable(input: &str) -> Result<'_, AtomOrVariable<&str>> {
    alt((
        preceded(tag("?"), symbol).map(AtomOrVariable::Variable),
        symbol.map(AtomOrVariable::Atom),
    ))
    .parse(input)
}

pub fn game(input: &str) -> Result<'_, Game<&str>> {
    many0(separated(rule)).map(Game).parse(input)
}

pub fn term(input: &str) -> Result<'_, Term<&str>> {
    alt((
        term_template("base", term_rc, Term::Base),
        term_template("does", (atom_or_variable, term_rc), |(role, action)| {
            Term::Does(role, action)
        }),
        term_template(
            "goal",
            (atom_or_variable, separated(atom_or_variable)),
            |(role, utility)| Term::Goal(role, utility),
        ),
        term_template("init", term_rc, Term::Init),
        term_template("input", (atom_or_variable, term_rc), |(role, action)| {
            Term::Input(role, action)
        }),
        term_template("legal", (atom_or_variable, term_rc), |(role, action)| {
            Term::Legal(role, action)
        }),
        term_template("next", term_rc, Term::Next),
        term_template("role", atom_or_variable, Term::Role),
        term_template("terminal", success(()), |()| Term::Terminal),
        value(Term::Terminal, tag("terminal")),
        term_template("true", term_rc, Term::True),
        alt((
            (atom_or_variable, success(None)),
            in_parens((atom_or_variable, opt(many1(term_rc)))),
        ))
        .map(|(name, arguments)| Term::new_custom(name, arguments.unwrap_or_default())),
    ))
    .parse(input)
}

pub fn predicate(input: &str) -> Result<'_, Predicate<&str>> {
    alt((
        term_template("not", term_rc, |term| Predicate {
            term,
            is_negated: true,
        }),
        term_rc.map(|term| Predicate {
            term,
            is_negated: false,
        }),
    ))
    .parse(input)
}

pub fn rule(input: &str) -> Result<'_, Rule<&str>> {
    alt((
        term_template("<=", (term_rc, many1(separated(predicate))), identity),
        (term_rc, success(vec![])),
    ))
    .map(|(term, predicates)| Rule { term, predicates })
    .parse(input)
}

fn term_rc(input: &str) -> Result<'_, Arc<Term<&str>>> {
    separated(term).map(Arc::from).parse(input)
}

fn term_template<'a, T, U>(
    string: &'a str,
    parser: impl Parser<&'a str, Output = T, Error = Error<&'a str>>,
    mapper: impl Fn(T) -> U,
) -> impl Parser<&'a str, Output = U, Error = Error<&'a str>> {
    in_parens(preceded(separated(tag(string)), parser)).map(mapper)
}

#[cfg(test)]
#[test]
fn verify() {
    use nom::combinator::all_consuming;
    use nom::Finish;

    fn verify(source: &str) {
        match all_consuming(game).parse(source).finish() {
            Ok((rest, game)) => {
                assert_eq!(rest, "");
                assert_eq!(source, game.as_prefix().to_string());
            }
            Err(error) => panic!("{error}"),
        }
    }

    verify("a");
    verify("(a 1)");
    verify("(<= (a 1) b)");
    verify("(<= (a 1) (b 2))");
    verify("(<= (a 1) (b 2) c)");
    verify("(<= (a 1) (b 2) (c 3))");
    verify("(a b)");
    verify("(<= (a b) c)");
    verify("(<= (a b) (c d))");
    verify("(<= (a b) (c d) e)");
    verify("(<= (a b) (c d) (e f))");
    verify("(<= (a 1 2 3) (b c d e) (f 4 5 6 g h i))");
    verify("(a ?X)");
    verify("(<= (a ?X) (b ?Y))");
    verify("(<= (a ?X) (b ?Y) (c ?Z))");
}
