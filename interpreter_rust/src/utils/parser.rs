use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use nom::combinator::map;
use nom::error::ParseError;
use nom::error::VerboseError;
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
delimited!(ws, multispace0, multispace0);

pub fn map_into<'a, O1, O2: From<O1>, E: ParseError<&'a str>>(
    inner: impl FnMut(&'a str) -> IResult<&'a str, O1, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Rc<O2>, E> {
    map(inner, |x| Rc::new(x.into()))
}
