use crate::ast::{
    Constant, Edge, EdgeLabel, EdgeName, EdgeNamePart, Expression, Game, Identifier, Pragma, Type,
    Typedef, Value, ValueEntry, Variable,
};
use crate::position::{Span as Position, *};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, multispace1};
use nom::combinator::{all_consuming, cut, eof, into, map, opt, success, verify};
use nom::error::{context, ParseError, VerboseError};
use nom::multi::{fold_many0, many1, separated_list0};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Error(Position, String);
#[derive(Clone, Debug)]
pub struct State<'a>(&'a RefCell<Vec<Error>>);

pub type Span<'a> = LocatedSpan<&'a str, State<'a>>;
pub type Result<'a, T> = IResult<Span<'a>, T>;

fn constant(input: Span) -> Result<Constant> {
    context(
        "constant",
        into(tuple((
            tag("const"),
            cut(tuple((
                separated(identifier),
                delimited(cut(char(':')), separated(type_), cut(char('='))),
                separated(value),
            ))),
            cut(tag(";")),
        ))),
    )(input)
}

fn identifier_(input: Span) -> Result<Span> {
    static KEYWORDS: [&str; 4] = ["any", "const", "type", "var"];
    context(
        "identifier",
        verify(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            |identifier: &Span| !KEYWORDS.contains(identifier.fragment()),
        ),
    )(input)
}

fn identifier(input: Span) -> Result<Identifier> {
    map(identifier_, |identifier| {
        let span: Position = Position::from(&identifier);
        Identifier {
            span,
            identifier: identifier.fragment().to_string(),
        }
    })(input)
    // into(identifier_)(input)
}

fn edge(input: Span) -> Result<Edge> {
    context(
        "edge",
        into(tuple((
            terminated(separated(edge_name), char(',')),
            terminated(separated(edge_name), cut(char(':'))),
            separated(edge_label),
            cut(tag(";")),
        ))),
    )(input)
}

fn edge_label(input: Span) -> Result<EdgeLabel> {
    context(
        "edge_label",
        alt((
            into(preceded(char('$'), cut(separated(identifier)))),
            into(tuple((
                expression,
                separated(map(alt((tag("=="), tag("!="))), |c: Span| {
                    *c.fragment() == "!="
                })),
                cut(expression),
            ))),
            into(separated_pair(
                expression,
                separated(char('=')),
                cut(expression),
            )),
            into(tuple((
                alt((tag("!"), tag("?"))),
                cut(terminated(separated(edge_name), separated(tag("->")))),
                edge_name,
            ))),
            success::<_, _, _>(EdgeLabel::Skip {
                span: Position::from(&input),
            }),
        )),
    )(input)
}

fn edge_name(input: Span) -> Result<EdgeName> {
    context("edge_name", into(many1(separated(edge_name_part))))(input)
}

fn edge_name_part(input: Span) -> Result<EdgeNamePart> {
    context(
        "edge_name_part",
        alt((
            into(tuple((
                tag("("),
                (cut(separated_pair(
                    separated(identifier),
                    cut(char(':')),
                    separated(type_),
                ))),
                tag(")"),
            ))),
            into(identifier),
        )),
    )(input)
}

fn expression(input: Span) -> Result<Rc<Expression>> {
    // Eliminate direct left recursion.
    fn inner(input: Span) -> Result<Rc<Expression>> {
        let (input, identifier) = separated(identifier)(input)?;
        let (input, maybe_cast) =
            opt(tuple((tag("("), cut(separated(expression)), tag(")"))))(input)?;
        let (input, expression) = fold_many0(
            tuple((tag("["), (cut(separated(expression))), tag("]"))),
            || match maybe_cast.clone() {
                Some((_, rhs, r_paren)) => Rc::new(Expression::Cast {
                    span: Position::from(r_paren).with_start(identifier.span().start),
                    lhs: Rc::new(Type::TypeReference {
                        identifier: identifier.clone(),
                    }),
                    rhs,
                }),
                None => Rc::new(Expression::Reference {
                    identifier: identifier.clone(),
                }),
            },
            |lhs, (_, rhs, r_bracket)| {
                Rc::new(Expression::Access {
                    span: Position::from(r_bracket).with_start(lhs.start()),
                    lhs,
                    rhs,
                })
            },
        )(input)?;

        Ok((input, expression))
    }

    context("expression", inner)(input)
}

