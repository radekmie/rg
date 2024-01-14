use crate::ast::{AtomOrVariable, Term};
use std::iter::zip;

impl<Id: Clone + Ord> AtomOrVariable<Id> {
    pub fn unify<'a>(&'a self, other: &'a Self) -> Unification<Id> {
        use AtomOrVariable::{Atom, Variable};
        use Unification::{Empty, Failed, NotEmpty};
        match (self, other) {
            (Variable(x), Atom(y)) => NotEmpty(vec![(x, y)]),
            (x, y) if x == y => Empty,
            _ => Failed,
        }
    }
}

impl<Id: Clone + Ord> Term<Id> {
    pub fn unify<'a>(&'a self, other: &'a Self) -> Unification<Id> {
        use Term::{Base, Custom, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True};
        match (self, other) {
            (Base(x), Base(y)) => x.unify(y),
            (Custom(xn, xa), Custom(yn, ya)) if xa.is_empty() && ya.is_empty() => xn.unify(yn),
            (Custom(xn, xa), Custom(yn, ya)) if xn == yn => {
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

#[derive(Debug, Eq, PartialEq)]
pub enum Unification<'a, Id> {
    Empty,
    Failed,
    NotEmpty(Vec<(&'a Id, &'a Id)>),
}

impl<Id> Unification<'_, Id> {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty | Self::Failed)
    }
}

impl<Id: Ord> Unification<'_, Id> {
    pub fn get(&self, symbol: &Id) -> Option<&Id> {
        match self {
            Self::NotEmpty(mapping) => mapping
                .binary_search_by(|pair| pair.0.cmp(symbol))
                .ok()
                .map(|index| mapping[index].1),
            _ => None,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        use Unification::{Empty, Failed, NotEmpty};
        match (self, other) {
            (x, Empty) => x,
            (Empty, y) => y,
            (NotEmpty(mut xs), NotEmpty(ys)) => {
                for y in ys {
                    match xs.binary_search_by(|x| x.0.cmp(y.0)) {
                        Ok(index) if xs[index].1 != y.1 => return Failed,
                        Ok(_) => {}
                        Err(index) => xs.insert(index, y),
                    }
                }

                NotEmpty(xs)
            }
            _ => Failed,
        }
    }
}

impl<Id: Ord> FromIterator<Self> for Unification<'_, Id> {
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

    fn parse(input: &str) -> Term<&str> {
        all_consuming(term)(input).unwrap().1
    }

    macro_rules! map {
        ($($k:expr => $v:expr),* $(,)?) => {
            ([$(($k, $v),)*])
                .iter()
                .map(|(k, v)| (k, v))
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
    test!(rebind_1, "a(X, X)", "a(1, 2)", Failed);
    test!(rebind_2, "a(X, X)", "a(1, 1)", NotEmpty(map!("X"=>"1")));
}
