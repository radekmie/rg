use crate::ast::{
    Constant, Edge, Expression, Game, Label, Node, NodePart, Pragma, PragmaAssignment, PragmaTag,
    Type, Typedef, Value, ValueEntry, Variable,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, into, map, opt, success};
use nom::error::context;
use nom::multi::{fold_many0, many0, many1, separated_list0};
use nom::sequence::{pair, preceded, separated_pair, terminated, tuple};
use std::cell::RefCell;
use std::sync::Arc;
use utils::parser::{
    comments_and_whitespaces, expect, expect_preceded_tag, identifier_, in_brackets, integer,
    into_arc, parse_error_line, preceded_tag, preceded_whitespace, with_semicolon, ww_char, Input,
    Result, State,
};
use utils::position::{Position, Positioned, Span};
use utils::{Error, Identifier};

fn arc_expression(expression: Expression<Identifier>) -> Arc<Expression<Identifier>> {
    Arc::new(expression)
}

fn identifier(input: Input) -> Result<Identifier> {
    map(identifier_, |identifier| {
        let span: Span = Span::from(&identifier);
        Identifier::new(span, (*identifier.fragment()).to_string())
    })(input)
}

fn preceded_opt_id<'a>(context: &'a str) -> impl FnMut(Input<'a>) -> Result<'a, Identifier> {
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
        terminated(preceded_whitespace(node), expect_preceded_tag(",")),
        expect(
            tuple((
                terminated(preceded_whitespace(expect_node), expect_preceded_tag(":")),
                label,
            )),
            "`<node>, <node> : <label>;",
        ),
    )(input)
}

fn label(input: Input) -> Result<Label<Identifier>> {
    alt((
        tag_label,
        compare_or_assign_label,
        reachability_label,
        expr_label,
        success::<_, _, _>(Label::Skip {
            span: Span::at(&input),
        }),
    ))(input)
}

fn tag_label(input: Input) -> Result<Label<Identifier>> {
    into(preceded(
        preceded_whitespace(char('$')),
        cut(preceded_opt_id("label")),
    ))(input)
}

fn reachability_label(input: Input) -> Result<Label<Identifier>> {
    into(tuple((
        preceded_whitespace(alt((tag("!"), tag("?")))),
        cut(terminated(
            preceded_whitespace(expect_node),
            expect_preceded_tag("->"),
        )),
        preceded_whitespace(expect_node),
    )))(input)
}

