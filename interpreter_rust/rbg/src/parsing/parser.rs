use crate::ast::{
    Action, Atom, ComparisonOperator, Edge, ExpressionOperator, Game, Node, RValue, Rule, Variable,
};
use nom::branch::{alt, permutation};
use nom::bytes::complete::tag;
use nom::combinator::{all_consuming, opt, value};
use nom::error::Error;
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::Parser;
use std::cell::RefCell;
use std::sync::Arc;
use utils::parser::{
    comma_separated0, comma_separated1, identifier_, in_braces, in_brackets, in_parens, integer,
    ww, ww_char, ww_tag, Input, ParserState, Result,
};
use utils::position::Span;
use utils::{Identifier, ParserError};

const PIECES: &str = "pieces";
const VARIABLES: &str = "variables";
const PLAYERS: &str = "players";
const BOARD: &str = "board";
const RULES: &str = "rules";

type Id = Identifier;

fn identifier(input: Input) -> Result<Id> {
    ww(identifier_.map(|identifier| {
        let span: Span = Span::from(&identifier);
        Identifier::new(span, (*identifier.fragment()).to_string())
    }))
    .parse(input)
}

fn section<'a, O>(
    name: &'static str,
    parser: impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>>,
) -> impl Parser<Input<'a>, Output = O, Error = Error<Input<'a>>> {
    preceded((ww_char('#'), tag(name)), preceded(ww_char('='), parser))
}

fn pieces(input: Input) -> Result<Vec<Id>> {
    section(PIECES, comma_separated0(identifier)).parse(input)
}

fn bounded_variable(input: Input) -> Result<Variable<Id>> {
    (identifier, in_parens(integer))
        .map(|(name, bound)| Variable::new(name, bound))
        .parse(input)
}

fn variables(input: Input) -> Result<Vec<Variable<Id>>> {
    section(VARIABLES, comma_separated0(bounded_variable)).parse(input)
}

fn players(input: Input) -> Result<Vec<Variable<Id>>> {
    section(PLAYERS, comma_separated0(bounded_variable)).parse(input)
}

fn edge(input: Input) -> Result<Edge<Id>> {
    separated_pair(identifier, ww_char(':'), identifier)
        .map(|(node, edge)| Edge::new(node, edge))
        .parse(input)
}

fn node(input: Input) -> Result<Node<Id>> {
    (
        identifier,
        in_brackets(identifier),
        in_braces(comma_separated1(edge)),
    )
        .map(|(name, piece, edges)| Node::new(name, piece, edges))
        .parse(input)
}

fn board(input: Input) -> Result<Vec<Node<Id>>> {
    section(BOARD, many1(node)).parse(input)
}

fn potential_power(input: Input) -> Result<bool> {
    opt(ww_char('*')).map(|c| c.is_some()).parse(input)
}

fn addsub_binop(input: Input) -> Result<ExpressionOperator> {
    ww(alt((
        value(ExpressionOperator::Add, tag("+")),
        value(ExpressionOperator::Sub, tag("-")),
    )))
    .parse(input)
}

fn muldiv_binop(input: Input) -> Result<ExpressionOperator> {
    ww(alt((
        value(ExpressionOperator::Mul, tag("*")),
        value(ExpressionOperator::Div, tag("/")),
    )))
    .parse(input)
}

fn expression(input: Input) -> Result<RValue<Id>> {
    (expression1, opt((muldiv_binop, expression)))
        .map(|(lhs, rhs)| match rhs {
            Some((op, rhs)) => RValue::new_expression(Arc::new(lhs), Arc::new(rhs), op),
            None => lhs,
        })
        .parse(input)
}

fn expression1(input: Input) -> Result<RValue<Id>> {
    (expression2, opt((addsub_binop, expression1)))
        .map(|(lhs, rhs)| match rhs {
            Some((op, rhs)) => RValue::new_expression(Arc::new(lhs), Arc::new(rhs), op),
            None => lhs,
        })
        .parse(input)
}

fn expression2(input: Input) -> Result<RValue<Id>> {
    alt((
        ww(integer).map(RValue::new_number),
        identifier.map(RValue::new_string),
        in_parens(expression),
    ))
    .parse(input)
}

