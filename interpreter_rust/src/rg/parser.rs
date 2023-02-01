use crate::rg::ast::{
    ConstantDeclaration, EdgeDeclaration, EdgeLabel, EdgeName, EdgeNamePart, Expression,
    GameDeclaration, Type, TypeDeclaration, Value, ValueEntry, VariableDeclaration,
};
use crate::utils::parser::{in_braces, in_brackets, in_parens, map_into, ws, Result};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::char;
use nom::combinator::{cut, map, opt, success};
use nom::error::context;
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{delimited, separated_pair, terminated, tuple};
use std::rc::Rc;

pub fn constant_declaration(input: &str) -> Result<Rc<ConstantDeclaration<&str>>> {
    context(
        "constant_declaration",
        map_into(delimited(
            tag("const"),
            cut(tuple((
                ws(identifier),
                delimited(cut(char(':')), ws(type_), cut(char('='))),
                ws(value),
            ))),
            cut(char(';')),
        )),
    )(input)
}

pub fn edge_declaration(input: &str) -> Result<Rc<EdgeDeclaration<&str>>> {
    context(
        "edge_declaration",
        map_into(tuple((
            terminated(ws(edge_name), char(',')),
            terminated(ws(edge_name), cut(char(':'))),
            terminated(ws(edge_label), cut(char(';'))),
        ))),
    )(input)
}

pub fn edge_label(input: &str) -> Result<Rc<EdgeLabel<&str>>> {
    context(
        "edge_label",
        alt((
            map_into(tuple((
                expression,
                ws(map(alt((tag("=="), tag("!="))), |c| c == "!=")),
                cut(expression),
            ))),
            map_into(separated_pair(expression, ws(char('=')), cut(expression))),
            map_into(tuple((
                map(alt((char('!'), char('?'))), |c| c == '!'),
                cut(terminated(ws(edge_name), ws(tag("->")))),
                edge_name,
            ))),
            map_into(success(())),
        )),
    )(input)
}

pub fn edge_name(input: &str) -> Result<Rc<EdgeName<&str>>> {
    context("edge_name", map_into(many1(edge_name_part)))(input)
}

pub fn edge_name_part(input: &str) -> Result<Rc<EdgeNamePart<&str>>> {
    context(
        "edge_name_part",
        alt((
            map_into(in_parens(cut(separated_pair(
                ws(identifier),
                cut(char(':')),
                ws(type_),
            )))),
            map_into(identifier),
        )),
    )(input)
}

pub fn expression(input: &str) -> Result<Rc<Expression<&str>>> {
    fn expression(input: &str) -> Result<Rc<Expression<&str>>> {
        let (input, identifier) = identifier(input)?;
        let (input, maybe_cast) = opt(in_parens(cut(ws(expression))))(input)?;
        let (input, accesses) = many0(in_brackets(cut(ws(expression))))(input)?;

        let expression = accesses.into_iter().fold(
            match maybe_cast {
                Some(rhs) => Expression::Cast {
                    lhs: Type::TypeReference { identifier }.into(),
                    rhs,
                }
                .into(),
                None => Expression::Reference { identifier }.into(),
            },
            |lhs, rhs| Expression::Access { lhs, rhs }.into(),
        );

        Ok((input, expression))
    }

    context("expression", expression)(input)
}

pub fn game_declaration(input: &str) -> Result<Rc<GameDeclaration<&str>>> {
    context(
        "game_declaration",
        map(
            many0(ws(alt((
                map(constant_declaration, |x| (Some(x), None, None, None)),
                map(type_declaration, |x| (None, None, Some(x), None)),
                map(variable_declaration, |x| (None, None, None, Some(x))),
                map(edge_declaration, |x| (None, Some(x), None, None)),
            )))),
            |declarations| {
                declarations
                    .into_iter()
                    .fold(
                        GameDeclaration::default(),
                        |mut game_declaration, declaration| {
                            match declaration {
                                (Some(x), None, None, None) => game_declaration.constants.push(x),
                                (None, Some(x), None, None) => game_declaration.edges.push(x),
                                (None, None, Some(x), None) => game_declaration.types.push(x),
                                (None, None, None, Some(x)) => game_declaration.variables.push(x),
                                _ => unreachable!(),
                            }
                            game_declaration
                        },
                    )
                    .into()
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

pub fn type_(input: &str) -> Result<Rc<Type<&str>>> {
    context(
        "type",
        alt((
            map_into(in_braces(cut(ws(separated_list0(
                char(','),
                ws(identifier),
            ))))),
            map_into(separated_pair(identifier, ws(tag("->")), cut(type_))),
            map_into(identifier),
        )),
    )(input)
}

pub fn type_declaration(input: &str) -> Result<Rc<TypeDeclaration<&str>>> {
    context(
        "type_declaration",
        map_into(delimited(
            tag("type"),
            cut(separated_pair(ws(identifier), cut(char('=')), ws(type_))),
            cut(char(';')),
        )),
    )(input)
}

pub fn value(input: &str) -> Result<Rc<Value<&str>>> {
    context(
        "value",
        alt((
            map_into(in_braces(cut(ws(separated_list0(
                char(','),
                ws(value_entry),
            ))))),
            map_into(identifier),
        )),
    )(input)
}

pub fn value_entry(input: &str) -> Result<Rc<ValueEntry<&str>>> {
    context(
        "value_entry",
        map_into(separated_pair(
            opt(identifier),
            cut(ws(char(':'))),
            cut(value),
        )),
    )(input)
}

pub fn variable_declaration(input: &str) -> Result<Rc<VariableDeclaration<&str>>> {
    context(
        "variable_declaration",
        map_into(delimited(
            tag("var"),
            cut(tuple((
                ws(identifier),
                delimited(cut(char(':')), ws(type_), cut(char('='))),
                ws(value),
            ))),
            cut(char(';')),
        )),
    )(input)
}
