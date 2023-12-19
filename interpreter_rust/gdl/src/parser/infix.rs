use super::utils::{in_parens, separated, symbol, Result};
use crate::ast::{AtomOrVariable, Game, Rule, Term};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::{into, map, opt, success, value};
use nom::multi::{many0, separated_list1};
use nom::sequence::{pair, preceded, separated_pair};
use std::rc::Rc;

pub fn atom_or_variable(input: &str) -> Result<AtomOrVariable<&str>> {
    map(symbol, |symbol: &str| {
        if symbol.chars().next().unwrap().is_uppercase() {
            AtomOrVariable::Variable(symbol)
        } else {
            AtomOrVariable::Atom(symbol)
        }
    })(input)
}

pub fn game(input: &str) -> Result<Game<&str>> {
    map(many0(separated(rule)), Game)(input)
}

pub fn term(input: &str) -> Result<Term<&str>> {
    alt((
        term_template("base", term_rc, Term::Base),
        term_template("does", pair(atom_or_variable, term_rc), |(role, action)| {
            Term::Does(role, action)
        }),
        term_template(
            "goal",
            separated_pair(atom_or_variable, separated(tag(",")), atom_or_variable),
            |(role, utility)| Term::Goal(role, utility),
        ),
        term_template("init", term_rc, Term::Init),
        term_template(
            "input",
            separated_pair(atom_or_variable, separated(tag(",")), term_rc),
            |(role, action)| Term::Input(role, action),
        ),
        term_template(
            "legal",
            separated_pair(atom_or_variable, separated(tag(",")), term_rc),
            |(role, action)| Term::Legal(role, action),
        ),
        term_template("next", term_rc, Term::Next),
        term_template("role", atom_or_variable, Term::Role),
        value(Term::Terminal, tag("terminal")),
        term_template("true", term_rc, Term::True),
        map(
            pair(
                atom_or_variable,
                opt(in_parens(separated_list1(tag(","), term_rc))),
            ),
            |(name, arguments)| Term::Custom(name, arguments),
        ),
    ))(input)
}

fn term_rc(input: &str) -> Result<Rc<Term<&str>>> {
    map(separated(term), Rc::from)(input)
}

fn term_template<'a, T, U>(
    string: &'a str,
    parser: impl FnMut(&'a str) -> Result<T>,
    mapper: impl Fn(T) -> U,
) -> impl FnMut(&'a str) -> Result<U> {
    map(preceded(tag(string), in_parens(parser)), mapper)
}

pub fn rule(input: &str) -> Result<Rule<&str>> {
    let predicate = pair(map(opt(tag("~")), |negation| negation.is_some()), term_rc);
    let predicates = preceded(
        separated(tag(":-")),
        separated_list1(separated(tag("&")), predicate),
    );

    into(pair(term_rc, alt((predicates, success(vec![])))))(input)
}

#[cfg(test)]
#[test]
fn verify() {
    use nom::combinator::all_consuming;
    use nom::Finish;

    fn verify(source: &str) {
        match all_consuming(game)(source).finish() {
            Ok((rest, game)) => {
                assert_eq!(rest, "");
                assert_eq!(source, game.as_infix().to_string());
            }
            Err(error) => assert!(false, "{error}"),
        }
    }

    verify("a");
    verify("a(1)");
    verify("a(1) :- b");
    verify("a(1) :- b(2)");
    verify("a(1) :- b(2) & c");
    verify("a(1) :- b(2) & c(3)");
    verify("a(b)");
    verify("a(b) :- c");
    verify("a(b) :- c(d)");
    verify("a(b) :- c(d) & e");
    verify("a(b) :- c(d) & e(f)");
    verify("a(1, 2, 3) :- b(c, d, e) & f(4, 5, 6, g, h, i)");
    verify("a(X)");
    verify("a(X) :- b(Y)");
    verify("a(X) :- b(Y) & c(Z)");
}
