use crate::ast::{
    Binop, DomainDeclaration, DomainElement, DomainValue, Expression, Function, FunctionArg,
    FunctionDeclaration, GameDeclaration, Pattern, Statement, Type, TypeDeclaration,
    VariableDeclaration,
};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::{all_consuming, into, map, opt, value, verify};
use nom::error::context;
use nom::multi::{fold_many0, many0, many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use std::cell::RefCell;
use std::sync::Arc;
use utils::parser::{
    comma_separated, comments_and_whitespaces, identifier_, in_braces, in_brackets, in_parens,
    into_arc, parse_error_line, ww, ww_char, ww_tag, Input, Result, State,
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
        preceded(ww_char('='), expression),
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

fn call(input: Input) -> Result<Statement<Identifier>> {
    into(pair(identifier, in_parens(comma_separated(expression))))(input)
}

fn forall(input: Input) -> Result<Statement<Identifier>> {
    into(pair(
        preceded(tag("forall"), separated_pair(identifier, char(':'), type_)),
        in_braces(many0(statement)),
    ))(input)
}

fn loop_(input: Input) -> Result<Statement<Identifier>> {
    map(preceded(tag("loop"), in_braces(many0(statement))), |body| {
        Statement::Loop { body }
    })(input)
}

fn when(input: Input) -> Result<Statement<Identifier>> {
    map(
        pair(
            preceded(tag("when"), expression),
            in_braces(many0(statement)),
        ),
        |(condition, body)| Statement::When { condition, body },
    )(input)
}

fn while_(input: Input) -> Result<Statement<Identifier>> {
    map(
        pair(
            preceded(tag("while"), expression),
            in_braces(many0(statement)),
        ),
        |(condition, body)| Statement::While { condition, body },
    )(input)
}

fn tag_statement(input: Input) -> Result<Statement<Identifier>> {
    map(preceded(char('$'), identifier), |symbol| Statement::Tag {
        symbol,
    })(input)
}

fn statement(input: Input) -> Result<Statement<Identifier>> {
    ww(alt((
        assignment,
        branch,
        call,
        forall,
        loop_,
        when,
        while_,
        tag_statement,
    )))(input)
}

fn domain_element(input: Input) -> Result<DomainElement<Identifier>> {
    ww(alt((
        into(tuple((
            identifier,
            in_parens(comma_separated(identifier)),
            preceded(ww_tag("where"), comma_separated(domain_value)),
        ))),
        into(identifier),
    )))(input)
}

fn domain_value(input: Input) -> Result<DomainValue<Identifier>> {
    ww(alt((
        into(pair(
            terminated(identifier, ww_tag("in")),
            ww(separated_pair(identifier, ww_tag(".."), identifier)),
        )),
        into(pair(
            terminated(identifier, ww_tag("in")),
            in_braces(comma_separated(identifier)),
        )),
    )))(input)
}

fn binop(input: Input) -> Result<Binop> {
    ww(alt((
        value(Binop::Add, tag("+")),
        value(Binop::Sub, tag("-")),
        value(Binop::And, tag("&&")),
        value(Binop::Or, tag("||")),
        value(Binop::Eq, tag("==")),
        value(Binop::Ne, tag("!=")),
        value(Binop::Lt, tag("<")),
        value(Binop::Lte, tag("<=")),
        value(Binop::Gt, tag(">")),
        value(Binop::Gte, tag(">=")),
    )))(input)
}

fn expression(input: Input) -> Result<Arc<Expression<Identifier>>> {
    let (input, lhs) = alt((
        into_arc(in_braces(pair(
            separated_pair(pattern, char('='), expression),
            opt(preceded(tag("where"), comma_separated(domain_value))),
        ))),
        into_arc(tuple((
            preceded(ww_tag("if"), expression),
            preceded(tag("then"), expression),
            preceded(tag("else"), expression),
        ))),
        into_arc(pair(identifier, in_parens(comma_separated(expression)))),
        in_parens(expression),
        into_arc(identifier),
    ))(input)?;
    let (input, lhs) = left_rec_expr(input, lhs)?;
    let (input, op) = opt(binop)(input)?;
    match op {
        Some(op) => {
            let (input, rhs) = expression(input)?;
            Ok((input, Arc::new(Expression::BinExpr { lhs, op, rhs })))
        }
        None => Ok((input, lhs)),
    }
}

fn left_rec_expr(
    input: Input,
    lhs: Arc<Expression<Identifier>>,
) -> Result<Arc<Expression<Identifier>>> {
    let (input, access) = opt(in_brackets(expression))(input)?;
    let (input, args) = opt(in_parens(comma_separated(expression)))(input)?;

    match access {
        Some(rhs) => {
            let lhs = Arc::new(Expression::Access { lhs, rhs });
            match args {
                Some(args) => left_rec_expr(
                    input,
                    Arc::new(Expression::Call {
                        expression: lhs,
                        args,
                    }),
                ),
                None => left_rec_expr(input, lhs),
            }
        }
        None => match args {
            Some(args) => left_rec_expr(
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
        into_arc(pair(identifier, in_parens(comma_separated(pattern)))),
        into_arc(identifier),
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
        preceded(ww_tag("graph"), identifier),
        in_parens(comma_separated(function_arg)),
        in_braces(many0(statement)),
    )))(input)
}

fn function_arg(input: Input) -> Result<FunctionArg<Identifier>> {
    into(separated_pair(identifier, char(':'), type_))(input)
}

fn type_declaration(input: Input) -> Result<TypeDeclaration<Identifier>> {
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
            in_parens(comma_separated(pattern)),
        )),
        preceded(ww_char('='), expression),
    ))(input)?;
    Ok((input, (id, type_, cases).into()))
}

