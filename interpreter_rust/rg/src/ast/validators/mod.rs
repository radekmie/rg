mod check_assignments;
mod check_duplicated_names;
mod check_maps;
mod check_multiple_edges;
mod check_reachabilities;
mod check_tag_variables;
mod check_types;
mod lint_reachabilities;

#[cfg(test)]
mod test {
    #[macro_export]
    macro_rules! test_linter {
        ($fn:ident, $name:ident, $source:expr, $expect:expr) => {
            #[test]
            fn $name() {
                use map_id::MapId;
                use std::sync::Arc;
                use $crate::ast::ErrorReason;
                use $crate::parsing::parser::parse_with_errors;

                let (game, errors) = parse_with_errors($source);
                assert!(errors.is_empty(), "Parse errors: {errors:?}");
                let game = game.map_id(&mut |id| Arc::from(id.identifier.as_str()));

                let actual = game.$fn().map(|error| error.reason).collect::<Vec<_>>();
                let expect: Vec<ErrorReason<Arc<str>>> = $expect;

                assert_eq!(actual, expect);
            }
        };
    }

    #[macro_export]
    macro_rules! test_validator {
        ($fn:ident, $name:ident, $source:expr, $expect:expr) => {
            #[test]
            fn $name() {
                use map_id::MapId;
                use std::sync::Arc;
                use $crate::ast::ErrorReason;
                use $crate::parsing::parser::parse_with_errors;

                let (game, errors) = parse_with_errors($source);
                assert!(errors.is_empty(), "Parse errors: {errors:?}");
                let game = game.map_id(&mut |id| Arc::from(id.identifier.as_str()));

                let actual = game.$fn();
                let expect: Result<(), ErrorReason<Arc<str>>> = $expect;

                assert_eq!(actual.map_err(|error| error.reason), expect);
            }
        };
    }
}
