use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while1};
use nom::character::complete::{multispace0, multispace1};
use nom::error::Error;
use nom::multi::fold_many0;
use nom::sequence::delimited;
use nom::{IResult, Parser};

pub type Result<'a, T> = IResult<&'a str, T, Error<&'a str>>;

pub fn comment(input: &str) -> Result<'_, &str> {
    delimited(tag(";"), alt((is_not("\n\r"), tag(""))), multispace0).parse(input)
}

pub fn comments_and_whitespaces(input: &str) -> Result<'_, ()> {
    fold_many0(alt((comment, multispace1)), || (), |(), _| ()).parse(input)
}

pub fn in_parens<'a, O>(
    inner: impl Parser<&'a str, Output = O, Error = Error<&'a str>>,
) -> impl Parser<&'a str, Output = O, Error = Error<&'a str>> {
    delimited(tag("("), inner, tag(")"))
}

pub fn separated<'a, O>(
    inner: impl Parser<&'a str, Output = O, Error = Error<&'a str>>,
) -> impl Parser<&'a str, Output = O, Error = Error<&'a str>> {
    delimited(comments_and_whitespaces, inner, comments_and_whitespaces)
}

pub fn symbol(input: &str) -> Result<'_, &str> {
    const EXTRAS: &str = "+-<>_";
    take_while1(|c: char| c.is_alphanumeric() || EXTRAS.contains(c)).parse(input)
}
