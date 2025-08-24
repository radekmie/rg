use crate::ast::{
    Binop, DomainDeclaration, DomainElement, DomainElementPattern, DomainValue, Expression,
    ExpressionMapPart, Function, FunctionArg, FunctionDeclaration, Game, Pattern, Statement, Type,
    VariableDeclaration,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, fail, into, opt, success, value, verify};
use nom::error::context;
use nom::error::Error;
use nom::multi::{fold_many0, many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::Parser;
use nom_language::precedence::{binary_op, precedence, unary_op, Assoc, Operation};
use std::cell::RefCell;
use std::sync::Arc;
use utils::parser::{
    comma_separated0, comma_separated1, comments_and_whitespaces0, comments_and_whitespaces1,
    identifier_, in_braces, in_brackets, in_parens, integer, into_arc, parse_error_line, ww,
    ww_char, ww_tag, Input, ParserState, Result,
};
use utils::position::Span;
use utils::{Identifier, ParserError};

fn identifier(input: Input) -> Result<Identifier> {
    ww(identifier_.map(|identifier| {
        let span: Span = Span::from(&identifier);
        Identifier::new(span, (*identifier.fragment()).to_string())
    }))
    .parse(input)
}

fn assignment(input: Input) -> Result<Statement<Identifier>> {
    into((
        identifier,
        many0(in_brackets(expression)),
        preceded(
            ww_char('='),
            alt((
                terminated(type_, in_parens(char('*'))).map(Err),
                expression.map(Ok),
            )),
        ),
    ))
    .parse(input)
}

fn branch(input: Input) -> Result<Statement<Identifier>> {
    preceded(
        ww_tag("branch"),
        in_braces(separated_list0(
            delimited(ww_char('}'), tag("or"), ww_char('{')),
            many0(statement),
        ))
        .map(|arms| Statement::Branch { arms }),
    )
    .parse(input)
}

fn branch_var(input: Input) -> Result<Statement<Identifier>> {
    preceded(
        ww_tag("branch"),
        (
            ww(identifier),
            preceded(ww_tag("in"), type_),
            in_braces(many0(statement)),
        )
            .map(|(identifier, type_, body)| Statement::BranchVar {
                identifier,
                type_,
                body,
            }),
    )
    .parse(input)
}

fn call(input: Input) -> Result<Statement<Identifier>> {
    into((identifier, in_parens(comma_separated0(expression)))).parse(input)
}

fn loop_(input: Input) -> Result<Statement<Identifier>> {
    preceded(tag("loop"), in_braces(many0(statement)))
        .map(|body| Statement::Loop { body })
        .parse(input)
}

fn repeat(input: Input) -> Result<Statement<Identifier>> {
    (
        preceded(ww_tag("repeat"), integer),
        in_braces(many0(statement)),
    )
        .map(|(count, body)| Statement::Repeat { count, body })
        .parse(input)
}

fn repeat_var(input: Input) -> Result<Statement<Identifier>> {
    preceded(
        ww_tag("repeat"),
        (
            ww(identifier),
            preceded(ww_tag("in"), type_),
            in_braces(many0(statement)),
        )
            .map(|(identifier, type_, body)| Statement::RepeatVar {
                identifier,
                type_,
                body,
            }),
    )
    .parse(input)
}

fn if_(input: Input) -> Result<Statement<Identifier>> {
    (
        preceded(tag("if"), expression),
        in_braces(many0(statement)),
        opt(preceded(
            ww_tag("else"),
            alt((
                in_braces(many0(statement)),
                if_.map(|statement| vec![statement]),
            )),
        )),
    )
        .map(|(expression, then, else_)| Statement::If {
            expression,
            then,
            else_,
        })
        .parse(input)
}

fn while_(input: Input) -> Result<Statement<Identifier>> {
    (
        preceded(tag("while"), expression),
        in_braces(many0(statement)),
    )
        .map(|(expression, body)| Statement::While { expression, body })
        .parse(input)
}

fn tag_statement(input: Input) -> Result<Statement<Identifier>> {
    preceded(
        char('$'),
        (
            opt(char('_')),
            alt((
                in_parens(many1(identifier)),
                identifier.map(|symbol| vec![symbol]),
            )),
        ),
    )
    .map(|(artificial, symbols)| Statement::Tag {
        artificial: artificial.is_some(),
        symbols,
    })
    .parse(input)
}

fn tag_variable_statement(input: Input) -> Result<Statement<Identifier>> {
    preceded(tag("$$"), identifier)
        .map(|identifier| Statement::TagVariable { identifier })
        .parse(input)
}

fn statement(input: Input) -> Result<Statement<Identifier>> {
    ww(alt((
        assignment,
        branch_var,
        branch,
        call,
        if_,
        loop_,
        repeat_var,
        repeat,
        while_,
        tag_variable_statement,
        tag_statement,
    )))
    .parse(input)
}

fn domain_element(input: Input) -> Result<DomainElement<Identifier>> {
    ww(alt((
        into((
            identifier,
            in_parens(comma_separated0(domain_element_pattern)),
            preceded(ww_tag("where"), comma_separated0(domain_value)),
        )),
        into(identifier),
    )))
    .parse(input)
}

fn domain_element_pattern(input: Input) -> Result<DomainElementPattern<Identifier>> {
    identifier
        .map(|identifier| {
            if Pattern::is_literal(&identifier.identifier) {
                DomainElementPattern::Literal { identifier }
            } else {
                DomainElementPattern::Variable { identifier }
            }
        })
        .parse(input)
}

fn domain_value(input: Input) -> Result<DomainValue<Identifier>> {
    ww(alt((
        into((
            terminated(identifier, ww_tag("in")),
            ww(separated_pair(integer, ww_tag(".."), integer)),
        )),
        into((
            terminated(identifier, ww_tag("in")),
            in_braces(comma_separated0(identifier)),
        )),
    )))
    .parse(input)
}

fn expression(input: Input) -> Result<Arc<Expression<Identifier>>> {
    precedence(
        fail(),
        unary_op(
            0,
            alt((
                in_brackets(expression).map(|x| (Some(x), None)),
                in_parens(comma_separated0(expression)).map(|x| (None, Some(x))),
            )),
        ),
        ww(alt((
            binary_op(
                1,
                Assoc::Left,
                alt((
                    value(Binop::Add, tag("+")),
                    // Force a comment or some whitespace to prevent consuming the next identifier.
                    value(Binop::In, terminated(tag("in"), comments_and_whitespaces1)),
                    value(Binop::Mod, tag("%")),
                    value(Binop::Sub, tag("-")),
                )),
            ),
            binary_op(
                2,
                Assoc::Left,
                alt((
                    value(Binop::Eq, tag("==")),
                    value(Binop::Ne, tag("!=")),
                    value(Binop::Lte, tag("<=")),
                    value(Binop::Lt, tag("<")),
                    value(Binop::Gte, tag(">=")),
                    value(Binop::Gt, tag(">")),
                )),
            ),
            binary_op(3, Assoc::Right, value(Binop::And, tag("&&"))),
            binary_op(4, Assoc::Right, value(Binop::Or, tag("||"))),
        ))),
        alt((
            into_arc((
                preceded(ww_tag("if"), expression),
                preceded(tag("then"), expression),
                preceded(tag("else"), expression),
            )),
            into_arc(in_braces(alt((
                (
                    preceded(char(':'), cut(expression.map(Some))),
                    alt((
                        preceded(char(';'), separated_list1(char(';'), expression_map_part)),
                        success(vec![]),
                    )),
                ),
                (
                    success(None),
                    separated_list1(char(';'), expression_map_part),
                ),
            )))),
            into_arc(verify(identifier, |x| Pattern::is_literal(&x.identifier))),
            (
                verify(identifier, |x| !Pattern::is_literal(&x.identifier)),
                in_parens(comma_separated0(expression)),
            )
                .map(|(identifier, args)| Arc::new(Expression::Constructor { identifier, args })),
            into_arc(identifier),
            in_parens(expression),
        )),
        |operation: Operation<(), _, _, _>| {
            Ok::<_, Error<Input<'_>>>(Arc::from(match operation {
                Operation::Binary(lhs, op, rhs) => Expression::BinExpr { lhs, op, rhs },
                Operation::Postfix(lhs, (Some(rhs), None)) => Expression::Access { lhs, rhs },
                Operation::Postfix(expression, (None, Some(args))) => {
                    Expression::Call { expression, args }
                }
                _ => unreachable!(),
            }))
        },
    )(input)
}

