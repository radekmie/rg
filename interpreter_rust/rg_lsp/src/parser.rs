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
use std::rc::Rc;

type Span<'a> = LocatedSpan<&'a str>;
pub type Result<'a, T> = IResult<Span<'a>, T, VerboseError<Span<'a>>>;

pub fn constant(input: Span) -> Result<Constant> {
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

pub fn identifier_(input: Span) -> Result<Span> {
    static KEYWORDS: [&str; 4] = ["any", "const", "type", "var"];
    context(
        "identifier",
        verify(
            take_while1(|c: char| c.is_alphanumeric() || c == '_'),
            |identifier: &LocatedSpan<&str>| !KEYWORDS.contains(identifier.fragment()),
        ),
    )(input)
}

pub fn identifier(input: Span) -> Result<Identifier> {
    into(identifier_)(input)
}

pub fn edge<'a>(input: Span<'a>) -> Result<Edge<'a>> {
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

pub fn edge_label<'a>(input: Span<'a>) -> Result<EdgeLabel<'a>> {
    context(
        "edge_label",
        alt((
            into(preceded(char('$'), cut(separated(identifier)))),
            into(tuple((
                expression,
                separated(map(alt((tag("=="), tag("!="))), |c: Span<'_>| {
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
            success::<_, _, VerboseError<_>>(EdgeLabel::Skip {
                span: Position::from((input, input)),
            }),
        )),
    )(input)
}

pub fn edge_name<'a>(input: Span<'a>) -> Result<EdgeName<'a>> {
    context("edge_name", into(many1(separated(edge_name_part))))(input)
}

pub fn edge_name_part<'a>(input: Span<'a>) -> Result<EdgeNamePart<'a>> {
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

pub fn expression<'a>(input: Span<'a>) -> Result<Rc<Expression<'a>>> {
    // Eliminate direct left recursion.
    fn inner<'a>(input: Span<'a>) -> Result<Rc<Expression<'a>>> {
        let (input, identifier) = separated(identifier)(input)?;
        let (input, maybe_cast) =
            opt(tuple((tag("("), cut(separated(expression)), tag(")"))))(input)?;
        let (input, expression) = fold_many0(
            tuple((tag("["), (cut(separated(expression))), tag("]"))),
            || match maybe_cast.clone() {
                Some((_, rhs, r_paren)) => Rc::new(Expression::Cast {
                    span: Position::from((r_paren, r_paren)).with_start(identifier.span().start),
                    lhs: Rc::new(Type::TypeReference { identifier }),
                    rhs,
                }),
                None => Rc::new(Expression::Reference { identifier }),
            },
            |lhs, (_, rhs, r_bracket)| {
                Rc::new(Expression::Access {
                    span: Position::from((r_bracket, r_bracket)).with_start(lhs.start()),
                    lhs,
                    rhs,
                })
            },
        )(input)?;

        Ok((input, expression))
    }

    context("expression", inner)(input)
}

pub fn type_<'a>(input: Span<'a>) -> Result<Rc<Type<'a>>> {
    // Eliminate direct left recursion.
    fn inner<'a>(input: Span<'a>) -> Result<Rc<Type<'a>>> {
        let (input, lhs): (LocatedSpan<&str>, Rc<Type<'_>>) = alt((
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

pub fn typedef<'a>(input: Span<'a>) -> Result<Typedef<'a>> {
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

pub fn value<'a>(input: Span<'a>) -> Result<Rc<Value<'a>>> {
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

pub fn value_entry(input: Span) -> Result<ValueEntry> {
    context(
        "value_entry",
        into(separated_pair(
            opt(identifier),
            cut(separated(char(':'))),
            cut(value),
        )),
    )(input)
}

pub fn variable<'a>(input: Span<'a>) -> Result<Variable<'a>> {
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

pub fn pragma<'a>(input: Span<'a>) -> Result<Pragma<'a>> {
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

pub fn game<'a>(input: Span<'a>) -> Result<Game<'a>> {
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

pub fn comment(input: Span) -> Result<Span> {
    delimited(
        tag("//"),
        cut(take_while(|c| c != '\n')),
        alt((eof, tag("\n"))),
    )(input)
}

pub fn comments_and_whitespaces(input: Span) -> Result<()> {
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

pub fn into_rc<'a, O1, O2: From<O1>>(
    inner: impl FnMut(Span<'a>) -> Result<'a, O1>,
) -> impl FnMut(Span<'a>) -> Result<'a, Rc<O2>> {
    map(into(inner), Rc::new)
}

pub fn parse(input: &str) -> Game {
    let input = nom_locate::LocatedSpan::new(input);
    let (_, game) = all_consuming(game)(input).unwrap();
    game
}
