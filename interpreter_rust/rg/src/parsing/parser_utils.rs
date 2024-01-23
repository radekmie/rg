use super::error::Error;
use super::parser::{Input, Result};
use crate::ast::{Expression, Identifier};
use crate::position::{Position, Span};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while, take_while1};
use nom::character::complete::{anychar, digit1, multispace1};
use nom::combinator::{cut, eof, into, map, map_res, opt, verify};
use nom::multi::fold_many0;
use nom::sequence::{delimited, preceded};
use std::fmt::Display;
use std::sync::Arc;

pub fn parse_error_line(input: Input) -> Result<()> {
    let error_pos = Span::at(&input);
    let (input, unexpected) = anychar(input)?;
    let error_msg = format!("unexpected character: `{unexpected}`");
    let err = Error::parser_error(error_pos, error_msg);
    input.extra.report_error(err);
    let (input, _) = take_while(|c| c != '\n')(input)?;
    Ok((input, ()))
}

pub fn with_semicolon<'a, O1, O2, O3: From<(O1, O2, Span)>>(
    mut first: impl FnMut(Input<'a>) -> Result<O1>,
    mut second: impl FnMut(Input<'a>) -> Result<Option<O2>>,
) -> impl FnMut(Input<'a>) -> Result<Option<O3>> {
    move |input| {
        let (input, first) = first(input)?;
        let (input, second) = second(input)?;
        let semicolon_pos = Span::from(&input).focus_start();
        let (input, end) = preceded_whitespace(opt(tag(";")))(input)?;
        if end.is_none() && second.is_some() {
            let err = Error::parser_error(semicolon_pos, "expected `;`".to_string());
            input.extra.report_error(err);
        }
        let end_pos = end.map_or_else(|| Span::at(&input), |end| Span::from(&end).focus_end());
        Ok((input, second.map(|second| (first, second, end_pos).into())))
    }
}

pub fn expect<'a, F, E, T>(
    mut parser: F,
    error_msg: E,
) -> impl FnMut(Input<'a>) -> Result<Option<T>>
where
    F: FnMut(Input<'a>) -> Result<T>,
    E: Display,
{
    move |input| {
        let error_pos = Span::at(&input);
        match parser(input) {
            Ok((remaining, out)) => Ok((remaining, Some(out))),
            Err(nom::Err::Error(input) | nom::Err::Failure(input)) => {
                let err = Error::parser_error(error_pos, format!("expected: {error_msg}"));
                input.input.extra.report_error(err);
                Ok((input.input, None))
            }
            Err(err) => Err(err),
        }
    }
}

pub fn comment(input: Input) -> Result<Input> {
    delimited(tag("//"), cut(take_until("\n")), alt((eof, tag("\n"))))(input)
}

pub fn comments_and_whitespaces(input: Input) -> Result<()> {
    fold_many0(alt((comment, multispace1)), || (), |(), _| ())(input)
}

pub fn preceded_whitespace<'a, O>(
    inner: impl FnMut(Input<'a>) -> Result<O>,
) -> impl FnMut(Input<'a>) -> Result<O> {
    preceded(comments_and_whitespaces, inner)
}

pub fn preceded_tag<'a>(str: &'a str) -> impl FnMut(Input<'a>) -> Result<Input> {
    preceded_whitespace(tag(str))
}

pub fn expect_preceded_tag<'a>(str: &'a str) -> impl FnMut(Input<'a>) -> Result<Option<Input>> {
    expect(preceded_tag(str), format!("`{str}`"))
}

pub fn into_arc<'a, O1, O2: From<O1>>(
    inner: impl FnMut(Input<'a>) -> Result<'a, O1>,
) -> impl FnMut(Input<'a>) -> Result<'a, Arc<O2>> {
    map(into(inner), Arc::new)
}

pub fn arc_expression(expression: Expression<Identifier>) -> Arc<Expression<Identifier>> {
    Arc::new(expression)
}

pub fn identifier_(input: Input) -> Result<Input> {
    static KEYWORDS: [&str; 4] = ["any", "const", "type", "var"];
    verify(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |identifier: &Input| !KEYWORDS.contains(identifier.fragment()),
    )(input)
}

pub fn identifier(input: Input) -> Result<Identifier> {
    map(identifier_, |identifier| {
        let span: Span = Span::from(&identifier);
        Identifier::new(span, (*identifier.fragment()).to_string())
    })(input)
}

pub fn preceded_opt_id<'a>(context: &'a str) -> impl FnMut(Input<'a>) -> Result<Identifier> {
    move |input| {
        let start = Position::from(&input);
        expect(
            preceded_whitespace(identifier),
            format!("{context}: identifier"),
        )(input)
        .map(|(input, res)| {
            if let Some(res) = res {
                (input, res)
            } else {
                let span = start.with_end((&input).into());
                (input, Identifier::none(span))
            }
        })
    }
}

pub fn integer(input: Input) -> Result<usize> {
    map_res(digit1, |digits: Input| digits.parse())(input)
}