fn expression_map_part(input: Input) -> Result<ExpressionMapPart<Identifier>> {
    into((
        pattern,
        preceded(char('='), expression),
        opt(preceded(tag("where"), comma_separated0(domain_value))),
    ))
    .parse(input)
}

fn pattern(input: Input) -> Result<Arc<Pattern<Identifier>>> {
    ww(alt((
        char('_').map(|_| Arc::new(Pattern::Wildcard)),
        into_arc((identifier, in_parens(comma_separated0(pattern)))),
        identifier.map(|identifier| {
            if Pattern::is_literal(&identifier.identifier) {
                Arc::new(Pattern::Literal { identifier })
            } else {
                Arc::new(Pattern::Variable { identifier })
            }
        }),
    )))
    .parse(input)
}

fn type_(input: Input) -> Result<Arc<Type<Identifier>>> {
    alt((
        into_arc(in_braces(cut(comma_separated1(identifier)))),
        |input| {
            let (input, lhs) = into_arc(identifier).parse(input)?;
            match opt(preceded(tag("->"), type_)).parse(input)? {
                (input, Some(rhs)) => Ok((input, Arc::from(Type::Function { lhs, rhs }))),
                (input, None) => Ok((input, lhs)),
            }
        },
    ))
    .parse(input)
}

