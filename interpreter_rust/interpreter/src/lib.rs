mod flags;
mod ist;

pub use crate::ist::Game;
pub use flags::Flags;
use hrg::parsing::parser::parse_with_errors as unsafe_parse_hrg;
use js_sys::{Array, Function};
use map_id::MapId;
use rand::thread_rng;
use rbg::parsing::parser::parse_with_errors as unsafe_parse_rbg;
use rg::ast::Game as GameAst;
use rg::parsing::parser::parse_with_errors as unsafe_parse_rg;
use serde_json::{from_str, json, to_string};
use std::collections::BTreeSet;
use std::sync::Arc;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

macro_rules! safe_parse {
    ($result:expr) => {{
        let (game, errors) = $result;
        if errors.is_empty() {
            let game = game.map_id(&mut |id| Arc::from(id.identifier.as_str()));
            Ok(game)
        } else {
            Err(errors
                .into_iter()
                .map(|error| format!("{error}"))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }};
}

type Id = Arc<str>;

fn analyze_gdl_inner(
    source: &str,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<GameAst<Id>, String> {
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

fn analyze_hrg_inner(
    source: &str,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<GameAst<Id>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    let hrg = safe_parse!(unsafe_parse_hrg(source))?;
    step!({ "kind": "ast", "language": "hrg", "value": hrg });

    let serialized = hrg.to_string();
    assert_eq!(safe_parse!(unsafe_parse_hrg(&serialized))?, hrg);
    step!({ "kind": "source", "language": "hrg", "value": serialized, "title": "formatted" });

    hrg_to_rg::hrg_to_rg(hrg).map_err(|error| error.to_string())
}

fn analyze_rbg_inner(
    source: &str,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<GameAst<Id>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    let rbg = safe_parse!(unsafe_parse_rbg(source))?;
    step!({ "kind": "ast", "language": "rbg", "value": rbg });

    let serialized = rbg.to_string();
    assert_eq!(safe_parse!(unsafe_parse_rbg(&serialized))?, rbg);
    step!({ "kind": "source", "language": "rbg", "value": serialized, "title": "formatted" });

    rbg_to_rg::rbg_to_rg(rbg).map_err(|error| error.to_string())
}

fn analyze_rg_inner(
    game_or_source: Result<GameAst<Id>, String>,
    flags: &Flags,
    #[allow(unused_variables)] // It's used only in non-WASM builds.
    verbose: bool,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<GameAst<Id>, String> {
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
            let mut stats_and_warnings = $game.to_stats();
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
        ($game:expr, $fn:ident) => {
            #[cfg(not(target_arch = "wasm32"))]
            let now = std::time::Instant::now();
            let result = $game.$fn();
            #[cfg(not(target_arch = "wasm32"))]
            if verbose {
                eprintln!("{} {:?}", stringify!($fn), now.elapsed());
            }
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
        Err(source) => (check!(safe_parse!(unsafe_parse_rg(&source))), source),
    };

    // Add AST for the original source
    step!({ "kind": "source", "language": "rg", "value": source, "title": "original" });
    step!({ "kind": "ast", "language": "rg", "value": game, "title": "original" });

    let serialized = game.to_string();
    assert_eq!(safe_parse!(unsafe_parse_rg(&serialized))?, game);
    step!({ "kind": "source", "language": "rg", "value": serialized, "title": "formatted" });

    // Builtins may not be required.
    let copy = game.clone();
    game_call!(game, add_builtins);
    if copy != game {
        game_step!(game, "add_builtins");
    }

    loop {
        game_call!(game, check_assignments);
        game_call!(game, check_duplicated_names);
        game_call!(game, check_maps);
        // game_call!(game, check_multiple_edges);
        game_call!(game, check_reachabilities);
        game_call!(game, check_types);

        let copy = game.clone();
        let mut restart = true;

        macro_rules! pass {
            ($fn:ident) => {
                if flags.$fn {
                    game_call!(game, $fn);
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
        pass!(inline_reachability);
        pass!(propagate_constants);
        pass!(merge_accesses);
        pass!(merge_bindings);

        // Compact the automaton.
        pass!(compact_comparisons);
        pass!(join_exclusive_edges);
        pass!(join_fork_prefixes);
        pass!(join_fork_suffixes);
        pass!(join_generators);
        pass!(skip_generator_comparisons);
        pass!(skip_self_assignments);
        pass!(skip_self_comparisons);
        pass!(skip_unused_tags);

        // Pruning.
        pass!(prune_singleton_types);
        pass!(prune_unreachable_nodes);
        pass!(prune_unused_bindings);
        pass!(prune_unused_constants);
        pass!(prune_unused_variables);

        // Normalization.
        pass!(normalize_types);
        pass!(normalize_constants);
        pass!(add_explicit_casts);

        // Expand generator nodes before calculating pragmas.
        pass!(expand_generator_nodes);

        // Point of no return -- these passes are forward-only.
        restart = false;

        // Pragmas.
        pass!(calculate_disjoints);
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

pub fn analyze_inner(
    source: String,
    extension: &str,
    flags: &Flags,
    verbose: bool,
    callback: &mut Option<impl FnMut(String)>,
) -> Result<GameAst<Id>, String> {
    let game_or_source = match extension {
        "hrg" => Ok(analyze_hrg_inner(&source, callback)?),
        "kif" => Ok(analyze_gdl_inner(&source, callback)?),
        "rbg" => Ok(analyze_rbg_inner(&source, callback)?),
        "rg" => Err(source),
        _ => return Err("Unknown game type: {extension}.".to_string()),
    };

    analyze_rg_inner(game_or_source, flags, verbose, callback)
}

#[wasm_bindgen(js_name = analyze)]
pub fn analyze(
    source: String,
    extension: &str,
    flags: &str,
    callback: &Function,
) -> Result<(), String> {
    console_error_panic_hook::set_once();
    analyze_inner(
        source,
        extension,
        &from_str::<Flags>(flags).unwrap(),
        false,
        &mut Some(|step| {
            callback
                .call1(&JsValue::null(), &JsValue::from(step))
                .unwrap();
        }),
    )?;
    Ok(())
}

#[wasm_bindgen(js_name = perf)]
pub fn perf(ast: &str, depth: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let game = Game::try_from(from_str(ast).map_err(|error| error.to_string())?)?.0;
    let (count, time) = game.perf(depth);
    callback
        .call2(&JsValue::null(), &count.into(), &time.into())
        .unwrap();
    Ok(())
}

#[wasm_bindgen(js_name = run)]
pub fn run(ast: &str, plays: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let (game, interner) = Game::try_from(from_str(ast).map_err(|error| error.to_string())?)?;
    let this = JsValue::null();
    let mut rng = thread_rng();
    game.run(
        &mut rng,
        &interner,
        plays,
        &Some(|lines: Vec<_>| {
            let lines = Array::from_iter(lines.into_iter().map(JsValue::from));
            callback.call1(&this, &lines).unwrap();
        }),
    )
}