fn compare_or_assign_label(input: Input) -> Result<Label<Identifier>> {
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
fn expr_label(input: Input) -> Result<Label<Identifier>> {
    let (input, lhs) = preceded_whitespace(expression)(input)?;
    let error_pos = Span::from(&input).focus_start();
    let err = Error::parser_error(error_pos, "expected: `=`, `==` or `!=`".to_string());
    input.extra.report_error(err);
    let (input, rhs) = expect_expression(input)?;
    Ok((input, (lhs, rhs).into()))
}

fn node(input: Input) -> Result<Node<Identifier>> {
    into(pair(identifier, node_bindings))(input)
}

fn expect_node(input: Input) -> Result<Node<Identifier>> {
    let (input, first) = expect(identifier, "edge name")(input)?;
    if let Some(name) = first {
        let (input, rest) = node_bindings(input)?;
        Ok((input, (name, rest).into()))
    } else {
        let identifier = Identifier::none(Span::at(&input));
        Ok((input, identifier.into()))
    }
}

fn node_bindings(input: Input) -> Result<Vec<NodePart<Identifier>>> {
    many0(node_binding)(input)
}

fn node_binding(input: Input) -> Result<NodePart<Identifier>> {
    into(tuple((
        tag("("),
        cut(preceded_opt_id("node_part")),
        preceded_type_,
        preceded_tag(")"),
    )))(input)
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
    let (span, value) = value.map_or_else(
        || {
            let span = identifier
                .as_ref()
                .map_or_else(Span::none, Positioned::span);
            (span, Arc::new(Value::new(Identifier::none(Span::none()))))
        },
        |value| {
            let span = identifier.as_ref().map_or(value.as_ref().span(), |id| {
                id.span().with_end(value.as_ref().span().end)
            });
            (span, value)
        },
    );
    Ok((input, ValueEntry::new(span, identifier, value)))
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

fn pragma(input: Input) -> Result<Option<Pragma<Identifier>>> {
    let pragma = alt((
        map(
            tuple((
                tag("disjointExhaustive"),
                cut(preceded_whitespace(node)),
                preceded_whitespace(tag(":")),
                cut(many1(preceded_whitespace(node))),
                preceded_whitespace(tag(";")),
            )),
            |(tag, node, _, nodes, semicolon)| Pragma::DisjointExhaustive {
                span: Span::from((&tag, &semicolon)),
                node,
                nodes,
            },
        ),
        map(
            tuple((
                tag("disjoint"),
                cut(preceded_whitespace(node)),
                preceded_whitespace(tag(":")),
                cut(many1(preceded_whitespace(node))),
                preceded_whitespace(tag(";")),
            )),
            |(tag, node, _, nodes, semicolon)| Pragma::Disjoint {
                span: Span::from((&tag, &semicolon)),
                node,
                nodes,
            },
        ),
        map(
            tuple((
                tag("repeat"),
                cut(many1(preceded_whitespace(node))),
                preceded_whitespace(tag(":")),
                cut(many0(preceded_whitespace(identifier))),
                preceded_whitespace(tag(";")),
            )),
            |(tag, nodes, _, identifiers, semicolon)| Pragma::Repeat {
                span: Span::from((&tag, &semicolon)),
                nodes,
                identifiers,
            },
        ),
        map(
            tuple((
                tag("simpleApplyExhaustive"),
                cut(preceded_whitespace(node)),
                preceded_whitespace(node),
                in_brackets(separated_list0(ww_char(','), pragma_tag)),
                separated_list0(ww_char(','), pragma_assignment),
                preceded_whitespace(tag(";")),
            )),
            |(tag, lhs, rhs, tags, assignments, semicolon)| Pragma::SimpleApplyExhaustive {
                span: Span::from((&tag, &semicolon)),
                lhs,
                rhs,
                tags,
                assignments,
            },
        ),
        map(
            tuple((
                tag("simpleApply"),
                cut(preceded_whitespace(node)),
                preceded_whitespace(node),
                in_brackets(separated_list0(ww_char(','), pragma_tag)),
                separated_list0(ww_char(','), pragma_assignment),
                preceded_whitespace(tag(";")),
            )),
            |(tag, lhs, rhs, tags, assignments, semicolon)| Pragma::SimpleApply {
                span: Span::from((&tag, &semicolon)),
                lhs,
                rhs,
                tags,
                assignments,
            },
        ),
        map(
            tuple((
                tag("tagIndex"),
                cut(many1(preceded_whitespace(node))),
                preceded_whitespace(tag(":")),
                preceded_whitespace(integer),
                preceded_whitespace(tag(";")),
            )),
            |(tag, nodes, _, index, semicolon)| Pragma::TagIndex {
                span: Span::from((&tag, &semicolon)),
                nodes,
                index,
            },
        ),
        map(
            tuple((
                tag("tagMaxIndex"),
                cut(many1(preceded_whitespace(node))),
                preceded_whitespace(tag(":")),
                preceded_whitespace(integer),
                preceded_whitespace(tag(";")),
            )),
            |(tag, nodes, _, index, semicolon)| Pragma::TagMaxIndex {
                span: Span::from((&tag, &semicolon)),
                nodes,
                index,
            },
        ),
        map(
            tuple((
                tag("unique"),
                cut(many1(preceded_whitespace(node))),
                preceded_whitespace(tag(";")),
            )),
            |(tag, nodes, semicolon)| Pragma::Unique {
                span: Span::from((&tag, &semicolon)),
                nodes,
            },
        ),
    ));

    context("pragma", preceded(tag("@"), expect(pragma, "pragma")))(input)
}

fn pragma_assignment(input: Input) -> Result<PragmaAssignment<Identifier>> {
    into(separated_pair(
        expression,
        cut(ww_char('=')),
        expect_expression,
    ))(input)
}

fn pragma_tag(input: Input) -> Result<PragmaTag<Identifier>> {
    into(pair(
        identifier,
        alt((map(preceded(ww_char(':'), cut(type_)), Some), success(None))),
    ))(input)
}

pub fn game(input: Input) -> Result<Game<Identifier>> {
    context(
        "game",
        terminated(
            fold_many0(
                preceded(
                    comments_and_whitespaces,
                    alt((
                        map(constant, |x| (x, None, None, None, None)),
                        map(typedef, |x| (None, x, None, None, None)),
                        map(variable, |x| (None, None, x, None, None)),
                        map(edge, |x| (None, None, None, x, None)),
                        map(pragma, |x| (None, None, None, None, x)),
                        map(parse_error_line, |()| (None, None, None, None, None)),
                    )),
                ),
                Game::default,
                |mut game, declaration| {
                    match declaration {
                        (Some(constant), _, _, _, _) => game.constants.push(constant),
                        (_, Some(typedef), _, _, _) => game.typedefs.push(typedef),
                        (_, _, Some(variable), _, _) => game.variables.push(variable),
                        (_, _, _, Some(edge), _) => game.edges.push(Arc::from(edge)),
                        (_, _, _, _, Some(pragma)) => game.pragmas.push(pragma),
                        _ => (),
                    }
                    game
                },
            ),
            comments_and_whitespaces,
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
            "Failed to parse:\n{input}\nErrors:\n{}",
            errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert_eq!(
            input,
            game.to_string().trim(),
            "Failed to parse:\n{game}\nExpected:\n{input}"
        );
    }

    fn check_error(input: &str) {
        let (game, errors) = parse_with_errors(input);
        assert!(
            !errors.is_empty(),
            "Expected to fail to parse:\n{input}\nParsed:\n{game}"
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
        check_parse("foo(x: Y)(y: Z), bar: ;");
        check_parse("foo(x: Y), bar(x: Y): ;");
        check_parse("foo(x: Y)(z: Z), bar(y: Z)(v: Z): ;");
        check_parse("type Z = { z };\nbegin, loop(x: X)(y: Y): ;");
    }

    #[test]
    fn incorrect_edge() {
        check_error("foo bar, goo: ;");
        check_error("foo, goo bar: ;");
        check_error("foo (x: Y), goo: ;");
        check_error("foo, goo(x:Y) (y: Y): ;");
        check_error("foo(x: Y) bar, goo: ;");
        check_error("(x: Y)foo, goo: ;");
        check_error("foo,(x: Y)goo: ;");
    }
}
