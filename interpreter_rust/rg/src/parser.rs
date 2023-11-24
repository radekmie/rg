use crate::ast::*;
use crate::position::{Span as Position, *};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::{anychar, char, multispace1};
use nom::combinator::{all_consuming, cut, eof, into, map, opt, success, verify};
use nom::error::context;
use nom::multi::{fold_many0, many0, many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::sync::Arc;

use super::error::Error;

#[derive(Clone, Debug)]
pub struct State<'a>(&'a RefCell<Vec<Error>>);

pub type Span<'a> = LocatedSpan<&'a str, State<'a>>;
pub type Result<'a, T> = IResult<Span<'a>, T>;

fn constant(input: Span) -> Result<Option<Constant<Identifier>>> {
    context(
        "constant",
        map(
            with_semicolon(
                tag("const"),
                expect(
                    tuple((
                        preceded_opt_id("const"),
                        terminated(
                            preceded_type_,
                            expect(preceded_whitespace(cut(char('='))), "expected `=`"),
                        ),
                        value,
                    )),
                    "syntax error: expected `const <identifier> : <type> = <value>;`",
                ),
            ),
            |(tag_span, res, end)| {
                res.map(|(identifier, type_, value)| {
                    (tag_span, identifier, type_, value, end).into()
                })
            },
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
}

fn preceded_opt_id<'a>(context: &'a str) -> impl FnMut(Span<'a>) -> Result<Identifier> {
    move |input| {
        let start = Position::from(&input).start;
        expect(
            preceded_whitespace(identifier),
            format!("{}: expected identifier", context),
        )(input)
        .map(|(input, res)| {
            if let Some(res) = res {
                (input, res)
            } else {
                let span = Position::from(&input).focus_start();
                let span = span.with_start(start);
                (input, Identifier::none(span))
            }
        })
    }
}

fn preceded_type_(input: Span) -> Result<Arc<Type<Identifier>>> {
    expect(preceded_whitespace(char(':')), "expected `:`")(input).and_then(|(input, _)| {
        let (input, res) = type_(input)?;
        Ok((input, res))
    })
}

fn edge(input: Span) -> Result<Option<Edge<Identifier>>> {
    context(
        "edge",
        map(
            with_semicolon(
                terminated(
                    preceded_whitespace(edge_name),
                    expect(preceded_whitespace(char(',')), "expected `,`"),
                ),
                expect(
                    tuple((
                        terminated(
                            preceded_whitespace(expect_edge_name),
                            expect(preceded_whitespace(cut(char(':'))), "expected `:`"),
                        ),
                        edge_label,
                    )),
                    "syntax error: expected `<edge_name>, <edge_name> : <edge_label>;",
                ),
            ),
            |(lhs, res, end)| res.map(|(rhs, label)| (lhs, rhs, label, end).into()),
        ),
    )(input)
}

fn edge_label(input: Span) -> Result<EdgeLabel<Identifier>> {
    context(
        "edge_label",
        alt((
            into(preceded(
                preceded_whitespace(char('$')),
                cut(preceded_opt_id("edge_label")),
            )),
            compare_label,
            into(tuple((
                preceded_whitespace(alt((tag("!"), tag("?")))),
                cut(terminated(
                    preceded_whitespace(expect_edge_name),
                    expect(preceded_whitespace(tag("->")), "expected `->`"),
                )),
                preceded_whitespace(expect_edge_name),
            ))),
            assign_label,
            expr_label,
            success::<_, _, _>(EdgeLabel::Skip {
                span: Position::from(&input),
            }),
        )),
    )(input)
}

fn edge_name(input: Span) -> Result<EdgeName<Identifier>> {
    context(
        "edge_name",
        into(many1(preceded_whitespace(edge_name_part))),
    )(input)
}

fn expect_edge_name(input: Span) -> Result<EdgeName<Identifier>> {
    let (input, first) = expect(edge_name_part, "expected edge name")(input)?;
    match first {
        Some(name) => {
            let (input, rest) = many0(preceded_whitespace(edge_name_part))(input)?;
            let mut parts = vec![name];
            parts.extend(rest);
            Ok((input, parts.into()))
        }
        None => {
            let span = Position::from(&input).focus_start();
            Ok((
                input,
                vec![EdgeNamePart::Literal {
                    identifier: Identifier::none(span),
                }]
                .into(),
            ))
        }
    }
}

fn edge_name_part(input: Span) -> Result<EdgeNamePart<Identifier>> {
    context(
        "edge_name_part",
        alt((
            into(tuple((
                tag("("),
                cut(preceded_opt_id("edge_name_part")),
                preceded_type_,
                preceded_whitespace(tag(")")),
            ))),
            into(identifier),
        )),
    )(input)
}

fn expect_expression(input: Span) -> Result<Arc<Expression<Identifier>>> {
    let start = Position::from(&input).start;
    expect(expression, "expected expression")(input).map(|(input, res)| {
        if let Some(res) = res {
            (input, res)
        } else {
            let span = Position::from(&input).focus_start();
            let span = span.with_start(start);
            (
                input,
                Arc::new(Expression::Reference {
                    identifier: Identifier::none(span),
                }),
            )
        }
    })
}

fn expression(input: Span) -> Result<Arc<Expression<Identifier>>> {
    // Eliminate direct left recursion.
    fn inner(input: Span) -> Result<Arc<Expression<Identifier>>> {
        let (input, identifier) = preceded_whitespace(identifier)(input)?;
        let (input, maybe_cast) = opt(preceded_whitespace(preceded(
            tag("("),
            pair(cut(expect_expression), preceded_whitespace(tag(")"))),
        )))(input)?;
        let (input, expression) = fold_many0(
            preceded(
                tag("["),
                pair(cut(expect_expression), preceded_whitespace(tag("]"))),
            ),
            || match maybe_cast.clone() {
                Some((rhs, end)) => {
                    let span = Position::from(end).with_start(identifier.span().start);
                    let lhs = Arc::new(Type::TypeReference {
                        identifier: identifier.clone(),
                    });
                    Arc::new(Expression::Cast { span, lhs, rhs })
                }
                None => Arc::new(Expression::Reference {
                    identifier: identifier.clone(),
                }),
            },
            |lhs, (rhs, end)| {
                let span = Position::from(end).with_start(lhs.start());
                Arc::new(Expression::Access { span, lhs, rhs })
            },
        )(input)?;

        Ok((input, expression))
    }

    context("expression", inner)(input)
}

fn type_(input: Span) -> Result<Arc<Type<Identifier>>> {
    // Eliminate direct left recursion.
    fn inner(input: Span) -> Result<Arc<Type<Identifier>>> {
        let (input, lhs): (Span, Arc<Type<Identifier>>) = alt((
            into_arc(tuple((
                preceded_whitespace(tag("{")),
                preceded_whitespace(cut(separated_list0(
                    preceded_whitespace(char(',')),
                    preceded_opt_id("type_member"),
                ))),
                preceded_whitespace(tag("}")),
            ))),
            into_arc(preceded_opt_id("type")),
        ))(input)?;

        match opt(preceded(preceded_whitespace(tag("->")), type_))(input)? {
            (input, Some(rhs)) => Ok((input, Arc::new(Type::Arrow { lhs, rhs }))),
            (input, None) => Ok((input, lhs)),
        }
    }

    context("type", inner)(input)
}

fn typedef(input: Span) -> Result<Option<Typedef<Identifier>>> {
    context(
        "typedef",
        map(
            with_semicolon(
                tag("type"),
                expect(
                    separated_pair(
                        preceded_opt_id("typedef"),
                        expect(preceded_whitespace(cut(char('='))), "expected `=`"),
                        type_,
                    ),
                    "syntax error: expected `type <identifier> = <type>;`",
                ),
            ),
            |(tag_span, res, end)| res.map(|(ident, tpe)| (tag_span, ident, tpe, end).into()),
        ),
    )(input)
}

fn value(input: Span) -> Result<Arc<Value<Identifier>>> {
    context(
        "value",
        alt((value_entries, into_arc(preceded_opt_id("value")))),
    )(input)
}

fn value_entries(input: Span) -> Result<Arc<Value<Identifier>>> {
    alt((
        into_arc(tuple((
            preceded_whitespace(tag("{")),
            preceded_whitespace(tag("}")),
        ))),
        into_arc(tuple((
            preceded_whitespace(tag("{")),
            cut(separated_list0(
                preceded_whitespace(char(',')),
                expect(preceded_whitespace(value_entry), "expected value entry"),
            )),
            preceded_whitespace(tag("}")),
        ))),
    ))(input)
}

fn value_entry(input: Span) -> Result<ValueEntry<Identifier>> {
    let (input, identifier) = opt(identifier)(input)?;
    let (input, _) = preceded_whitespace(char(':'))(input)?;
    let (input, value) = expect(value, "expected value")(input)?;
    let (span, value) = match value {
        Some(value) => {
            let span = match &identifier {
                Some(identifier) => {
                    Position::new(identifier.span().start, value.as_ref().span().end)
                }
                None => value.as_ref().span().clone(),
            };
            (span, value)
        }
        None => {
            let span = match &identifier {
                Some(identifier) => identifier.span().clone(),
                None => Position::none(),
            };
            (
                span,
                Arc::new(Value::Element {
                    identifier: Identifier::none(Position::none()),
                }),
            )
        }
    };
    Ok((input, ValueEntry::from((span, identifier, value))))
}

fn variable(input: Span) -> Result<Option<Variable<Identifier>>> {
    map(
        with_semicolon(
            tag("var"),
            expect(
                tuple((
                    preceded_opt_id("variable"),
                    terminated(
                        preceded_type_,
                        expect(preceded_whitespace(cut(char('='))), "expected `=`"),
                    ),
                    value,
                )),
                "syntax error: expected `var <identifier> : <type> = <value>;`",
            ),
        ),
        |(tag_span, res, end)| {
            res.map(|(ident, tpe, value)| (tag_span, ident, tpe, value, end).into())
        },
    )(input)
}

fn pragma(input: Span) -> Result<Pragma<Identifier>> {
    macro_rules! edge_name {
        ($tag:literal, $constructor:ident) => {
            map(
                tuple((
                    tag("@"),
                    preceded_whitespace(tag($tag)),
                    cut(preceded_whitespace(expect_edge_name)),
                    expect(preceded_whitespace(cut(tag(";"))), "missing `;`"),
                )),
                |(start, _, edge_name, _)| {
                    let start = Position::from(&start);
                    let span = Position {
                        start: start.start,
                        end: edge_name.end(),
                    };
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

fn parse_error_line(input: Span) -> Result<()> {
    let error_pos = Position::from(&input);
    let (input, unexpected) = anychar(input)?;
    let error_msg = format!("unexpected character: `{}`", unexpected);
    let err = Error::parser_error(error_pos, error_msg.to_string());
    input.extra.report_error(err);
    let (input, _) = take_while(|c| c != '\n')(input)?;
    Ok((input, ()))
}

pub fn game(input: Span) -> Result<Game<Identifier>> {
    context(
        "game",
        fold_many0(
            delimited(
                comments_and_whitespaces,
                alt((
                    map(constant, |x| (Some(x), None, None, None, None)),
                    map(typedef, |x| (None, Some(x), None, None, None)),
                    map(variable, |x| (None, None, Some(x), None, None)),
                    map(edge, |x| (None, None, None, Some(x), None)),
                    map(pragma, |x| (None, None, None, None, Some(x))),
                    map(parse_error_line, |_| (None, None, None, None, None)),
                )),
                comments_and_whitespaces,
            ),
            Game::default,
            |mut game, declaration| {
                match declaration {
                    (Some(constant), _, _, _, _) => constant
                        .into_iter()
                        .for_each(|constant| game.constants.push(constant)),
                    (_, Some(typedef), _, _, _) => typedef
                        .into_iter()
                        .for_each(|typedef| game.typedefs.push(typedef)),
                    (_, _, Some(variable), _, _) => variable
                        .into_iter()
                        .for_each(|variable| game.variables.push(variable)),
                    (_, _, _, Some(edge), _) => {
                        edge.into_iter().for_each(|edge| game.edges.push(edge))
                    }
                    (_, _, _, _, Some(pragma)) => game.pragmas.push(pragma),
                    _ => (),
                }
                game
            },
        ),
    )(input)
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

macro_rules! preceded {
    ($name:ident, $prefix:expr) => {
        pub fn $name<'a, O>(
            inner: impl FnMut(Span<'a>) -> Result<O>,
        ) -> impl FnMut(Span<'a>) -> Result<O> {
            preceded($prefix, inner)
        }
    };
}

preceded!(preceded_whitespace, comments_and_whitespaces);

fn into_arc<'a, O1, O2: From<O1>>(
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
    move |input| {
        let error_pos = Position::from(&input);
        match parser(input) {
            Ok((remaining, out)) => Ok((remaining, Some(out))),
            Err(nom::Err::Error(input)) | Err(nom::Err::Failure(input)) => {
                if error_msg.to_string() == "" {
                    return Ok((input.input, None));
                } else {
                    let err = Error::parser_error(error_pos, error_msg.to_string());
                    input.input.extra.report_error(err);
                    Ok((input.input, None))
                }
            }
            Err(err) => Err(err),
        }
    }
}

fn with_semicolon<'a, F, G, T, U>(
    mut first: G,
    mut parser: F,
) -> impl FnMut(Span<'a>) -> Result<(U, Option<T>, Position)>
where
    G: FnMut(Span<'a>) -> Result<U>,
    F: FnMut(Span<'a>) -> Result<Option<T>>,
{
    move |input| {
        let (input, first) = first(input)?;
        let (input, second) = parser(input)?;
        let (input, end) = preceded_whitespace(opt(tag(";")))(input)?;
        let semicolon_pos = Position::from(&input).focus_start();
        if end.is_none() && second.is_some() {
            let err = Error::parser_error(semicolon_pos, "expected `;`".to_string());
            input.extra.report_error(err);
        }
        let end_pos = end.map_or(semicolon_pos, |end| Position::from(&end).focus_end());
        Ok((input, (first, second, end_pos)))
    }
}

fn compare_label(input: Span) -> Result<EdgeLabel<Identifier>> {
    let error_pos = Position::from(&input).focus_start();
    let (input, maybe_expr) = opt(preceded_whitespace(expression))(input)?;
    let (input, comparison) = preceded_whitespace(map(alt((tag("=="), tag("!="))), |c: Span| {
        *c.fragment() == "!="
    }))(input)?;
    let (input, rhs) = expect_expression(input)?;
    if let Some(lhs) = maybe_expr {
        Ok((input, (lhs, comparison, rhs).into()))
    } else {
        let err = Error::parser_error(error_pos, "expected expression".to_string());
        input.extra.report_error(err);
        let lhs = Arc::new(Expression::Reference {
            identifier: Identifier::none(error_pos),
        });
        Ok((input, (lhs, comparison, rhs).into()))
    }
}

fn assign_label(input: Span) -> Result<EdgeLabel<Identifier>> {
    let error_pos = Position::from(&input).focus_start();
    let (input, maybe_expr) = opt(preceded_whitespace(expression))(input)?;
    let (input, _) = preceded_whitespace(char('='))(input)?;
    let (input, rhs) = expect_expression(input)?;
    if let Some(lhs) = maybe_expr {
        Ok((input, (lhs, rhs).into()))
    } else {
        let err = Error::parser_error(error_pos, "expected expression".to_string());
        input.extra.report_error(err);
        let lhs = Arc::new(Expression::Reference {
            identifier: Identifier::none(error_pos),
        });
        Ok((input, (lhs, rhs).into()))
    }
}

// Additional parser for cases like `foo, bar: abc^`
// It always returns an error or fails, so it's used only in LSP
fn expr_label(input: Span) -> Result<EdgeLabel<Identifier>> {
    let (input, lhs) = preceded_whitespace(expression)(input)?;
    let error_pos = Position::from(&input).focus_start();
    let err = Error::parser_error(error_pos, "expected `=`, `==` or `!=`".to_string());
    input.extra.report_error(err);
    let (input, rhs) = expect_expression(input)?;
    Ok((input, (lhs, rhs).into()))
}

pub fn parse_with_errors(input: &str) -> (Game<Identifier>, Vec<Error>) {
    let errors = RefCell::new(Vec::new());
    let input = nom_locate::LocatedSpan::new_extra(input, State(&errors));
    let (_, game) = all_consuming(game)(input).expect("Parser cannot fail");
    (game, errors.into_inner())
}

pub fn parse_expression(input: &str) -> Arc<Expression<Identifier>> {
    let errors = RefCell::new(Vec::new());
    let input = nom_locate::LocatedSpan::new_extra(input, State(&errors));
    let (_, expression) = all_consuming(expression)(input).expect("Parser cannot fail");
    expression
}

#[cfg(test)]
mod test {
    use super::parse_with_errors;

    fn check_parse(input: &str) {
        let (game, errors) = parse_with_errors(input);
        assert!(
            errors.is_empty(),
            "Failed to parse:\n{}\nErrors: {:#?}",
            input,
            errors
        );
        let game = game.to_string();
        let game_str = game.strip_suffix("\n").unwrap();
        assert!(
            game_str == input,
            "Failed to parse:\n{}\nExpected:\n{}",
            game_str,
            input
        );
    }

    #[test]
    fn typedef() {
        check_parse("type A = B;");
        check_parse("type Foo = { foo, bar, goo };");
        check_parse("type Foo = Bar -> Baz -> Goo;");
    }

    #[test]
    fn variable() {
        check_parse("var foo: Foo = { foo: 1, :bar, :goo };");
        check_parse("var foo: Foo = {  };");
        check_parse("var foo: Foo = { :null };");
        check_parse("var foo: Foo = { :null, :null };");
    }

    #[test]
    fn constant() {
        check_parse("const foo: Foo = { foo: 1, :bar, goo: 3 };");
    }

    #[test]
    fn edge() {
        check_parse("foo, bar: $ F;");
        check_parse("foo, bar: ;");
        check_parse("foo, bar: x == y;");
        check_parse("foo, bar: ? move -> move;");
    }
}
