use crate::ast::*;
use crate::position::{Span as Position, *};
use nom_locate::LocatedSpan;
use std::rc::Rc;

type Span<'a> = LocatedSpan<&'a str>;

// impl<'a> From<(Span<'a>, EdgeLabel<'a>, EdgeName<'a>, EdgeName<'a>)> for Edge<'a> {
//     fn from(
//         (span, label, lhs, rhs): (Span<'a>, EdgeLabel<'a>, EdgeName<'a>, EdgeName<'a>),
//     ) -> Self {
//         Self {
//             span: span.into(),
//             label,
//             lhs,
//             rhs,
//         }
//     }
// }

// impl<'a> From<(Span<'a>, EdgeName<'a>, EdgeName<'a>, EdgeLabel<'a>)> for Edge<'a> {
//     fn from(
//         (span, lhs, rhs, label): (Span<'a>, EdgeName<'a>, EdgeName<'a>, EdgeLabel<'a>),
//     ) -> Self {
//         Self {
//             span: span.into(),
//             label,
//             lhs,
//             rhs,
//         }
//     }
// }

// impl<'a> From<(Span<'a>, Rc<Expression<'a>>, Rc<Expression<'a>>)> for EdgeLabel<'a> {
//     fn from((span, lhs, rhs): (Span<'a>, Rc<Expression<'a>>, Rc<Expression<'a>>)) -> Self {
//         Self::Assignment {
//             span: span.into(),
//             lhs,
//             rhs,
//         }
//     }
// }

// impl<'a> From<(Span<'a>, Rc<Expression<'a>>, bool, Rc<Expression<'a>>)> for EdgeLabel<'a> {
//     fn from(
//         (span, lhs, negated, rhs): (Span<'a>, Rc<Expression<'a>>, bool, Rc<Expression<'a>>),
//     ) -> Self {
//         Self::Comparison {
//             span: span.into(),
//             lhs,
//             rhs,
//             negated,
//         }
//     }
// }

// impl<'a> From<(Span<'a>, bool, EdgeName<'a>, EdgeName<'a>)> for EdgeLabel<'a> {
//     fn from((span, negated, lhs, rhs): (Span<'a>, bool, EdgeName<'a>, EdgeName<'a>)) -> Self {
//         Self::Reachability {
//             span: span.into(),
//             lhs,
//             rhs,
//             negated,
//         }
//     }
// }

// impl<'a> From<(Span<'a>)> for EdgeLabel<'a> {
//     fn from((span): Span<'a>) -> Self {
//         Self::Tag {
//             span: span.into(),
//             symbol: span.fragment(),
//         }
//     }
// }

// impl<'a> From<(Span<'a>, Vec<EdgeNamePart<'a>>)> for EdgeName<'a> {
//     fn from((span, parts): (Span<'a>, Vec<EdgeNamePart<'a>>)) -> Self {
//         Self {
//             span: span.into(),
//             parts,
//         }
//     }
// }

// impl<'a> From<(Span<'a>)> for EdgeName<'a> {
//     fn from(span: Span<'a>) -> Self {
//         Self::from((span, vec![EdgeNamePart::from(span)]))
//     }
// }

// impl<'a> From<(Span<'a>, &'a str, Rc<Type<'a>>)> for EdgeNamePart<'a> {
//     fn from((span, identifier, type_): (Span<'a>, &'a str, Rc<Type<'a>>)) -> Self {
//         Self::Binding {
//             span: span.into(),
//             identifier,
//             type_,
//         }
//     }
// }

// impl<'a> From<(Span<'a>)> for EdgeNamePart<'a> {
//     fn from(span: Span<'a>) -> Self {
//         Self::Literal {
//             span: span.into(),
//             identifier: span.fragment(),
//         }
//     }
// }

// impl<'a> From<(Span<'a>, Vec<&'a str>)> for Type<'a> {
//     fn from((span, identifiers): (Span<'a>, Vec<&'a str>)) -> Self {
//         Self::Set {
//             span: span.into(),
//             identifiers,
//         }
//     }
// }

