use rg::ast::{EdgeDeclaration, EdgeLabel, Expression, GameDeclaration};
use std::rc::Rc;

fn is_equal_reference<Id: PartialEq>(x: &Expression<Id>, y: &Expression<Id>) -> bool {
    use Expression::*;
    match (x, y) {
        (Cast { rhs: x, .. }, _) => is_equal_reference(x, y),
        (_, Cast { rhs: y, .. }) => is_equal_reference(x, y),
        (
            Access {
                lhs: x_lhs,
                rhs: x_rhs,
            },
            Access {
                lhs: y_lhs,
                rhs: y_rhs,
            },
        ) => is_equal_reference(x_lhs, y_lhs) && is_equal_reference(x_rhs, y_rhs),
        (Reference { identifier: x }, Reference { identifier: y }) => x == y,
        _ => false,
    }
}

fn is_self_assignment<Id: PartialEq>(edge_label: &EdgeLabel<Id>) -> bool {
    matches!(edge_label, EdgeLabel::Assignment { lhs, rhs } if is_equal_reference(lhs, rhs))
}

pub fn skip_self_assignments<Id: PartialEq>(game_declaration: &mut GameDeclaration<Id>) {
    for edge in &mut game_declaration.edges {
        if is_self_assignment(&edge.label) {
            *edge = Rc::new(EdgeDeclaration {
                label: Rc::new(EdgeLabel::Skip),
                lhs: edge.lhs.clone(),
                rhs: edge.rhs.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::skip_self_assignments::is_self_assignment;
    use rg::parser::edge_label;

    fn check(input: &str, expected: bool) {
        let result = edge_label(input).expect("Incorrect input.");
        assert_eq!(result.0, "");
        assert_eq!(is_self_assignment(&result.1), expected);
    }

    #[test]
    fn references() {
        check("x = x", true);
        check("x = y", false);
    }

    #[test]
    fn references_with_casts() {
        check("x = T(x)", true);
        check("T(x) = x", true);
        check("T(x) = T(x)", true);

        check("x = T(y)", false);
        check("T(x) = y", false);
        check("T(x) = T(y)", false);
    }

    #[test]
    fn accesses() {
        check("x[y] = x[y]", true);
        check("x[y] = z[y]", false);
        check("x[y] = x[z]", false);
    }

    #[test]
    fn accesses_with_casts() {
        check("x[y] = T(x[y])", true);
        check("T(x[y]) = x[y]", true);
        check("T(x[y]) = T(x[y])", true);

        check("x[y] = T(z[y])", false);
        check("T(x[y]) = z[y]", false);
        check("T(x[y]) = T(z[y])", false);

        check("x[y] = T(x[z])", false);
        check("T(x[y]) = x[z]", false);
        check("T(x[y]) = T(x[z])", false);
    }
}
