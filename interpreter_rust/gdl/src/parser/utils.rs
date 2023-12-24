use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::multispace0;
use nom::sequence::delimited;
use nom::IResult;

pub type Result<'a, T> = IResult<&'a str, T>;

pub fn in_parens<'a, T>(
    parser: impl FnMut(&'a str) -> Result<T>,
) -> impl FnMut(&'a str) -> Result<T> {
    delimited(tag("("), parser, tag(")"))
}

pub fn separated<'a, T>(
    parser: impl FnMut(&'a str) -> Result<T>,
) -> impl FnMut(&'a str) -> Result<T> {
    delimited(multispace0, parser, multispace0)
}

pub fn symbol(input: &str) -> Result<&str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}