fn comparison_operator(input: Input) -> Result<ComparisonOperator> {
    alt((
        value(ComparisonOperator::Eq, tag("==")),
        value(ComparisonOperator::Ne, tag("!=")),
        value(ComparisonOperator::Lte, tag("<=")),
        value(ComparisonOperator::Lt, tag("<")),
        value(ComparisonOperator::Gte, tag(">=")),
        value(ComparisonOperator::Gt, tag(">")),
    ))
    .parse(input)
}

fn action(input: Input) -> Result<Action<Id>> {
    alt((
        identifier.map(Action::new_shift),
        in_braces(comma_separated0(identifier)).map(Action::new_on),
        in_brackets(identifier).map(Action::new_off),
        delimited(
            ww_tag("[$"),
            separated_pair(identifier, ww_char('='), expression),
            ww_char(']'),
        )
        .map(|(variable, rvalue)| Action::new_assignment(variable, rvalue)),
        delimited(
            ww_tag("{$"),
            (expression, comparison_operator, expression),
            ww_char('}'),
        )
        .map(|(lhs, op, rhs)| Action::new_comparison(lhs, rhs, op)),
        preceded(ww_tag("->"), identifier.map(Some)).map(Action::new_switch),
        value(Action::new_switch(None), ww_tag("->>")),
        delimited(ww_tag("{?"), rule_sum, ww_char('}')).map(|rule| Action::new_check(false, rule)),
        delimited(ww_tag("{!"), rule_sum, ww_char('}')).map(|rule| Action::new_check(true, rule)),
    ))
    .parse(input)
}

fn rule_sum_element(input: Input) -> Result<Vec<Atom<Id>>> {
    many1(alt((
        (action, potential_power).map(|(action, power)| Atom::new_action(action, power)),
        (in_parens(rule_sum), potential_power).map(|(sum, power)| Atom::new_rule(sum, power)),
    )))
    .parse(input)
}

fn rule_sum(input: Input) -> Result<Rule<Id>> {
    separated_list1(ww_char('+'), rule_sum_element)
        .map(|elements| Rule { elements })
        .parse(input)
}

fn rules(input: Input) -> Result<Rule<Id>> {
    section(RULES, rule_sum).parse(input)
}

fn game(input: Input) -> Result<Game<Id>> {
    permutation((pieces, variables, players, board, rules))
        .map(|(pieces, variables, players, board, rules)| Game {
            pieces,
            variables,
            players,
            board,
            rules,
        })
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

#[cfg(test)]
mod test {
    use super::{board, game, pieces, players, rules};
    use nom::combinator::all_consuming;
    use nom::Parser;
    use std::cell::RefCell;
    use utils::parser::{Input, ParserState, Result};

    fn check_parse<O>(parser: impl FnMut(Input) -> Result<O>, input: &str) {
        let errors = RefCell::new(Vec::new());
        let input = nom_locate::LocatedSpan::new_extra(input, ParserState(&errors));
        let _ = all_consuming(parser)
            .parse(input)
            .expect("Parser cannot fail");
        let errors = errors.into_inner();
        assert!(
            errors.is_empty(),
            "Failed to parse. \nErrors:\n{}",
            errors
                .into_iter()
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn players_test() {
        check_parse(players, "#players = a(1), b(2), c(3)");
    }

    #[test]
    fn pieces_test() {
        check_parse(pieces, "#pieces = a, b, c");
    }

    #[test]
    fn board_test() {
        check_parse(
            board,
            r#"
            #board = 
                r1[a] {a: b, b: c, c: a}
                r2[b] {a: b, b: c, c: a}
                r3[c] {a: b, b: c, c: a}
            "#,
        );
    }

    #[test]
    fn rules_test() {
        check_parse(
            rules,
            r#"
            #rules = 
                [$ xplayer=50]
            "#,
        );
    }

    #[test]
    fn game_test() {
        check_parse(
            game,
            "
            #pieces = a, b, c
            #variables = 
            #players = a(1), b(2), c(3)
            #board = 
                r1[a] {a: b, b: c, c: a}
                r2[b] {a: b, b: c, c: a}
                r3[c] {a: b, b: c, c: a}
            #rules = 
                [$ xplayer=50]
            ",
        );
    }
}
