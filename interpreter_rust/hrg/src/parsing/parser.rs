use crate::ast::{
    Binop, DomainDeclaration, DomainElement, DomainElementPattern, DomainValue, Expression,
    ExpressionMapPart, Function, FunctionArg, FunctionDeclaration, Game, Pattern, Statement, Type,
    VariableDeclaration,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, cut, into, map, opt, success, value, verify};
use nom::error::context;
use nom::multi::{fold_many0, many0, many1, separated_list0, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use std::cell::RefCell;
use std::sync::Arc;
use utils::parser::{
    comma_separated0, comments_and_whitespaces, identifier_, in_braces, in_brackets, in_parens,
    integer, into_arc, parse_error_line, ww, ww_char, ww_tag, Input, Result, State,
};
use utils::position::Span;
use utils::Error;
use utils::Identifier;

pub fn arc_expression(expression: Expression<Identifier>) -> Arc<Expression<Identifier>> {
    Arc::new(expression)
}

fn identifier(input: Input) -> Result<Identifier> {
    ww(map(identifier_, |identifier| {
        let span: Span = Span::from(&identifier);
        Identifier::new(span, (*identifier.fragment()).to_string())
    }))(input)
}

fn assignment(input: Input) -> Result<Statement<Identifier>> {
    into(tuple((
        identifier,
        many0(in_brackets(expression)),
        preceded(
            ww_char('='),
            alt((
                map(terminated(type_, in_parens(char('*'))), Err),
                map(expression, Ok),
            )),
        ),
    )))(input)
}

fn branch(input: Input) -> Result<Statement<Identifier>> {
    preceded(
        ww_tag("branch"),
        map(
            in_braces(separated_list0(
                delimited(ww_char('}'), tag("or"), ww_char('{')),
                many0(statement),
            )),
            |arms| Statement::Branch { arms },
        ),
    )(input)
}

fn branch_var(input: Input) -> Result<Statement<Identifier>> {
    preceded(
        ww_tag("branch"),
        map(
            tuple((
                ww(identifier),
                preceded(ww_tag("in"), type_),
                in_braces(many0(statement)),
            )),
            |(identifier, type_, body)| Statement::BranchVar {
                identifier,
                type_,
                body,
            },
        ),
    )(input)
}

fn call(input: Input) -> Result<Statement<Identifier>> {
    into(pair(identifier, in_parens(comma_separated0(expression))))(input)
}

fn loop_(input: Input) -> Result<Statement<Identifier>> {
    map(preceded(tag("loop"), in_braces(many0(statement))), |body| {
        Statement::Loop { body }
    })(input)
}

fn repeat(input: Input) -> Result<Statement<Identifier>> {
    map(
        pair(
            preceded(ww_tag("repeat"), integer),
            in_braces(many0(statement)),
        ),
        |(count, body)| Statement::Repeat { count, body },
    )(input)
}

fn if_(input: Input) -> Result<Statement<Identifier>> {
    map(
        tuple((
            preceded(tag("if"), expression),
            in_braces(many0(statement)),
            opt(preceded(
                ww_tag("else"),
                alt((
                    in_braces(many0(statement)),
                    map(if_, |statement| vec![statement]),
                )),
            )),
        )),
        |(expression, then, else_)| Statement::If {
            expression,
            then,
            else_,
        },
    )(input)
}

fn while_(input: Input) -> Result<Statement<Identifier>> {
    map(
        pair(
            preceded(tag("while"), expression),
            in_braces(many0(statement)),
        ),
        |(expression, body)| Statement::While { expression, body },
    )(input)
}

fn tag_statement(input: Input) -> Result<Statement<Identifier>> {
    map(
        preceded(char('$'), pair(opt(char('_')), identifier)),
        |(artificial, symbol)| Statement::Tag {
            artificial: artificial.is_some(),
            symbol,
        },
    )(input)
}

fn tag_variable_statement(input: Input) -> Result<Statement<Identifier>> {
    map(preceded(tag("$$"), identifier), |identifier| {
        Statement::TagVariable { identifier }
    })(input)
}

fn statement(input: Input) -> Result<Statement<Identifier>> {
    ww(alt((
        assignment,
        branch_var,
        branch,
        call,
        if_,
        loop_,
        repeat,
        while_,
        tag_variable_statement,
        tag_statement,
    )))(input)
}

fn domain_element(input: Input) -> Result<DomainElement<Identifier>> {
    ww(alt((
        into(tuple((
            identifier,
            in_parens(comma_separated0(domain_element_pattern)),
            preceded(ww_tag("where"), comma_separated0(domain_value)),
        ))),
        into(identifier),
    )))(input)
}

fn domain_element_pattern(input: Input) -> Result<DomainElementPattern<Identifier>> {
    map(identifier, |identifier| {
        if Pattern::is_literal(&identifier.identifier) {
            DomainElementPattern::Literal { identifier }
        } else {
            DomainElementPattern::Variable { identifier }
        }
    })(input)
}

fn domain_value(input: Input) -> Result<DomainValue<Identifier>> {
    ww(alt((
        into(pair(
            terminated(identifier, ww_tag("in")),
            ww(separated_pair(integer, ww_tag(".."), integer)),
        )),
        into(pair(
            terminated(identifier, ww_tag("in")),
            in_braces(comma_separated0(identifier)),
        )),
    )))(input)
}

fn expression(input: Input) -> Result<Arc<Expression<Identifier>>> {
    alt((
        into_arc(tuple((
            preceded(ww_tag("if"), expression),
            preceded(tag("then"), expression),
            preceded(tag("else"), expression),
        ))),
        into_arc(in_braces(alt((
            pair(
                preceded(char(':'), cut(map(expression, Some))),
                alt((
                    preceded(char(';'), separated_list1(char(';'), expression_map_part)),
                    success(vec![]),
                )),
            ),
            pair(
                success(None),
                separated_list1(char(';'), expression_map_part),
            ),
        )))),
        map(
            pair(expression2, opt(preceded(ww_tag("||"), expression))),
            |(lhs, rhs)| match rhs {
                Some(rhs) => Arc::new(Expression::BinExpr {
                    lhs,
                    op: Binop::Or,
                    rhs,
                }),
                None => lhs,
            },
        ),
    ))(input)
}

fn expression2(input: Input) -> Result<Arc<Expression<Identifier>>> {
    map(
        pair(expression3, opt(preceded(ww_tag("&&"), expression2))),
        |(lhs, rhs)| match rhs {
            Some(rhs) => Arc::new(Expression::BinExpr {
                lhs,
                op: Binop::And,
                rhs,
            }),
            None => lhs,
        },
    )(input)
}

fn comp_binop(input: Input) -> Result<Binop> {
    ww(alt((
        value(Binop::Eq, tag("==")),
        value(Binop::Ne, tag("!=")),
        value(Binop::Lte, tag("<=")),
        value(Binop::Lt, tag("<")),
        value(Binop::Gte, tag(">=")),
        value(Binop::Gt, tag(">")),
    )))(input)
}

fn expression3(input: Input) -> Result<Arc<Expression<Identifier>>> {
    map(
        pair(expression4, opt(pair(comp_binop, expression3))),
        |(lhs, rhs)| match rhs {
            Some((op, rhs)) => Arc::new(Expression::BinExpr { lhs, op, rhs }),
            None => lhs,
        },
    )(input)
}

fn addsub_binop(input: Input) -> Result<Binop> {
    ww(alt((
        value(Binop::Add, tag("+")),
        value(Binop::Mod, tag("%")),
        value(Binop::Sub, tag("-")),
    )))(input)
}

fn expression4(input: Input) -> Result<Arc<Expression<Identifier>>> {
    map(
        pair(expression5, opt(pair(addsub_binop, expression4))),
        |(lhs, rhs)| match rhs {
            Some((op, rhs)) => Arc::new(Expression::BinExpr { lhs, op, rhs }),
            None => lhs,
        },
    )(input)
}

fn expression5(input: Input) -> Result<Arc<Expression<Identifier>>> {
    let (input, identifier) = opt(identifier)(input)?;
    match identifier {
        Some(identifier) => {
            if Pattern::is_literal(&identifier.identifier) {
                let expression = Arc::new(identifier.into());
                expression_suffix(input, expression)
            } else {
                let (input, args) = opt(in_parens(comma_separated0(expression)))(input)?;
                if let Some(args) = args {
                    let expr = Arc::new(Expression::Constructor { identifier, args });
                    Ok((input, expr))
                } else {
                    let expression = Arc::new(identifier.into());
                    expression_suffix(input, expression)
                }
            }
        }
        None => in_parens(expression)(input),
    }
}

fn expression_map_part(input: Input) -> Result<ExpressionMapPart<Identifier>> {
    into(tuple((
        pattern,
        preceded(char('='), expression),
        opt(preceded(tag("where"), comma_separated0(domain_value))),
    )))(input)
}

fn expression_suffix(
    input: Input,
    lhs: Arc<Expression<Identifier>>,
) -> Result<Arc<Expression<Identifier>>> {
    let (input, access) = opt(in_brackets(expression))(input)?;
    let (input, args) = opt(in_parens(comma_separated0(expression)))(input)?;

    match access {
        Some(rhs) => {
            let lhs = Arc::new(Expression::Access { lhs, rhs });
            match args {
                Some(args) => expression_suffix(
                    input,
                    Arc::new(Expression::Call {
                        expression: lhs,
                        args,
                    }),
                ),
                None => expression_suffix(input, lhs),
            }
        }
        None => match args {
            Some(args) => expression_suffix(
                input,
                Arc::new(Expression::Call {
                    expression: lhs,
                    args,
                }),
            ),
            None => Ok((input, lhs)),
        },
    }
}

fn pattern(input: Input) -> Result<Arc<Pattern<Identifier>>> {
    ww(alt((
        map(char('_'), |_| Arc::new(Pattern::Wildcard)),
        into_arc(pair(identifier, in_parens(comma_separated0(pattern)))),
        map(identifier, |identifier| {
            if Pattern::is_literal(&identifier.identifier) {
                Arc::new(Pattern::Literal { identifier })
            } else {
                Arc::new(Pattern::Variable { identifier })
            }
        }),
    )))(input)
}

fn type_(input: Input) -> Result<Arc<Type<Identifier>>> {
    let (input, lhs): (Input, Arc<Type<Identifier>>) = into_arc(identifier)(input)?;
    match opt(preceded(tag("->"), type_))(input)? {
        (input, Some(rhs)) => Ok((input, Arc::new(Type::Function { lhs, rhs }))),
        (input, None) => Ok((input, lhs)),
    }
}

fn function(input: Input) -> Result<Function<Identifier>> {
    into(tuple((
        alt((value(true, ww_tag("reusable")), success(false))),
        preceded(ww_tag("graph"), identifier),
        in_parens(comma_separated0(function_arg)),
        in_braces(many0(statement)),
    )))(input)
}

fn function_arg(input: Input) -> Result<FunctionArg<Identifier>> {
    into(separated_pair(identifier, char(':'), type_))(input)
}

fn domain_declaration(input: Input) -> Result<DomainDeclaration<Identifier>> {
    into(pair(
        delimited(ww_tag("domain"), identifier, ww_char('=')),
        separated_list0(ww_char('|'), domain_element),
    ))(input)
}

fn function_declaration(input: Input) -> Result<FunctionDeclaration<Identifier>> {
    let (input, (id, type_)) = separated_pair(identifier, char(':'), type_)(input)?;
    let (input, cases) = many1(pair(
        ww(pair(
            verify(identifier, |identifier| {
                identifier.identifier == id.identifier
            }),
            in_parens(comma_separated0(pattern)),
        )),
        preceded(ww_char('='), expression),
    ))(input)?;
    Ok((input, (id, type_, cases).into()))
}

fn variable_declaration(input: Input) -> Result<VariableDeclaration<Identifier>> {
    into(tuple((
        identifier,
        preceded(char(':'), type_),
        opt(preceded(ww_char('='), expression)),
    )))(input)
}

fn game(input: Input) -> Result<Game<Identifier>> {
    context(
        "game_declaration",
        terminated(
            fold_many0(
                preceded(
                    comments_and_whitespaces,
                    alt((
                        map(domain_declaration, |x| (Some(x), None, None, None)),
                        map(function_declaration, |x| (None, Some(x), None, None)),
                        map(variable_declaration, |x| (None, None, Some(x), None)),
                        map(function, |x| (None, None, None, Some(x))),
                        map(parse_error_line, |()| (None, None, None, None)),
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
              if direction(me)(position) == null || not(reachable(move(opponent(me)))) {\n    \
                end()\n  \
              }\n\
            }",
        );
    }
}
