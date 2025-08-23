use crate::position::Span;
use crate::ParserError;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_while, take_while1};
use nom::character::complete::{anychar, char, digit1, multispace1};
use nom::combinator::{cut, into, map, map_res, opt, verify};
use nom::error::Error;
use nom::multi::{fold_many0, fold_many1, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded};
use nom::{IResult, Parser};
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ParserState<'a>(pub &'a RefCell<Vec<ParserError>>);

impl ParserState<'_> {
    pub fn report_error(&self, error: ParserError) {
        self.0.borrow_mut().push(error);
    }
}

pub type Input<'a> = LocatedSpan<&'a str, ParserState<'a>>;
pub type Result<'a, T> = IResult<Input<'a>, T, Error<Input<'a>>>;

pub fn parse_error_line(input: Input) -> Result<()> {
    let error_pos = Span::at(&input);
    let (input, unexpected) = anychar(input)?;
    let err = ParserError::new(error_pos, format!("unexpected character: `{unexpected}`"));
    input.extra.report_error(err);
    let (input, _) = take_while(|c| c != '\n').parse(input)?;
    Ok((input, ()))
}

pub fn with_semicolon<'a, O1, O2, O3: From<(O1, O2, Span)>>(
    mut first: impl Parser<Input<'a>, Output = O1, Error = Error<Input<'a>>>,
    mut second: impl Parser<Input<'a>, Output = Option<O2>, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = Option<O3>, Error = Error<Input<'a>>> {
    move |input| {
        let (input, first) = first.parse(input)?;
        let (input, second) = second.parse(input)?;
        let semicolon_pos = Span::from(&input).focus_start();
        let (input, end) = preceded_whitespace(opt(tag(";"))).parse(input)?;
        if end.is_none() && second.is_some() {
            let err = ParserError::new(semicolon_pos, "expected `;`".to_string());
            input.extra.report_error(err);
        }
        let end_pos = end.map_or_else(|| Span::at(&input), |end| Span::from(&end).focus_end());
        Ok((input, second.map(|second| (first, second, end_pos).into())))
    }
}

pub fn expect<'a, T, E: Display>(
    mut parser: impl Parser<Input<'a>, Output = T, Error = Error<Input<'a>>>,
    error_msg: E,
) -> impl Parser<Input<'a>, Output = Option<T>, Error = Error<Input<'a>>> {
    move |input| {
        let error_pos = Span::at(&input);
        match parser.parse(input) {
            Ok((remaining, out)) => Ok((remaining, Some(out))),
            Err(nom::Err::Error(input) | nom::Err::Failure(input)) => {
                let err = ParserError::new(error_pos, format!("expected: {error_msg}"));
                input.input.extra.report_error(err);
                Ok((input.input, None))
            }
            Err(err) => Err(err),
        }
    }
}

pub fn comment(input: Input) -> Result<Input> {
    preceded(tag("//"), cut(take_till(|c| c == '\n'))).parse(input)
}

pub fn comments_and_whitespaces0(input: Input) -> Result<()> {
    fold_many0(alt((comment, multispace1)), || (), |(), _| ()).parse(input)
}

pub fn comments_and_whitespaces1(input: Input) -> Result<()> {
    fold_many1(alt((comment, multispace1)), || (), |(), _| ()).parse(input)
}

pub fn preceded_whitespace<'a, O>(
    inner: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>> {
    preceded(comments_and_whitespaces0, inner)
}

pub fn preceded_tag<'a>(
    str: &'a str,
) -> impl Parser<Input<'a>, Output = Input<'a>, Error = Error<Input<'a>>> {
    preceded_whitespace(tag(str))
}

pub fn expect_preceded_tag<'a>(
    str: &'a str,
) -> impl Parser<Input<'a>, Output = Option<Input<'a>>, Error = Error<Input<'a>>> {
    expect(preceded_tag(str), format!("`{str}`"))
}

pub fn into_arc<'a, O1, O2: From<O1>>(
    inner: impl Parser<Input<'a>, Output = O1, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = Arc<O2>, Error = Error<Input<'a>>> {
    map(into(inner), Arc::new)
}

pub fn identifier_(input: Input) -> Result<Input> {
    static KEYWORDS: [&str; 3] = ["const", "type", "var"];
    verify(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |identifier: &Input| !KEYWORDS.contains(identifier.fragment()),
    )
    .parse(input)
}

pub fn integer<T: FromStr>(input: Input) -> Result<T> {
    map_res(digit1, |digits: Input| digits.parse()).parse(input)
}

pub fn ww<'a, O>(
    inner: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>> {
    delimited(comments_and_whitespaces0, inner, comments_and_whitespaces0)
}

pub fn ww_tag<'a>(
    str: &'a str,
) -> impl Parser<Input<'a>, Output = Input<'a>, Error = Error<Input<'a>>> {
    ww(tag(str))
}

pub fn ww_char<'a>(c: char) -> impl Parser<Input<'a>, Output = char, Error = Error<Input<'a>>> {
    ww(char(c))
}

pub fn in_braces<'a, O>(
    inner: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>> {
    delimited(ww_char('{'), inner, ww_char('}'))
}

pub fn in_parens<'a, O>(
    inner: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>> {
    delimited(ww_char('('), inner, ww_char(')'))
}

pub fn in_brackets<'a, O>(
    inner: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>> {
    delimited(ww_char('['), inner, ww_char(']'))
}

pub fn comma_separated0<'a, O>(
    inner: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = Vec<O>, Error = Error<Input<'a>>> {
    separated_list0(ww_char(','), inner)
}

pub fn comma_separated1<'a, O>(
    inner: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = Vec<O>, Error = Error<Input<'a>>> {
    separated_list1(ww_char(','), inner)
}
