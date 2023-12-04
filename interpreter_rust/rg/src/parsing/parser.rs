use super::error::Error;
use super::parser_utils::*;
use crate::ast::*;
use crate::position::{Position, Positioned, Span};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, into, map, opt, success};
use nom::error::context;
use nom::multi::{fold_many0, many0, many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use nom_locate::LocatedSpan;
use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct State<'a>(&'a RefCell<Vec<Error>>);

impl<'a> State<'a> {
    pub fn report_error(&self, error: Error) {
        self.0.borrow_mut().push(error);
    }
}

pub type Input<'a> = LocatedSpan<&'a str, State<'a>>;
pub type Result<'a, T> = IResult<Input<'a>, T>;

fn constant(input: Input) -> Result<Option<Constant<Identifier>>> {
    with_semicolon(
        tag("const"),
        expect(
            tuple((
                preceded_opt_id("const"),
                terminated(preceded_type_, expect_preceded_tag("=")),
                value,
            )),
            "`const <identifier> : <type> = <value>;`",
        ),
    )(input)
}

fn preceded_type_(input: Input) -> Result<Arc<Type<Identifier>>> {
    preceded(expect_preceded_tag(":"), type_)(input)
}

fn edge(input: Input) -> Result<Option<Edge<Identifier>>> {
    with_semicolon(
        terminated(preceded_whitespace(edge_name), expect_preceded_tag(",")),
        expect(
            tuple((
                terminated(
                    preceded_whitespace(expect_edge_name),
                    expect_preceded_tag(":"),
                ),
                edge_label,
            )),
            "`<edge_name>, <edge_name> : <edge_label>;",
        ),
    )(input)
}

fn edge_label(input: Input) -> Result<EdgeLabel<Identifier>> {
    alt((
        tag_label,
        compare_or_assign_label,
        reachability_label,
        expr_label,
        success::<_, _, _>(EdgeLabel::Skip {
            span: Span::at(&input),
        }),
    ))(input)
}

fn tag_label(input: Input) -> Result<EdgeLabel<Identifier>> {
    into(preceded(
        preceded_whitespace(char('$')),
        cut(preceded_opt_id("edge_label")),
    ))(input)
}

fn reachability_label(input: Input) -> Result<EdgeLabel<Identifier>> {
    into(tuple((
        preceded_whitespace(alt((tag("!"), tag("?")))),
        cut(terminated(
            preceded_whitespace(expect_edge_name),
            expect_preceded_tag("->"),
        )),
        preceded_whitespace(expect_edge_name),
    )))(input)
}

fn compare_or_assign_label(input: Input) -> Result<EdgeLabel<Identifier>> {
    let error_pos = Span::at(&input);
    let (input, (lhs, sep, rhs)) = tuple((
        opt(expression),
        preceded_whitespace(alt((tag("=="), tag("!="), tag("=")))),
        expect_expression,
    ))(input)?;
    let separator = *sep.fragment();
    if let Some(lhs) = lhs {
        Ok((input, (lhs, separator, rhs).into()))
    } else {
        let err = Error::parser_error(error_pos, "expected: expression".to_string());
        input.extra.report_error(err);
        let lhs = arc_expression(Identifier::none(error_pos).into());
        Ok((input, (lhs, separator, rhs).into()))
    }
}

// Additional parser for cases like `foo, bar: abc^`
// It always returns an error or fails, so it's used only in LSP
fn expr_label(input: Input) -> Result<EdgeLabel<Identifier>> {
    let (input, lhs) = preceded_whitespace(expression)(input)?;
    let error_pos = Span::from(&input).focus_start();
    let err = Error::parser_error(error_pos, "expected: `=`, `==` or `!=`".to_string());
    input.extra.report_error(err);
    let (input, rhs) = expect_expression(input)?;
    Ok((input, (lhs, rhs).into()))
}

fn edge_name(input: Input) -> Result<EdgeName<Identifier>> {
    into(many1(preceded_whitespace(edge_name_part)))(input)
}

fn expect_edge_name(input: Input) -> Result<EdgeName<Identifier>> {
    let (input, first) = expect(edge_name_part, "edge name")(input)?;
    if let Some(name) = first {
        let (input, rest) = many0(preceded_whitespace(edge_name_part))(input)?;
        let mut parts = vec![name];
        parts.extend(rest);
        Ok((input, parts.into()))
    } else {
        let identifier = Identifier::none(Span::at(&input));
        Ok((input, vec![EdgeNamePart::Literal { identifier }].into()))
    }
}

fn edge_name_part(input: Input) -> Result<EdgeNamePart<Identifier>> {
    alt((
        into(tuple((
            tag("("),
            cut(preceded_opt_id("edge_name_part")),
            preceded_type_,
            preceded_tag(")"),
        ))),
        into(identifier),
    ))(input)
}

fn expect_expression(input: Input) -> Result<Arc<Expression<Identifier>>> {
    let start = Position::from(&input);
    expect(expression, "expression")(input).map(|(input, res)| {
        if let Some(res) = res {
            (input, res)
        } else {
            let span = start.with_end((&input).into());
            (input, arc_expression(Identifier::none(span).into()))
        }
    })
}

