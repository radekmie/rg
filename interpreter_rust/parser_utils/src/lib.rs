use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::multispace1;
use nom::combinator::{cut, eof, map};
use nom::error::{ParseError, VerboseError};
use nom::multi::fold_many0;
use nom::sequence::delimited;
use nom::IResult;
use std::rc::Rc;

pub type Result<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

macro_rules! delimited {
    ($name:ident, $prefix:expr, $suffix:expr) => {
        pub fn $name<'a, O, E: ParseError<&'a str>>(
            inner: impl FnMut(&'a str) -> IResult<&'a str, O, E>,
        ) -> impl FnMut(&'a str) -> IResult<&'a str, O, E> {
            delimited($prefix, inner, $suffix)
        }
    };
}

delimited!(in_braces, tag("{"), tag("}"));
delimited!(in_brackets, tag("["), tag("]"));
delimited!(in_parens, tag("("), tag(")"));
delimited!(
    separated,
    comments_and_whitespaces,
    comments_and_whitespaces
);

/// ```
/// # use nom::IResult;
/// # use parser_utils::comment;
/// fn parser(input: &str) -> IResult<&str, &str> {
///     comment(input)
/// }
///
/// assert_eq!(parser("//"), Ok(("", "")));
/// assert_eq!(parser("//\n"), Ok(("", "")));
/// assert_eq!(parser("//a"), Ok(("", "a")));
/// assert_eq!(parser("//b\n"), Ok(("", "b")));
/// assert_eq!(parser("//c\n\n"), Ok(("\n", "c")));
/// ```
pub fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    delimited(
        tag("//"),
        cut(take_while(|c| c != '\n')),
        alt((eof, tag("\n"))),
    )(input)
}

pub fn comments_and_whitespaces<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, (), E> {
    fold_many0(alt((comment, multispace1)), || (), |_, _| ())(input)
}

pub fn map_into_rc<'a, O1, O2: From<O1>, E: ParseError<&'a str>>(
    inner: impl FnMut(&'a str) -> IResult<&'a str, O1, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Rc<O2>, E> {
    map(inner, |x| Rc::new(From::from(x)))
}