fn function(input: Input) -> Result<Function<Identifier>> {
    into((
        alt((value(true, ww_tag("reusable")), success(false))),
        preceded(ww_tag("graph"), identifier),
        in_parens(comma_separated0(function_arg)),
        in_braces(many0(statement)),
    ))
    .parse(input)
}

fn function_arg(input: Input) -> Result<FunctionArg<Identifier>> {
    into(separated_pair(identifier, char(':'), type_)).parse(input)
}

fn domain_declaration(input: Input) -> Result<DomainDeclaration<Identifier>> {
    into((
        delimited(ww_tag("domain"), identifier, ww_char('=')),
        separated_list0(ww_char('|'), domain_element),
    ))
    .parse(input)
}

fn function_declaration(input: Input) -> Result<FunctionDeclaration<Identifier>> {
    let (input, (id, type_)) = separated_pair(identifier, char(':'), type_).parse(input)?;
    let (input, cases) = many1((
        ww((
            verify(identifier, |identifier| {
                identifier.identifier == id.identifier
            }),
            in_parens(comma_separated0(pattern)),
        )),
        preceded(ww_char('='), expression),
    ))
    .parse(input)?;
    Ok((input, (id, type_, cases).into()))
}

fn variable_declaration(input: Input) -> Result<VariableDeclaration<Identifier>> {
    into((
        identifier,
        preceded(char(':'), type_),
        opt(preceded(ww_char('='), expression)),
    ))
    .parse(input)
}

fn game(input: Input) -> Result<Game<Identifier>> {
    context(
        "game_declaration",
        terminated(
            fold_many0(
                preceded(
                    comments_and_whitespaces0,
                    alt((
                        domain_declaration.map(|x| (Some(x), None, None, None)),
                        function_declaration.map(|x| (None, Some(x), None, None)),
                        variable_declaration.map(|x| (None, None, Some(x), None)),
                        function.map(|x| (None, None, None, Some(x))),
                        parse_error_line.map(|()| (None, None, None, None)),
                    )),
                ),
                Game::default,
                |mut game, declaration| {
                    match declaration {
                        (Some(domain), _, _, _) => game.domains.push(domain),
                        (_, Some(function), _, _) => game.functions.push(function),
                        (_, _, Some(variable), _) => game.variables.push(variable),
                        (_, _, _, Some(func)) => game.automaton.push(func),
                        _ => (),
                    }
                    game
                },
            ),
            comments_and_whitespaces0,
        ),
    )
    .parse(input)
}

pub fn parse_with_errors(input: &str) -> (Game<Identifier>, Vec<ParserError>) {
    let errors = RefCell::new(Vec::new());
    let input = nom_locate::LocatedSpan::new_extra(input, ParserState(&errors));
    let (_, game) = all_consuming(game)
        .parse(input)
        .expect("Parser cannot fail");
    (game, errors.into_inner())
}

pub fn parse_expression(input: &str) -> Arc<Expression<Identifier>> {
    let errors = RefCell::new(Vec::new());
    let input = nom_locate::LocatedSpan::new_extra(input, ParserState(&errors));
    let (_, expression) = all_consuming(expression)
        .parse(input)
        .expect("Parser cannot fail");
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

    #[test]
    fn operator_precedence() {
        check_parse(
            "next_d1 : Position -> Position\n\
            next_d1(P(I, J)) = if I == J\n  \
              then P((I + 1) % 3, (J + 1) % 3)\n  \
              else P(I, J)",
        );
        check_parse(
            "graph foo() {\n  \
              if me == first || (direction(me)(position) == null || not(reachable(move(opponent(me))))) {\n    \
                end()\n  \
              }\n\
            }",
        );
    }
}
