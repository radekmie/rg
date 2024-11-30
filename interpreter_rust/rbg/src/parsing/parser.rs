use crate::ast::{
    Action, Atom, ComparisonOperator, Edge, ExpressionOperator, Game, Node, RValue, Rule, Variable,
};
use nom::branch::{alt, permutation};
use nom::bytes::complete::tag;
use nom::combinator::{all_consuming, map, opt, value};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, tuple};
use std::cell::RefCell;
use std::sync::Arc;
use utils::parser::{
    comma_separated0, comma_separated1, identifier_, in_braces, in_brackets, in_parens, integer,
    ww, ww_char, ww_tag, Input, Result, State,
};
use utils::position::Span;
use utils::Error;
use utils::Identifier;

const PIECES: &str = "pieces";
const VARIABLES: &str = "variables";
const PLAYERS: &str = "players";
const BOARD: &str = "board";
const RULES: &str = "rules";

type Id = Identifier;

fn identifier(input: Input) -> Result<Id> {
    ww(map(identifier_, |identifier| {
        let span: Span = Span::from(&identifier);
        Identifier::new(span, (*identifier.fragment()).to_string())
    }))(input)
}

fn section<'a, O>(
    name: &'static str,
    parser: impl FnMut(Input<'a>) -> Result<O>,
) -> impl FnMut(Input<'a>) -> Result<O> {
    preceded(
        pair(ww_char('#'), tag(name)),
        preceded(ww_char('='), parser),
    )
}

fn pieces(input: Input) -> Result<Vec<Id>> {
    section(PIECES, comma_separated0(identifier))(input)
}

fn bounded_variable(input: Input) -> Result<Variable<Id>> {
    map(pair(identifier, in_parens(integer)), |(name, bound)| {
        Variable::new(name, bound)
    })(input)
}

fn variables(input: Input) -> Result<Vec<Variable<Id>>> {
    section(VARIABLES, comma_separated0(bounded_variable))(input)
}

fn players(input: Input) -> Result<Vec<Variable<Id>>> {
    section(PLAYERS, comma_separated0(bounded_variable))(input)
}

fn edge(input: Input) -> Result<Edge<Id>> {
    map(
        separated_pair(identifier, ww_char(':'), identifier),
        |(node, edge)| Edge::new(node, edge),
    )(input)
}

fn node(input: Input) -> Result<Node<Id>> {
    map(
        tuple((
            identifier,
            in_brackets(identifier),
            in_braces(comma_separated1(edge)),
        )),
        |(name, piece, edges)| Node::new(name, piece, edges),
    )(input)
}

fn board(input: Input) -> Result<Vec<Node<Id>>> {
    section(BOARD, many1(node))(input)
}

fn potential_power(input: Input) -> Result<bool> {
    map(opt(ww_char('*')), |c| c.is_some())(input)
}

fn addsub_binop(input: Input) -> Result<ExpressionOperator> {
    ww(alt((
        value(ExpressionOperator::Add, tag("+")),
        value(ExpressionOperator::Sub, tag("-")),
    )))(input)
}

fn muldiv_binop(input: Input) -> Result<ExpressionOperator> {
    ww(alt((
        value(ExpressionOperator::Mul, tag("*")),
        value(ExpressionOperator::Div, tag("/")),
    )))(input)
}

fn expression(input: Input) -> Result<RValue<Id>> {
    map(
        pair(expression1, opt(pair(muldiv_binop, expression))),
        |(lhs, rhs)| match rhs {
            Some((op, rhs)) => RValue::new_expression(Arc::new(lhs), Arc::new(rhs), op),
            None => lhs,
        },
    )(input)
}

fn expression1(input: Input) -> Result<RValue<Id>> {
    map(
        pair(expression2, opt(pair(addsub_binop, expression1))),
        |(lhs, rhs)| match rhs {
            Some((op, rhs)) => RValue::new_expression(Arc::new(lhs), Arc::new(rhs), op),
            None => lhs,
        },
    )(input)
}

fn expression2(input: Input) -> Result<RValue<Id>> {
    alt((
        map(integer, RValue::new_number),
        map(identifier, RValue::new_string),
        in_parens(expression),
    ))(input)
}

fn comparison_operator(input: Input) -> Result<ComparisonOperator> {
    alt((
        value(ComparisonOperator::Eq, tag("==")),
        value(ComparisonOperator::Ne, tag("!=")),
        value(ComparisonOperator::Lt, tag("<")),
        value(ComparisonOperator::Lte, tag("<=")),
        value(ComparisonOperator::Gt, tag(">")),
        value(ComparisonOperator::Gte, tag(">=")),
    ))(input)
}

fn action(input: Input) -> Result<Action<Id>> {
    alt((
        map(identifier, Action::new_shift),
        map(in_braces(comma_separated0(identifier)), Action::new_on),
        map(in_brackets(identifier), Action::new_off),
        map(
            delimited(
                ww_tag("[$"),
                separated_pair(identifier, ww_char('='), expression),
                ww_char(']'),
            ),
            |(variable, rvalue)| Action::new_assignment(variable, rvalue),
        ),
        map(
            delimited(
                ww_tag("{$"),
                tuple((expression, comparison_operator, expression)),
                ww_char('}'),
            ),
            |(lhs, op, rhs)| Action::new_comparison(lhs, rhs, op),
        ),
        map(preceded(ww_tag("->"), map(identifier, Some)), Action::new_switch),
        value(Action::new_switch(None), ww_tag("->>")),
        map(delimited(ww_tag("{?"), rule_sum, ww_char('}')), |rule| {
            Action::new_check(false, rule)
        }),
        map(delimited(ww_tag("{!"), rule_sum, ww_char('}')), |rule| {
            Action::new_check(true, rule)
        }),
    ))(input)
}

fn rule_sum_element(input: Input) -> Result<Vec<Atom<Id>>> {
    many1(alt((
        map(pair(action, potential_power), |(action, power)| {
            Atom::new_action(action, power)
        }),
        map(
            pair(in_parens(rule_sum), potential_power),
            |(sum, power)| Atom::new_rule(sum, power),
        ),
    )))(input)
}

fn rule_sum(input: Input) -> Result<Rule<Id>> {
    map(
        separated_list1(ww_char('+'), rule_sum_element),
        |elements| Rule { elements },
    )(input)
}

fn rules(input: Input) -> Result<Rule<Id>> {
    section(RULES, rule_sum)(input)
}

fn game(input: Input) -> Result<Game<Id>> {
    map(
        permutation((pieces, variables, players, board, rules)),
        |(pieces, variables, players, board, rules)| Game {
            pieces,
            variables,
            players,
            board,
            rules,
        },
    )(input)
}

pub fn parse_with_errors(input: &str) -> (Game<Identifier>, Vec<Error>) {
    let errors = RefCell::new(Vec::new());
    let input = nom_locate::LocatedSpan::new_extra(input, State(&errors));
    let (_, game) = all_consuming(game)(input).expect("Parser cannot fail");
    (game, errors.into_inner())
}

#[cfg(test)]
mod test {
    use super::{board, game, pieces, players, rules};
    use nom::combinator::all_consuming;
    use std::cell::RefCell;
    use utils::parser::{Input, Result, State};

    fn check_parse<O>(parser: impl FnMut(Input) -> Result<O>, input: &str) {
        let errors = RefCell::new(Vec::new());
        let input = nom_locate::LocatedSpan::new_extra(input, State(&errors));
        let _ = all_consuming(parser)(input).expect("Parser cannot fail");
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