fn variable_declaration(input: Input) -> Result<VariableDeclaration<Identifier>> {
    let (input, (id, type_)) = separated_pair(identifier, char(':'), type_)(input)?;
    let (input, default_value) = opt(ww(pair(
        terminated(
            verify(identifier, |identifier| {
                identifier.identifier == id.identifier
            }),
            ww_char('='),
        ),
        expression,
    )))(input)?;
    Ok((input, (id, type_, default_value).into()))
}

pub fn game(input: Input) -> Result<GameDeclaration<Identifier>> {
    context(
        "game_declaration",
        terminated(
            fold_many0(
                preceded(
                    comments_and_whitespaces,
                    alt((
                        map(domain_declaration, |x| (Some(x), None, None, None, None)),
                        map(function_declaration, |x| (None, Some(x), None, None, None)),
                        map(variable_declaration, |x| (None, None, Some(x), None, None)),
                        map(function, |x| (None, None, None, Some(x), None)),
                        map(type_declaration, |x| (None, None, None, None, Some(x))),
                        map(parse_error_line, |()| (None, None, None, None, None)),
                    )),
                ),
                GameDeclaration::default,
                |mut game, declaration| {
                    match declaration {
                        (Some(domain), _, _, _, _) => game.domains.push(domain),
                        (_, Some(function), _, _, _) => game.functions.push(function),
                        (_, _, Some(variable), _, _) => game.variables.push(variable),
                        (_, _, _, Some(func), _) => game.automaton.push(func),
                        (_, _, _, _, Some(type_)) => game.types.push(type_),
                        _ => (),
                    }
                    game
                },
            ),
            comments_and_whitespaces,
        ),
    )(input)
}

pub fn parse_with_errors(input: &str) -> (GameDeclaration<Identifier>, Vec<Error>) {
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
        // dbg!(game.clone());
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
            input.trim(),
            game.to_string().trim(),
            "Failed to parse:\n{game}\nExpected:\n{input}"
        );
    }

    #[test]
    fn typedef() {
        check_parse("A : B");
        check_parse("Foo : Bar -> Baz -> Goo");
    }

    #[test]
    fn breakthrough() {
        check_parse(
            "     
            // Moves form: Position (F + L + R)

            // Domains
            domain Piece = blackpawn | empty | whitepawn
            domain Player = black | white
            domain Score = 0 | 100
            domain Position = null | V(X, Y) where X in 0..7, Y in { 0, 1, 2, 3, 4, 5, 6, 7 }
            
            // Helpers
            left : Position -> Position
            left(null) = null
            left(V(X, Y)) = if X == 0 then null else V(X - 1, Y)
            
            right : Position -> Position
            right(null) = null
            right(V(X, Y)) = if X == 7 then null else V(X + 1, Y)
            
            up : Position -> Position
            up(null) = null
            up(V(_, 0)) = null
            up(V(X, Y)) = V(X, Y - 1)
            
            down : Position -> Position
            down(null) = null
            down(V(_, 7)) = null
            down(V(X, Y)) = V(X, Y + 1)
            
            direction : Player -> Position -> Position
            direction(white) = up
            direction(_) = down
            
            piece : Player -> Piece
            piece(white) = whitepawn
            piece(_) = blackpawn
            
            opponent : Player -> Player
            opponent(white) = black
            opponent(_) = white
            
            
            // Variables
            board : Position -> Piece
            board = {
              V(X, Y) = if Y < 2
                then blackpawn
                else if Y > 5
                  then whitepawn
                  else empty
              where X in 0..7, Y in 0..7
            }
            
            me : Player
            me = white
            
            position : Position
            
            
            // Automaton
            graph move(me: Player) {
              forall p:Position {
                assert(p != null && board[p] == piece(me))
                board[p] = empty
                position = direction(me)(p)
                $ p
              }
              branch {
                assert(board[position] == empty)
                $ F
              } or {
                branch {
                  position = left(position)
                  $ L
                } or {
                  position = right(position)
                  $ R
                }
                assert(position != null)
                branch {
                  assert(board[position] == empty)
                } or {
                  assert(board[position] == piece(opponent(me)))
                }
              }
            }
            
            graph turn() {
              player = me
              move(me)
              board[position] = piece(me)
              player = keeper
              when direction(me)(position) == null || not(reachable(move(opponent(me)))) {
                goals[me] = 100
                goals[opponent(me)] = 0
                end()
              }
              me = opponent(me)
            }
            
            graph rules() {
              loop {
                turn()
              }
            }
            
        ",
        );
    }
}
