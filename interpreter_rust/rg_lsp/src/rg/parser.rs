use crate::rg::ast::*;
use crate::rg::position::{Span as Position, *};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{char, multispace1};
use nom::combinator::{all_consuming, cut, eof, into, map, opt, success, verify};
use nom::error::{context, ParseError, VerboseError};
use nom::multi::{fold_many0, many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Debug)]
pub struct Error(Position, String);
#[derive(Clone, Debug)]
pub struct State<'a>(&'a RefCell<Vec<Error>>);

pub type Span<'a> = LocatedSpan<&'a str, State<'a>>;
pub type Result<'a, T> = IResult<Span<'a>, T>;

fn constant(input: Span) -> Result<Option<Constant>> {
    context(
        "constant",
        map(
            with_semicolon(
                tag("const"),
                cut(tuple((
                    separated(identifier),
                    delimited(cut(char(':')), separated(type_), cut(char('='))),
                    separated(value),
                ))),
                "expected constant",
            ),
            |(tag_span, res)| res.map(|res| (tag_span, res).into()),
        ),
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
        Identifier::new(span, identifier.fragment().to_string())
    })(input)
    // into(identifier_)(input)
}

fn edge(input: Span) -> Result<Option<Edge>> {
    context(
        "edge",
        map(
            with_semicolon(
                terminated(separated(edge_name), char(',')),
                tuple((
                    terminated(separated(edge_name), cut(char(':'))),
                    separated(edge_label),
                )),
                "expected edge",
            ),
            |(lhs, res)| res.map(|(rhs, label)| (lhs, rhs, label).into()),
        ),
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
            into(delimited(
                tag("("),
                cut(separated_pair(
                    separated(identifier),
                    cut(char(':')),
                    separated(type_),
                )),
                tag(")"),
            )),
            into(identifier),
        )),
    )(input)
}

fn expression(input: Span) -> Result<Arc<Expression>> {
    // Eliminate direct left recursion.
    fn inner(input: Span) -> Result<Arc<Expression>> {
        let (input, identifier) = separated(identifier)(input)?;
        let (input, maybe_cast) =
            opt(delimited(tag("("), cut(separated(expression)), tag(")")))(input)?;
        let (input, expression) = fold_many0(
            delimited(tag("["), cut(separated(expression)), tag("]")),
            || match maybe_cast.clone() {
                Some(rhs) => Arc::new(Expression::Cast {
                    span: rhs.span().with_start(identifier.span().start),
                    lhs: Arc::new(Type::TypeReference {
                        identifier: identifier.clone(),
                    }),
                    rhs,
                }),
                None => Arc::new(Expression::Reference {
                    identifier: identifier.clone(),
                }),
            },
            |lhs, rhs| {
                Arc::new(Expression::Access {
                    span: rhs.span().with_start(lhs.start()),
                    lhs,
                    rhs,
                })
            },
        )(input)?;

        Ok((input, expression))
    }

    context("expression", inner)(input)
}

fn type_(input: Span) -> Result<Arc<Type>> {
    // Eliminate direct left recursion.
    fn inner(input: Span) -> Result<Arc<Type>> {
        let (input, lhs): (Span, Arc<Type>) = alt((
            into_box(in_braces(
                cut(separated(separated_list0(char(','), separated(identifier)))),
                "expected type entries",
            )),
            into_box(identifier),
        ))(input)?;

        match opt(preceded(separated(tag("->")), type_))(input)? {
            (input, Some(rhs)) => Ok((input, Arc::new(Type::Arrow { lhs, rhs }))),
            (input, None) => Ok((input, lhs)),
        }
    }

    context("type", inner)(input)
}

fn typedef(input: Span) -> Result<Option<Typedef>> {
    context(
        "typedef",
        map(
            with_semicolon(
                tag("type"),
                cut(separated_pair(
                    separated(identifier),
                    cut(char('=')),
                    separated(type_),
                )),
                "expected edge",
            ),
            |(tag_span, res)| res.map(|(ident, tpe)| (tag_span, ident, tpe).into()),
        ),
    )(input)
}

