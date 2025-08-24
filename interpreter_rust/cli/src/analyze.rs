use crate::flags::Flags;
use hrg::parsing::parser::parse_with_errors as parse_hrg;
use map_id::MapId;
use rbg::parsing::parser::parse_with_errors as parse_rbg;
use rg::ast::Game;
use rg::parsing::parser::parse_with_errors as parse_rg;
use serde_json::{json, to_string};
use std::collections::BTreeSet;
use std::sync::Arc;
use utils::{Identifier, ParserError};

type Id = Arc<str>;

fn as_id(identifier: &Identifier) -> Id {
    Arc::from(identifier.identifier.as_str())
}

fn parse<T>(parser: fn(&str) -> (T, Vec<ParserError>), source: &str) -> Result<T, String> {
    let (game, errors) = parser(source);
    if errors.is_empty() {
        Ok(game)
    } else {
        Err(errors
            .into_iter()
            .map(|error| format!("{error}"))
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

fn analyze_gdl(
    source: &str,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<Game<Id>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    let gdl = gdl::parser::game(source)
        .map_err(|error| error.to_string())?
        .1;

    step!({ "kind": "source", "language": "gdl", "value": gdl.as_prefix().to_string(), "title": "formatted (prefix)" });
    step!({ "kind": "source", "language": "gdl", "value": gdl.as_infix().to_string(), "title": "formatted (infix)" });

    Ok(gdl_to_rg::gdl_to_rg(&gdl))
}

fn analyze_hrg(
    source: &str,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<Game<Id>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    let hrg = parse(parse_hrg, source)?.map_id(&mut as_id);
    step!({ "kind": "ast", "language": "hrg", "value": hrg });

    let serialized = hrg.to_string();
    assert_eq!(parse(parse_hrg, &serialized)?.map_id(&mut as_id), hrg);
    step!({ "kind": "source", "language": "hrg", "value": serialized, "title": "formatted" });

    hrg_to_rg::hrg_to_rg(hrg).map_err(|error| error.to_string())
}

fn analyze_rbg(
    source: &str,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<Game<Id>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    let rbg = parse(parse_rbg, source)?.map_id(&mut as_id);
    step!({ "kind": "ast", "language": "rbg", "value": rbg });

    let serialized = rbg.to_string();
    assert_eq!(parse(parse_rbg, &serialized)?.map_id(&mut as_id), rbg);
    step!({ "kind": "source", "language": "rbg", "value": serialized, "title": "formatted" });

    rbg_to_rg::rbg_to_rg(rbg).map_err(|error| error.to_string())
}

fn analyze_rg(
    game_or_source: Result<Game<Id>, String>,
    flags: &Flags,
    #[allow(unused_variables)] // It's used only in non-WASM builds.
    verbose: bool,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<Game<Id>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    macro_rules! add_game_stats {
        ($game:expr) => {
            // TODO: Should warnings be a separate step?
            let mut stats_and_warnings = $game.to_stats().to_string();
            let mut warnings = BTreeSet::new();
            warnings.extend($game.lint_reachabilities());
            if !warnings.is_empty() {
                stats_and_warnings.push_str("warnings:\n");
                for warning in warnings {
                    stats_and_warnings.push_str("  ");
                    stats_and_warnings.push_str(&warning.to_string());
                    stats_and_warnings.push('\n');
                }
            }

            step!({ "kind": "stats", "value": stats_and_warnings });
            step!({ "kind": "graphviz", "value": $game.to_graphviz() });
        }
    }

    macro_rules! check {
        ($expr:expr) => { check!($expr, {}) };
        ($expr:expr, $on_error:stmt) => {
            match $expr {
                Ok(expr) => expr,
                Err(error) => {
                    $on_error
                    let error = error.to_string();
                    step!({ "kind": "error", "value": error });
                    return Err(error);
                },
            }
        };
    }

    macro_rules! game_call {
        ($game:expr, $copy:expr, $fn:ident) => {
            let result = if cfg!(target_arch = "wasm32") || !verbose {
                $game.$fn()
            } else {
                let now = std::time::Instant::now();
                let result = $game.$fn();
                let elapsed = now.elapsed().as_micros();
                let changed = $game != $copy;
                eprintln!("{} {} {}", stringify!($fn), elapsed, changed);
                result
            };
            check!(result, add_game_stats!($game))
        };
    }

    macro_rules! game_step {
        ($game:expr, $title:expr) => {{
            step!({ "kind": "source", "language": "rg", "value": $game.to_string(), "title": $title });
            step!({ "kind": "ast", "language": "rg", "value": $game, "title": $title });
        }};
    }

    let (mut game, source) = match game_or_source {
        Ok(game) => {
            let source = game.to_string();
            (game, source)
        }
        Err(source) => (check!(parse(parse_rg, &source)).map_id(&mut as_id), source),
    };

    // Add AST for the original source
    step!({ "kind": "source", "language": "rg", "value": source, "title": "original" });
    step!({ "kind": "ast", "language": "rg", "value": game, "title": "original" });

    let serialized = game.to_string();
    assert_eq!(parse(parse_rg, &serialized)?.map_id(&mut as_id), game);
    step!({ "kind": "source", "language": "rg", "value": serialized, "title": "formatted" });

    // Builtins may not be required.
    let copy = game.clone();
    game_call!(game, copy, add_builtins);
    if copy != game {
        game_step!(game, "add_builtins");
    }

    loop {
        let copy = game.clone();
        let mut restart = true;

        game_call!(game, copy, check_assignments);
        game_call!(game, copy, check_duplicated_names);
        game_call!(game, copy, check_maps);
        // game_call!(game, copy, check_multiple_edges);
        game_call!(game, copy, check_reachabilities);
        game_call!(game, copy, check_tag_loops);
        game_call!(game, copy, check_tag_variables);
        game_call!(game, copy, check_types);

        macro_rules! pass {
            ($fn:ident) => {
                if flags.$fn {
                    game_call!(game, copy, $fn);
                    if game != copy {
                        game_step!(game, stringify!($fn));
                        if restart {
                            continue;
                        }
                    }
                }
            };
        }

        // Compact skip edges after every transformation.
        pass!(compact_skip_edges);

        // Inlining.
        pass!(inline_assignment);
        pass!(compact_reachability);
        pass!(inline_reachability);
        pass!(propagate_constants);
        pass!(merge_accesses);

        // Compact the automaton.
        pass!(reorder_conditions);
        pass!(compact_comparisons);
        pass!(join_exclusive_edges);
        pass!(join_fork_prefixes);
        pass!(join_fork_suffixes);
        pass!(skip_self_assignments);
        pass!(skip_self_comparisons);
        pass!(skip_unused_tags);
        pass!(skip_redundant_tags);

        // Pruning.
        pass!(prune_self_loops);
        pass!(prune_singleton_types);
        pass!(prune_unreachable_nodes);
        pass!(prune_unused_constants);
        pass!(prune_unused_variables);

        // Normalization.
        pass!(normalize_types);
        pass!(normalize_constants);
        pass!(add_explicit_casts);

        // Expand generator nodes before calculating pragmas.
        pass!(expand_assignment_any);
        pass!(expand_tag_variable);

        // Point of no return -- these passes are forward-only.
        restart = false;

        // Pragmas.
        pass!(calculate_disjoints);
        pass!(calculate_iterators);
        pass!(calculate_repeats_and_uniques);
        pass!(skip_artificial_tags);
        pass!(calculate_simple_apply);
        pass!(calculate_tag_indexes);

        // Mangling (last, to ensure best possible error messages before).
        pass!(mangle_symbols);

        break;
    }

    add_game_stats!(game);
    Ok(game)
}

pub fn analyze(
    source: String,
    extension: &str,
    flags: &Flags,
    verbose: bool,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<Game<Id>, String> {
    let game_or_source = match extension {
        "hrg" => Ok(analyze_hrg(&source, callback)?),
        "kif" => Ok(analyze_gdl(&source, callback)?),
        "rbg" => Ok(analyze_rbg(&source, callback)?),
        "rg" => Err(source),
        _ => return Err("Unknown game type: {extension}.".to_string()),
    };

    analyze_rg(game_or_source, flags, verbose, callback)
}
