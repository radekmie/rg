use crate::ast::{
    ConstantDeclaration, EdgeDeclaration, EdgeLabel, EdgeName, EdgeNamePart, Expression,
    GameDeclaration, Pragma, Type, TypeDeclaration, Value, ValueEntry, VariableDeclaration,
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

pub fn constant_declaration(input: &str) -> Result<ConstantDeclaration<&str>> {
    context(
        "constant_declaration",
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

pub fn edge_declaration(input: &str) -> Result<EdgeDeclaration<&str>> {
    context(
        "edge_declaration",
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
    fn expression(input: &str) -> Result<Rc<Expression<&str>>> {
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

    context("expression", expression)(input)
}

pub fn game_declaration(input: &str) -> Result<GameDeclaration<&str>> {
    context(
        "game_declaration",
        fold_many0(
            separated(alt((
                map(constant_declaration, |x| (Some(x), None, None, None, None)),
                map(type_declaration, |x| (None, Some(x), None, None, None)),
                map(variable_declaration, |x| (None, None, Some(x), None, None)),
                map(edge_declaration, |x| (None, None, None, Some(x), None)),
                map(pragma, |x| (None, None, None, None, Some(x))),
            ))),
            GameDeclaration::default,
            |mut game_declaration, declaration| {
                match declaration {
                    (Some(x), _, _, _, _) => game_declaration.constants.push(x),
                    (_, Some(x), _, _, _) => game_declaration.types.push(x),
                    (_, _, Some(x), _, _) => game_declaration.variables.push(x),
                    (_, _, _, Some(x), _) => game_declaration.edges.push(x),
                    (_, _, _, _, Some(x)) => game_declaration.pragmas.push(x),
                    _ => unreachable!(),
                }
                game_declaration
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
    context(
        "type",
        alt((
            map_into_rc(in_braces(cut(separated(separated_list0(
                char(','),
                separated(identifier),
            ))))),
            map_into_rc(separated_pair(identifier, separated(tag("->")), cut(type_))),
            map_into_rc(identifier),
        )),
    )(input)
}

pub fn type_declaration(input: &str) -> Result<TypeDeclaration<&str>> {
    context(
        "type_declaration",
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

pub fn variable_declaration(input: &str) -> Result<VariableDeclaration<&str>> {
    context(
        "variable_declaration",
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