fn value(input: Span) -> Result<Arc<Value>> {
    context(
        "value",
        alt((
            into_box(in_braces(
                cut(separated(separated_list0(
                    char(','),
                    separated(value_entry),
                ))),
                "expected value entries",
            )),
            into_box(identifier),
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

fn variable(input: Span) -> Result<Option<Variable>> {
    context(
        "variable",
        map(
            with_semicolon(
                tag("var"),
                cut(tuple((
                    separated(identifier),
                    delimited(cut(char(':')), separated(type_), cut(char('='))),
                    separated(value),
                ))),
                "expected variable",
            ),
            |(tag_span, res)| res.map(|(ident, tpe, value)| (tag_span, ident, tpe, value).into()),
        ),
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
                    Pragma::new(span, PragmaKind::$constructor, edge_name)
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
                    (Some(x), _, _, _, _) => x
                        .into_iter()
                        .for_each(|x| game.stats.push(Stat::Constant(x))),
                    (_, Some(x), _, _, _) => x
                        .into_iter()
                        .for_each(|x| game.stats.push(Stat::Typedef(x))),
                    (_, _, Some(x), _, _) => x
                        .into_iter()
                        .for_each(|x| game.stats.push(Stat::Variable(x))),
                    (_, _, _, Some(x), _) => {
                        x.into_iter().for_each(|x| game.stats.push(Stat::Edge(x)))
                    }
                    (_, _, _, _, Some(x)) => game.stats.push(Stat::Pragma(x)),
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
    let errors = errors.extra.0.borrow();
    if !errors.is_empty() {
        eprintln!("Errors:");
        for error in errors.iter() {
            eprintln!(" {} {}", error.0, error.1);
        }
    }
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

fn into_box<'a, O1, O2: From<O1>>(
    inner: impl FnMut(Span<'a>) -> Result<'a, O1>,
) -> impl FnMut(Span<'a>) -> Result<'a, Arc<O2>> {
    map(into(inner), Arc::new)
}

// ERROR HANDLIG

impl<'a> State<'a> {
    pub fn report_error(&self, error: Error) {
        self.0.borrow_mut().push(error);
    }
}

fn expect<'a, F, E, T>(mut parser: F, error_msg: E) -> impl FnMut(Span<'a>) -> Result<Option<T>>
where
    F: FnMut(Span<'a>) -> Result<T>,
    E: ToString,
{
    move |input| match parser(input) {
        Ok((remaining, out)) => Ok((remaining, Some(out))),
        Err(nom::Err::Error(input)) | Err(nom::Err::Failure(input)) => {
            let err = Error(Position::from(&input.input), error_msg.to_string());
            input.input.extra.report_error(err);
            Ok((input.input, None))
        }
        Err(err) => Err(err),
    }
}

fn in_paren<'a, F, T, E>(parser: F, error_msg: E) -> impl FnMut(Span<'a>) -> Result<Option<T>>
where
    F: FnMut(Span<'a>) -> Result<T>,
    E: ToString,
{
    delimited(
        tag("("),
        expect(parser, error_msg),
        expect(tag(")"), "missing `)`"),
    )
}

fn in_braces<'a, F, T, E>(parser: F, error_msg: E) -> impl FnMut(Span<'a>) -> Result<Option<T>>
where
    F: FnMut(Span<'a>) -> Result<T>,
    E: ToString,
{
    delimited(
        tag("{"),
        expect(parser, error_msg),
        expect(tag("}"), "missing `}`"),
    )
}

fn in_brackets<'a, F, T, E>(parser: F, error_msg: E) -> impl FnMut(Span<'a>) -> Result<Option<T>>
where
    F: FnMut(Span<'a>) -> Result<T>,
    E: ToString,
{
    delimited(
        tag("["),
        expect(parser, error_msg),
        expect(tag("]"), "missing `]`"),
    )
}

fn with_semicolon<'a, F, G, T, U, E>(
    first: G,
    parser: F,
    error_msg: E,
) -> impl FnMut(Span<'a>) -> Result<(U, Option<T>)>
where
    G: FnMut(Span<'a>) -> Result<U>,
    F: FnMut(Span<'a>) -> Result<T>,
    E: ToString,
{
    terminated(
        tuple((first, expect(parser, error_msg))),
        expect(cut(tag(";")), "missing `;`"),
    )
}
