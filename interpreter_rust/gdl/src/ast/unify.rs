use crate::ast::{AtomOrVariable, Term};
use std::iter::zip;

impl<Id: Clone + Ord> AtomOrVariable<Id> {
    pub fn unify<'a>(&'a self, other: &'a Self) -> Unification<'a, Id> {
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
    pub fn unify<'a>(&'a self, other: &'a Self) -> Unification<'a, Id> {
        use Term::{
            Base, Custom0, CustomN, Does, Goal, Init, Input, Legal, Next, Role, Terminal, True,
        };
        match (self, other) {
            (Base(x), Base(y)) => x.unify(y),
            (Custom0(xn), Custom0(yn)) => xn.unify(yn),
            (CustomN(xn, xa), CustomN(yn, ya)) if xn == yn => match (&xa[..], &ya[..]) {
                // Manually unrolled first few steps.
                ([a], [b]) => a.unify(b),
                ([a, b], [c, d]) => a.unify(c).merge(|| b.unify(d)),
                ([a, b, c], [d, e, f]) => a.unify(d).merge(|| b.unify(e).merge(|| c.unify(f))),
                (xa, ya) => {
                    if xa.len() == ya.len() {
                        let mut u = Unification::Empty;
                        for (x, y) in zip(xa, ya) {
                            u = u.merge(|| x.unify(y));
                            if u == Unification::Failed {
                                break;
                            }
                        }
                        u
                    } else {
                        Unification::Failed
                    }
                }
            },
            (Does(xr, xa), Does(yr, ya)) => xr.unify(yr).merge(|| xa.unify(ya)),
            (Goal(xr, xu), Goal(yr, yu)) => xr.unify(yr).merge(|| xu.unify(yu)),
            (Init(x), Init(y)) => x.unify(y),
            (Input(xr, xa), Input(yr, ya)) => xr.unify(yr).merge(|| xa.unify(ya)),
            (Legal(xr, xa), Legal(yr, ya)) => xr.unify(yr).merge(|| xa.unify(ya)),
            (Next(x), Next(y)) => x.unify(y),
            (Role(x), Role(y)) => x.unify(y),
            (Terminal, Terminal) => Unification::Empty,
            (True(x), True(y)) => x.unify(y),
            _ => Unification::Failed,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Unification<'a, Id> {
    Empty,
    Failed,
    NotEmpty(Vec<(&'a Id, &'a Id)>),
}

impl<Id> Unification<'_, Id> {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty | Self::Failed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

impl<'a, Id: Clone> Unification<'a, Id> {
    pub fn from_pairs(pairs: &'a [(Id, Id)]) -> Self {
        match pairs.is_empty() {
            false => Self::NotEmpty(pairs.iter().map(|(x, y)| (x, y)).collect()),
            true => Self::Empty,
        }
    }

    pub fn into_pairs(self) -> Vec<(Id, Id)> {
        match self {
            Self::Empty => vec![],
            Self::Failed => unreachable!(),
            Self::NotEmpty(pairs) => pairs
                .into_iter()
                .map(|(x, y)| (x.clone(), y.clone()))
                .collect(),
        }
    }
}

impl<Id: PartialEq> Unification<'_, Id> {
    pub fn is_subset(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::NotEmpty(xs), Self::NotEmpty(ys))
                if xs.len() <= ys.len() && xs.iter().all(|x| ys.contains(x))
        )
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

    pub fn merge(self, other: impl FnOnce() -> Self) -> Self {
        use Unification::{Empty, Failed, NotEmpty};
        match self {
            Empty => other(),
            Failed => Failed,
            NotEmpty(mut xs) => match other() {
                Empty => NotEmpty(xs),
                Failed => Failed,
                NotEmpty(ys) => {
                    for y in ys {
                        match xs.binary_search_by(|x| x.0.cmp(y.0)) {
                            Ok(index) if xs[index].1 != y.1 => return Failed,
                            Ok(_) => {}
                            Err(index) => xs.insert(index, y),
                        }
                    }

                    NotEmpty(xs)
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::Unification::*;
    use crate::ast::Term;

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
                assert_eq!(Term::from($x).unify(&Term::from($y)), $expected);
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