// impl<'a> From<(Span<'a>)> for Type<'a> {
//     fn from(span: Span<'a>) -> Self {
//         Self::TypeReference {
//             span: span.into(),
//             identifier: span.fragment(),
//         }
//     }
// }

// impl<'a> From<(Span<'a>, &'a str, Rc<Type<'a>>)> for Typedef<'a> {
//     fn from((span, identifier, type_): (Span<'a>, &'a str, Rc<Type<'a>>)) -> Self {
//         Self {
//             span: span.into(),
//             identifier,
//             type_,
//         }
//     }
// }

// impl<'a> From<(Span<'a>)> for Value<'a> {
//     fn from((span, identifier): (Span<'a>, &'a str)) -> Self {
//         Self::Element {
//             span: span.into(),
//             identifier,
//         }
//     }
// }

// impl<'a> From<(Span<'a>, Vec<ValueEntry<'a>>)> for Value<'a> {
//     fn from((span, entries): (Span<'a>, Vec<ValueEntry<'a>>)) -> Self {
//         Self::Map {
//             span: span.into(),
//             entries,
//         }
//     }
// }

// impl<'a> From<(Span<'a>, Option<&'a str>, Rc<Value<'a>>)> for ValueEntry<'a> {
//     fn from((span, identifier, value): (Span<'a>, Option<&'a str>, Rc<Value<'a>>)) -> Self {
//         Self {
//             span: span.into(),
//             identifier,
//             value,
//         }
//     }
// }

// impl<'a> From<(Span<'a>, &'a str, Rc<Type<'a>>, Rc<Value<'a>>)> for Variable<'a> {
//     fn from(
//         (span, identifier, type_, default_value): (Span<'a>, &'a str, Rc<Type<'a>>, Rc<Value<'a>>),
//     ) -> Self {
//         Self {
//             span: span.into(),
//             default_value,
//             identifier,
//             type_,
//         }
//     }
// }

impl<'a>
    From<(
        Span<'a>,
        (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
        Span<'a>,
    )> for Constant<'a>
{
    fn from(
        (start, (identifier, type_, value), end): (
            Span<'a>,
            (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
            Span<'a>,
        ),
    ) -> Self {
        let span = Position::from((start, end));
        Self {
            span,
            identifier,
            type_,
            value,
        }
    }
}

impl<'a> From<Vec<EdgeNamePart<'a>>> for EdgeName<'a> {
    fn from(parts: Vec<EdgeNamePart<'a>>) -> Self {
        let (first, last) = (parts.first().unwrap(), parts.last().unwrap());
        let span = Position::new(first.start().clone(), last.end().clone());
        Self { span, parts }
    }
}

impl<'a> From<Identifier<'a>> for EdgeName<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::from(vec![EdgeNamePart::from(identifier)])
    }
}

impl<'a> From<Identifier<'a>> for EdgeNamePart<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::Literal {
            span: identifier.span().clone(),
            identifier,
        }
    }
}

impl<'a> From<(Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>)> for EdgeNamePart<'a> {
    fn from(
        (start, (identifier, type_), end): (Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>),
    ) -> Self {
        let span = Position::from((start, end));
        Self::Binding {
            span,
            identifier,
            type_,
        }
    }
}

impl<'a> From<(EdgeName<'a>, EdgeName<'a>, EdgeLabel<'a>, Span<'a>)> for Edge<'a> {
    fn from((lhs, rhs, label, end): (EdgeName<'a>, EdgeName<'a>, EdgeLabel<'a>, Span<'a>)) -> Self {
        let span = Position::from((end, end)).with_start(lhs.start().clone());
        Self {
            span,
            label,
            lhs,
            rhs,
        }
    }
}

