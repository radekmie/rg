use crate::parser::alt;
use nom::bytes::complete::{tag, take_until, take_while1};
use nom::character::complete::multispace1;
use nom::combinator::cut;
use nom::combinator::eof;
use nom::multi::fold_many0;
use nom::sequence::delimited;
use nom::IResult;

pub type Result<'a, T> = IResult<&'a str, T>;

pub fn comment(input: &str) -> Result<&str> {
    delimited(tag(";"), cut(take_until("\n")), alt((eof, tag("\n"))))(input)
}

pub fn comments_and_whitespaces(input: &str) -> Result<()> {
    fold_many0(alt((comment, multispace1)), || (), |(), _| ())(input)
}

pub fn in_parens<'a, T>(
    parser: impl FnMut(&'a str) -> Result<'a, T>,
) -> impl FnMut(&'a str) -> Result<'a, T> {
    delimited(tag("("), parser, tag(")"))
}

pub fn separated<'a, T>(
    parser: impl FnMut(&'a str) -> Result<'a, T>,
) -> impl FnMut(&'a str) -> Result<'a, T> {
    delimited(comments_and_whitespaces, parser, comments_and_whitespaces)
}

pub fn symbol(input: &str) -> Result<&str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_' || c == '+')(input)
}