fn type_(input: Span) -> Result<Rc<Type>> {
    // Eliminate direct left recursion.
    fn inner(input: Span) -> Result<Rc<Type>> {
        let (input, lhs): (Span, Rc<Type>) = alt((
            into_rc(tuple((
                tag("{"),
                cut(separated(separated_list0(char(','), separated(identifier)))),
                tag("}"),
            ))),
            into_rc(identifier),
        ))(input)?;

        match opt(preceded(separated(tag("->")), type_))(input)? {
            (input, Some(rhs)) => Ok((input, Rc::new(Type::Arrow { lhs, rhs }))),
            (input, None) => Ok((input, lhs)),
        }
    }

    context("type", inner)(input)
}

fn typedef(input: Span) -> Result<Typedef> {
    context(
        "typedef",
        into(tuple((
            tag("type"),
            cut(separated_pair(
                separated(identifier),
                cut(char('=')),
                separated(type_),
            )),
            cut(tag(";")),
        ))),
    )(input)
}

fn value(input: Span) -> Result<Rc<Value>> {
    context(
        "value",
        alt((
            into_rc(tuple((
                tag("{"),
                cut(separated(separated_list0(
                    char(','),
                    separated(value_entry),
                ))),
                tag("}"),
            ))),
            into_rc(identifier),
        )),
    )(input)
}

fn value_entry(input: Span) -> Result<ValueEntry> {
    context(
        "value_entry",
        into(separated_pair(
            opt(identifier),
            cut(separated(char(':'))),
            cut(value),
        )),
    )(input)
}

fn variable(input: Span) -> Result<Variable> {
    context(
        "variable",
        into(tuple((
            tag("var"),
            cut(tuple((
                separated(identifier),
                delimited(cut(char(':')), separated(type_), cut(char('='))),
                separated(value),
            ))),
            cut(tag(";")),
        ))),
    )(input)
}

fn pragma(input: Span) -> Result<Pragma> {
    macro_rules! edge_name {
        ($tag:literal, $constructor:ident) => {
            map(
                tuple((
                    tag("@"),
                    separated(tag($tag)),
                    cut(separated(edge_name)),
                    cut(tag(";")),
                )),
                |(start, _, edge_name, end)| {
                    let span = Position::from((start, end));
                    Pragma::$constructor { span, edge_name }
                },
            )
        };
    }

    context(
        "pragma",
        alt((
            edge_name!("any", Any),
            edge_name!("disjoint", Disjoint),
            edge_name!("multiAny", MultiAny),
            edge_name!("unique", Unique),
        )),
    )(input)
}

pub fn game(input: Span) -> Result<Game> {
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

pub fn parse(input: &str) -> Game {
    let errors = RefCell::new(Vec::new());
    let input = nom_locate::LocatedSpan::new_extra(input, State(&errors));
    let (errors, game) = all_consuming(game)(input).unwrap();
    // errors.extra.to_owned();
    game
}

// Util functions

fn comment(input: Span) -> Result<Span> {
    delimited(
        tag("//"),
        cut(take_while(|c| c != '\n')),
        alt((eof, tag("\n"))),
    )(input)
}

fn comments_and_whitespaces(input: Span) -> Result<()> {
    fold_many0(alt((comment, multispace1)), || (), |_, _| ())(input)
}

macro_rules! delimited {
    ($name:ident, $prefix:expr, $suffix:expr) => {
        pub fn $name<'a, O>(
            inner: impl FnMut(Span<'a>) -> Result<O>,
        ) -> impl FnMut(Span<'a>) -> Result<O> {
            delimited($prefix, inner, $suffix)
        }
    };
}

delimited!(
    separated,
    comments_and_whitespaces,
    comments_and_whitespaces
);

fn into_rc<'a, O1, O2: From<O1>>(
    inner: impl FnMut(Span<'a>) -> Result<'a, O1>,
) -> impl FnMut(Span<'a>) -> Result<'a, Rc<O2>> {
    map(into(inner), Rc::new)
}