impl<'a> From<(Span<'a>, Identifier<'a>)> for EdgeLabel<'a> {
    fn from((start, symbol): (Span<'a>, Identifier<'a>)) -> Self {
        let span = Position::from((start, start)).with_end(symbol.span().end);
        EdgeLabel::Tag { span, symbol }
    }
}

impl<'a> From<(Rc<Expression<'a>>, Rc<Expression<'a>>)> for EdgeLabel<'a> {
    fn from((lhs, rhs): (Rc<Expression<'a>>, Rc<Expression<'a>>)) -> Self {
        let span = Position::new(lhs.span().start, rhs.span().end);
        Self::Assignment { span, lhs, rhs }
    }
}

impl<'a> From<(Rc<Expression<'a>>, bool, Rc<Expression<'a>>)> for EdgeLabel<'a> {
    fn from((lhs, negated, rhs): (Rc<Expression<'a>>, bool, Rc<Expression<'a>>)) -> Self {
        let span = Position::new(lhs.span().start, rhs.span().end);
        Self::Comparison {
            span,
            lhs,
            rhs,
            negated,
        }
    }
}

impl<'a> From<(Span<'a>, EdgeName<'a>, EdgeName<'a>)> for EdgeLabel<'a> {
    fn from((tag, lhs, rhs): (Span<'a>, EdgeName<'a>, EdgeName<'a>)) -> Self {
        let negated = *tag.fragment() == "!";
        let span = Position::from((tag, tag)).with_end(rhs.span().end);
        Self::Reachability {
            span,
            lhs,
            rhs,
            negated,
        }
    }
}

impl<'a> From<Span<'a>> for Identifier<'a> {
    fn from(value: Span<'a>) -> Self {
        Identifier {
            span: Position::from(value),
            identifier: value.fragment(),
        }
    }
}

impl<'a> From<Identifier<'a>> for Type<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::TypeReference {
            span: identifier.span,
            identifier,
        }
    }
}

impl<'a> From<(Span<'a>, Vec<Identifier<'a>>, Span<'a>)> for Type<'a> {
    fn from((start, identifiers, end): (Span<'a>, Vec<Identifier<'a>>, Span<'a>)) -> Self {
        let span = Position::from((start, end));
        Self::Set { span, identifiers }
    }
}

impl<'a> From<(Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>)> for Typedef<'a> {
    fn from(
        (start, (identifier, type_), end): (Span<'a>, (Identifier<'a>, Rc<Type<'a>>), Span<'a>),
    ) -> Self {
        let span = Position::from((start, end));
        Self {
            span,
            identifier,
            type_,
        }
    }
}

impl<'a> From<(Span<'a>, Vec<ValueEntry<'a>>, Span<'a>)> for Value<'a> {
    fn from((start, entries, end): (Span<'a>, Vec<ValueEntry<'a>>, Span<'a>)) -> Self {
        let span = Position::from((start, end));
        Self::Map { span, entries }
    }
}

impl<'a> From<(Identifier<'a>)> for Value<'a> {
    fn from(identifier: Identifier<'a>) -> Self {
        Self::Element {
            span: identifier.span().clone(),
            identifier,
        }
    }
}

impl<'a> From<(Option<Identifier<'a>>, Rc<Value<'a>>)> for ValueEntry<'a> {
    fn from((identifier, value): (Option<Identifier<'a>>, Rc<Value<'a>>)) -> Self {
        let span = match identifier {
            Some(identifier) => Position::new(identifier.span.start, value.as_ref().span().end),
            None => value.as_ref().span().clone(),
        };
        Self {
            span,
            identifier,
            value,
        }
    }
}

impl<'a>
    From<(
        Span<'a>,
        (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
        Span<'a>,
    )> for Variable<'a>
{
    fn from(
        (start, (identifier, type_, value), end): (
            Span<'a>,
            (Identifier<'a>, Rc<Type<'a>>, Rc<Value<'a>>),
            Span<'a>,
        ),
    ) -> Self {
        let span = Position::from((start, end));
        Self {
            span,
            identifier,
            default_value: value,
            type_,
        }
    }
}
