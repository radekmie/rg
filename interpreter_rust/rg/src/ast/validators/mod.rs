mod check_assignments;
mod check_duplicated_names;
mod check_maps;
mod check_multiple_edges;
mod check_reachabilities;
mod check_tag_loops;
mod check_tag_variables;
mod check_types;
mod lint_reachabilities;

#[cfg(test)]
use super::{Error, ErrorReason, Game};
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
type Id = Arc<str>;

#[cfg(test)]
impl Game<Id> {
    #[allow(clippy::type_complexity)]
    pub fn test_linter(
        source: &str,
        expect: &[ErrorReason<Id>],
        fn_: Box<dyn FnOnce(&Self) -> Box<dyn Iterator<Item = Error<Id>> + '_>>,
    ) {
        let game = Self::test_parse_or_fail(source);
        let actual = fn_(&game).map(|error| error.reason).collect::<Vec<_>>();
        assert_eq!(actual, expect);
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn test_validator(
        source: &str,
        expect: Result<(), ErrorReason<Id>>,
        fn_: Box<dyn FnOnce(Self) -> Result<(), Error<Id>>>,
    ) {
        let game = Self::test_parse_or_fail(source);
        let actual = fn_(game).map_err(|error| error.reason);
        assert_eq!(actual, expect);
    }
}

#[cfg(test)]
mod test {
    #[macro_export]
    macro_rules! test_linter {
        ($fn:ident, $name:ident, $source:expr, $expect:expr) => {
            #[test]
            fn $name() {
                $crate::ast::Game::test_linter($source, $expect, Box::new(|x| Box::new(x.$fn())));
            }
        };
    }

    #[macro_export]
    macro_rules! test_validator {
        ($fn:ident, $name:ident, $source:expr, $expect:expr) => {
            #[test]
            fn $name() {
                $crate::ast::Game::test_validator($source, $expect, Box::new(|x| x.$fn()));
            }
        };
    }
}
