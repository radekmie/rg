use crate::ast::{
    Constant, Edge, EdgeLabel, EdgeName, EdgeNamePart, Expression, Game, Pragma, Type, Typedef,
    Value, ValueEntry, Variable,
};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::char;
use nom::combinator::{cut, into, map, opt, success};
use nom::error::{context, VerboseError};
use nom::multi::{fold_many0, many1, separated_list0};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use parser_utils::{in_braces, in_brackets, in_parens, map_into_rc, separated, Result};
use std::rc::Rc;

pub fn constant(input: &str) -> Result<Constant<&str>> {
    context(
        "constant",
        into(delimited(
            tag("const"),
            cut(tuple((
                separated(identifier),
                delimited(cut(char(':')), separated(type_), cut(char('='))),
                separated(value),
            ))),
            cut(char(';')),
        )),
    )(input)
}

pub fn edge(input: &str) -> Result<Edge<&str>> {
    context(
        "edge",
        into(tuple((
            terminated(separated(edge_name), char(',')),
            terminated(separated(edge_name), cut(char(':'))),
            terminated(separated(edge_label), cut(char(';'))),
        ))),
    )(input)
}

pub fn edge_label(input: &str) -> Result<EdgeLabel<&str>> {
    context(
        "edge_label",
        alt((
            into(tuple((
                expression,
                separated(map(alt((tag("=="), tag("!="))), |c| c == "!=")),
                cut(expression),
            ))),
            into(separated_pair(
                expression,
                separated(char('=')),
                cut(expression),
            )),
            into(tuple((
                map(alt((char('!'), char('?'))), |c| c == '!'),
                cut(terminated(separated(edge_name), separated(tag("->")))),
                edge_name,
            ))),
            into(success::<_, _, VerboseError<_>>(())),
        )),
    )(input)
}

pub fn edge_name(input: &str) -> Result<EdgeName<&str>> {
    context("edge_name", into(many1(edge_name_part)))(input)
}

pub fn edge_name_part(input: &str) -> Result<EdgeNamePart<&str>> {
    context(
        "edge_name_part",
        alt((
            into(in_parens(cut(separated_pair(
                separated(identifier),
                cut(char(':')),
                separated(type_),
            )))),
            into(identifier),
        )),
    )(input)
}

pub fn expression(input: &str) -> Result<Rc<Expression<&str>>> {
    // Eliminate direct left recursion.
    fn inner(input: &str) -> Result<Rc<Expression<&str>>> {
        let (input, identifier) = identifier(input)?;
        let (input, maybe_cast) = opt(in_parens(cut(separated(expression))))(input)?;
        let (input, expression) = fold_many0(
            in_brackets(cut(separated(expression))),
            || match maybe_cast.clone() {
                Some(rhs) => Rc::new(Expression::Cast {
                    lhs: Rc::new(Type::TypeReference { identifier }),
                    rhs,
                }),
                None => Rc::new(Expression::Reference { identifier }),
            },
            |lhs, rhs| Rc::new(Expression::Access { lhs, rhs }),
        )(input)?;

        Ok((input, expression))
    }

    context("expression", inner)(input)
}

pub fn game(input: &str) -> Result<Game<&str>> {
    context(
        "game",
        fold_many0(
            separated(alt((
                map(constant, |x| (Some(x), None, None, None, None)),
                map(typedef, |x| (None, Some(x), None, None, None)),
                map(variable, |x| (None, None, Some(x), None, None)),
                map(edge, |x| (None, None, None, Some(x), None)),
                map(pragma, |x| (None, None, None, None, Some(x))),
            ))),
            Game::default,
            |mut game, declaration| {
                match declaration {
                    (Some(x), _, _, _, _) => game.constants.push(x),
                    (_, Some(x), _, _, _) => game.typedefs.push(x),
                    (_, _, Some(x), _, _) => game.variables.push(x),
                    (_, _, _, Some(x), _) => game.edges.push(x),
                    (_, _, _, _, Some(x)) => game.pragmas.push(x),
                    _ => unreachable!(),
                }
                game
            },
        ),
    )(input)
}

pub fn identifier(input: &str) -> Result<&str> {
    context(
        "identifier",
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
    )(input)
}

pub fn pragma(input: &str) -> Result<Pragma<&str>> {
    context(
        "pragma",
        into(delimited(
            tag("@"),
            cut(preceded(tag("disjoint "), separated(edge_name))),
            cut(char(';')),
        )),
    )(input)
}

pub fn type_(input: &str) -> Result<Rc<Type<&str>>> {
    // Eliminate direct left recursion.
    fn inner(input: &str) -> Result<Rc<Type<&str>>> {
        let (input, lhs) = alt((
            map_into_rc(in_braces(cut(separated(separated_list0(
                char(','),
                separated(identifier),
            ))))),
            map_into_rc(identifier),
        ))(input)?;

        match opt(preceded(separated(tag("->")), type_))(input)? {
            (input, Some(rhs)) => Ok((input, Rc::new(Type::Arrow { lhs, rhs }))),
            (input, None) => Ok((input, lhs)),
        }
    }

    context("type", inner)(input)
}

pub fn typedef(input: &str) -> Result<Typedef<&str>> {
    context(
        "typedef",
        into(delimited(
            tag("type"),
            cut(separated_pair(
                separated(identifier),
                cut(char('=')),
                separated(type_),
            )),
            cut(char(';')),
        )),
    )(input)
}

pub fn value(input: &str) -> Result<Rc<Value<&str>>> {
    context(
        "value",
        alt((
            map_into_rc(in_braces(cut(separated(separated_list0(
                char(','),
                separated(value_entry),
            ))))),
            map_into_rc(identifier),
        )),
    )(input)
}

pub fn value_entry(input: &str) -> Result<ValueEntry<&str>> {
    context(
        "value_entry",
        into(separated_pair(
            opt(identifier),
            cut(separated(char(':'))),
            cut(value),
        )),
    )(input)
}

pub fn variable(input: &str) -> Result<Variable<&str>> {
    context(
        "variable",
        into(delimited(
            tag("var"),
            cut(tuple((
                separated(identifier),
                delimited(cut(char(':')), separated(type_), cut(char('='))),
                separated(value),
            ))),
            cut(char(';')),
        )),
    )(input)
}
