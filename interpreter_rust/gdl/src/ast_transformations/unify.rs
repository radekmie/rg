use crate::ast::{AtomOrVariable, Term};
use std::collections::BTreeMap;
use std::iter::zip;

impl<Symbol: Clone + Ord> AtomOrVariable<Symbol> {
    pub fn unify(&self, other: &Self) -> Unification<Symbol> {
        use AtomOrVariable::*;
        use Unification::*;
        match (self, other) {
            (Variable(x), y @ Atom(_)) => {
                NotEmpty(BTreeMap::from([(x.clone(), y.clone().as_term())]))
            }
            (x, y) if x == y => Empty,
            _ => Failed,
        }
    }
}

impl<Symbol: Clone + Ord> Term<Symbol> {
    pub fn unify(&self, other: &Self) -> Unification<Symbol> {
        use Term::*;
        match (self, other) {
            (Base(x), Base(y)) => x.unify(y),
            (Custom(xn, None), Custom(yn, None)) => xn.unify(yn),
            (Custom(xn, Some(xa)), Custom(yn, Some(ya))) if xn == yn => {
                assert!(xa.len() == ya.len());
                zip(xa, ya).map(|(x, y)| x.unify(y)).collect()
            }
            (Does(xr, xa), Does(yr, ya)) => xr.unify(yr).merge(xa.unify(ya)),
            (Goal(xr, xu), Goal(yr, yu)) => xr.unify(yr).merge(xu.unify(yu)),
            (Init(x), Init(y)) => x.unify(y),
            (Input(xr, xa), Input(yr, ya)) => xr.unify(yr).merge(xa.unify(ya)),
            (Legal(xr, xa), Legal(yr, ya)) => xr.unify(yr).merge(xa.unify(ya)),
            (Next(x), Next(y)) => x.unify(y),
            (Role(x), Role(y)) => x.unify(y),
            (Terminal, Terminal) => Unification::Empty,
            (True(x), True(y)) => x.unify(y),
            _ => Unification::Failed,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Unification<Symbol> {
    Empty,
    Failed,
    NotEmpty(BTreeMap<Symbol, Term<Symbol>>),
}

impl<Symbol> Unification<Symbol> {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty | Self::Failed)
    }
}

impl<Symbol: Ord> Unification<Symbol> {
    pub fn get(&self, symbol: &Symbol) -> Option<&Term<Symbol>> {
        match self {
            Self::NotEmpty(mapping) => mapping.get(symbol),
            _ => None,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        use Unification::*;
        match (self, other) {
            (x, Empty) => x,
            (Empty, y) => y,
            (NotEmpty(x), NotEmpty(y)) => {
                NotEmpty(y.into_iter().fold(x, |mut mapping, (variable, atom)| {
                    mapping
                        .entry(variable)
                        .and_modify(|existing| assert!(existing == &atom))
                        .or_insert(atom);
                    mapping
                }))
            }
            _ => Failed,
        }
    }
}

impl<Symbol: Ord> FromIterator<Unification<Symbol>> for Unification<Symbol> {
    fn from_iter<I: IntoIterator<Item = Self>>(iter: I) -> Self {
        let mut u = Self::Empty;
        for x in iter {
            u = u.merge(x);
            if u == Self::Failed {
                break;
            }
        }

        u
    }
}

#[cfg(test)]
mod test {
    use super::Unification::*;
    use crate::ast::Term;
    use crate::parser::infix::term;
    use nom::combinator::all_consuming;
    use std::collections::BTreeMap;

    fn parse(input: &str) -> Term<&str> {
        all_consuming(term)(&input).unwrap().1
    }

    macro_rules! map {
        ($($k:expr => $v:expr),* $(,)?) => {
            BTreeMap::from([$(($k, $v),)*])
                .into_iter()
                .map(|(k, v)| (k, parse(v)))
                .collect()
        }
    }

    macro_rules! test {
        ($name:ident, $x:expr, $y:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(parse($x).unify(&parse($y)), $expected);
            }
        };
    }

    test!(atom_1, "a", "a", Empty);
    test!(atom_2, "a(1)", "a(1)", Empty);
    test!(conflict_a, "a(1, X)", "a(2, 3)", Failed);
    test!(conflict_b, "a(X, 1)", "a(2, 3)", Failed);
    test!(ok_lhs_1, "a(X)", "a(1)", NotEmpty(map!("X"=>"1")));
    test!(ok_lhs_2a, "a(X, 2)", "a(1, 2)", NotEmpty(map!("X"=>"1")));
    test!(ok_lhs_2b, "a(1, X)", "a(1, 2)", NotEmpty(map!("X"=>"2")));
    test!(ok_rhs_1, "a(1)", "a(X)", Failed);
    test!(ok_rhs_2a, "a(1, 2)", "a(X, 2)", Failed);
    test!(ok_rhs_2b, "a(1, 2)", "a(1, X)", Failed);
}