fn expression(input: Input) -> Result<Arc<Expression<Identifier>>> {
    let (input, identifier) = preceded_whitespace(identifier)(input)?;
    let (input, maybe_cast) = opt(preceded_whitespace(preceded(
        tag("("),
        pair(cut(expect_expression), preceded_tag(")")),
    )))(input)?;
    let (input, expression) = fold_many0(
        preceded(tag("["), pair(cut(expect_expression), preceded_tag("]"))),
        || match maybe_cast.clone() {
            Some((rhs, end)) => {
                let span = identifier.span().with_end((&end).into());
                let lhs = Arc::new(identifier.clone().into());
                Arc::new(Expression::Cast { span, lhs, rhs })
            }
            None => arc_expression(identifier.clone().into()),
        },
        |lhs, (rhs, end)| {
            let span = lhs.span().with_end((&end).into());
            Arc::new(Expression::Access { span, lhs, rhs })
        },
    )(input)?;

    Ok((input, expression))
}

fn type_(input: Input) -> Result<Arc<Type<Identifier>>> {
    let (input, lhs): (Input, Arc<Type<Identifier>>) = alt((
        into_arc(tuple((
            preceded_tag("{"),
            preceded_whitespace(cut(separated_list0(
                preceded_whitespace(char(',')),
                preceded_opt_id("type_member"),
            ))),
            preceded_tag("}"),
        ))),
        into_arc(preceded_opt_id("type")),
    ))(input)?;

    match opt(preceded(preceded_tag("->"), type_))(input)? {
        (input, Some(rhs)) => Ok((input, Arc::new(Type::Arrow { lhs, rhs }))),
        (input, None) => Ok((input, lhs)),
    }
}

fn typedef(input: Input) -> Result<Option<Typedef<Identifier>>> {
    with_semicolon(
        tag("type"),
        expect(
            separated_pair(preceded_opt_id("typedef"), expect_preceded_tag("="), type_),
            "`type <identifier> = <type>;`",
        ),
    )(input)
}

fn value(input: Input) -> Result<Arc<Value<Identifier>>> {
    alt((value_entries, into_arc(preceded_opt_id("value"))))(input)
}

fn value_entries(input: Input) -> Result<Arc<Value<Identifier>>> {
    alt((
        into_arc(tuple((preceded_tag("{"), preceded_tag("}")))),
        into_arc(tuple((
            preceded_tag("{"),
            cut(separated_list0(
                preceded_whitespace(char(',')),
                expect(preceded_whitespace(value_entry), "value entry"),
            )),
            preceded_tag("}"),
        ))),
    ))(input)
}

fn value_entry(input: Input) -> Result<ValueEntry<Identifier>> {
    let (input, (identifier, value)) = separated_pair(
        opt(identifier),
        preceded_whitespace(char(':')),
        expect(value, "value"),
    )(input)?;
    let (span, value) = match value {
        Some(value) => {
            let span = identifier.as_ref().map_or(value.as_ref().span(), |id| {
                id.span().with_end(value.as_ref().span().end)
            });
            (span, value)
        }
        None => {
            let span = identifier.as_ref().map_or(Span::none(), |id| id.span());
            (span, Arc::new(Identifier::none(Span::none()).into()))
        }
    };
    Ok((input, (span, identifier, value).into()))
}

fn variable(input: Input) -> Result<Option<Variable<Identifier>>> {
    with_semicolon(
        tag("var"),
        expect(
            tuple((
                preceded_opt_id("variable"),
                terminated(preceded_type_, expect_preceded_tag("=")),
                value,
            )),
            "`var <identifier> : <type> = <value>;`",
        ),
    )(input)
}

fn pragma(input: Input) -> Result<Pragma<Identifier>> {
    macro_rules! edge_name {
        ($tag:literal, $constructor:ident) => {
            map(
                tuple((
                    tag("@"),
                    preceded_tag($tag),
                    cut(preceded_whitespace(expect_edge_name)),
                    expect_preceded_tag(";"),
                )),
                |(start, _, edge_name, _)| {
                    let span = edge_name.end().with_start((&start).into());
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

pub fn game(input: Input) -> Result<Game<Identifier>> {
    context(
        "game",
        fold_many0(
            delimited(
                comments_and_whitespaces,
                alt((
                    map(constant, |x| (x, None, None, None, None)),
                    map(typedef, |x| (None, x, None, None, None)),
                    map(variable, |x| (None, None, x, None, None)),
                    map(edge, |x| (None, None, None, x, None)),
                    map(pragma, |x| (None, None, None, None, Some(x))),
                    map(parse_error_line, |_| (None, None, None, None, None)),
                )),
                comments_and_whitespaces,
            ),
            Game::default,
            |mut game, declaration| {
                match declaration {
                    (Some(constant), _, _, _, _) => game.constants.push(constant),
                    (_, Some(typedef), _, _, _) => game.typedefs.push(typedef),
                    (_, _, Some(variable), _, _) => game.variables.push(variable),
                    (_, _, _, Some(edge), _) => game.edges.push(edge),
                    (_, _, _, _, Some(pragma)) => game.pragmas.push(pragma),
                    _ => (),
                }
                game
            },
        ),
    )(input)
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
            "Failed to parse:\n{}\nErrors:\n{}",
            input,
            errors
                .iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let game = game.to_string();
        let game_str = game.strip_suffix('\n').unwrap();
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
