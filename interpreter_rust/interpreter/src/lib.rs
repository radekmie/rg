mod flags;
pub mod ist;

pub use flags::Flags;
use hrg::ast::Game as HrgGame;
use hrg::parsing::parser::parse_with_errors as unsafe_parse_hrg;
use ist::tools::{new_ist_interner, ISTInterner};
use js_sys::{Array, Function};
use map_id::MapId;
use rand::thread_rng;
use rg::ast::Game as RgGame;
use rg::parsing::parser::parse_with_errors as unsafe_parse_rg;
use serde_json::{from_str, json, to_string};
use std::sync::Arc;
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

pub fn prepare_ist(
    mut game: RgGame<Arc<str>>,
) -> Result<(ist::Game<ist::RuntimeId>, ISTInterner), String> {
    game.expand_generator_nodes()?;
    let mut interner = new_ist_interner();
    let game = ist::Game::from(game).map_id(&mut |id| interner.intern(id));
    Ok((game, interner))
}

fn safe_parse_rg_ast(ast: &str) -> Result<RgGame<Arc<str>>, String> {
    from_str::<RgGame<Arc<str>>>(ast).map_err(|error| error.to_string())
}

fn safe_parse_hrg_source(source: &str) -> Result<HrgGame<Arc<str>>, String> {
    let (game, errors) = unsafe_parse_hrg(source);
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
}

fn safe_parse_rg_source(source: &str) -> Result<RgGame<Arc<str>>, String> {
    let (game, errors) = unsafe_parse_rg(source);
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
}

pub fn analyze_gdl_inner(
    source: &str,
    mut callback: Option<impl FnMut(String)>,
) -> Result<RgGame<Arc<str>>, String> {
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

    step!({ "kind": "source", "language": "gdl", "value": gdl.as_infix().to_string(), "title": "infix" });
    step!({ "kind": "source", "language": "gdl", "value": gdl.as_prefix().to_string(), "title": "prefix" });

    let rg = gdl_to_rg::gdl_to_rg(&gdl);
    step!({ "kind": "ast", "language": "rg", "value": rg });

    Ok(rg)
}

#[wasm_bindgen(js_name = analyzeGdl)]
pub fn analyze_gdl(source: &str, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let this = JsValue::null();
    let callback = Some(|step| {
        callback.call1(&this, &JsValue::from(step)).unwrap();
    });
    analyze_gdl_inner(source, callback)?;
    Ok(())
}

pub fn analyze_hrg_inner(
    source: &str,
    reuse_functions: bool,
    mut callback: Option<impl FnMut(String)>,
) -> Result<RgGame<Arc<str>>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    let hrg = safe_parse_hrg_source(source)?;
    step!({ "kind": "ast", "language": "hrg", "value": hrg });

    let serialized = hrg.to_string();
    assert_eq!(safe_parse_hrg_source(&serialized)?, hrg);
    step!({ "kind": "source", "language": "hrg", "value": serialized, "title": "formatted" });

    let rg = hrg_to_rg::hrg_to_rg(hrg, reuse_functions).map_err(|error| error.to_string())?;
    step!({ "kind": "ast", "language": "rg", "value": rg });

    Ok(rg)
}

#[wasm_bindgen(js_name = analyzeHrg)]
pub fn analyze_hrg(source: &str, reuse_functions: bool, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let this = JsValue::null();
    let callback = Some(|step| {
        callback.call1(&this, &JsValue::from(step)).unwrap();
    });
    analyze_hrg_inner(source, reuse_functions, callback)?;
    Ok(())
}

pub fn analyze_rg_inner(
    source: &str,
    flags: &Flags,
    mut callback: Option<impl FnMut(String)>,
) -> Result<RgGame<Arc<str>>, String> {
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            if let Some(callback) = callback.as_mut() {
                let step = json!({ $($json)+ });
                callback(to_string(&step).map_err(|error| error.to_string())?);
            }
        }};
    }

    macro_rules! check {
        ($expr:expr) => {
            match $expr {
                Ok(expr) => expr,
                Err(error) => {
                    let error = error.to_string();
                    step!({ "kind": "error", "value": error });
                    return Err(error);
                },
            }
        };
    }

    macro_rules! game_step {
        ($game:expr, $title:expr) => {{
            step!({ "kind": "source", "language": "rg", "value": $game.to_string(), "title": $title });
            step!({ "kind": "ast", "language": "rg", "value": $game, "title": $title });
        }};
    }

    let mut game = check!(safe_parse_rg_source(source));

    // Add AST for the original source
    step!({ "kind": "ast", "language": "rg", "value": game, "title": "original" });

    // Builtins may not be required.
    let copy = game.clone();
    game.add_builtins()?;
    if copy != game {
        game_step!(game, "add_builtins");
    }

    loop {
        check!(game.check_assignments());
        check!(game.check_maps());
        // check!(game.check_multiple_edges());
        check!(game.check_reachabilities());
        check!(game.check_types());

        let copy = game.clone();

        macro_rules! pass {
            ($fn:ident) => {
                if flags.$fn {
                    check!(game.$fn());
                    if game != copy {
                        game_step!(game, stringify!($fn));
                        continue;
                    }
                }
            };
        }

        // Normalization.
        pass!(normalize_types);
        pass!(normalize_constants);
        pass!(add_explicit_casts);

        // Inlining.
        pass!(inline_assignment);
        pass!(inline_reachability);
        pass!(propagate_constants);
        pass!(merge_accesses);

        // Compact the automaton.
        pass!(compact_comparisons);
        pass!(compact_skip_edges);
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

        // Expand generator nodes before calculating pragmas.
        pass!(expand_generator_nodes);

        // Pragmas.
        pass!(calculate_disjoints);
        pass!(calculate_repeats);
        pass!(calculate_simple_apply);
        pass!(calculate_tag_indexes);
        pass!(calculate_uniques);

        // Mangling (last, to ensure best possible error messages before).
        pass!(mangle_symbols);

        break;
    }

    assert_eq!(check!(safe_parse_rg_source(&game.to_string())), game);

    step!({ "kind": "stats", "value": game.to_stats() });
    step!({ "kind": "graphviz", "value": game.to_graphviz() });
    Ok(game)
}

#[wasm_bindgen(js_name = analyzeRg)]
pub fn analyze_rg(source: &str, flags: &str, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let flags = from_str::<Flags>(flags).unwrap();
    let this = JsValue::null();
    let callback = Some(|step| {
        callback.call1(&this, &JsValue::from(step)).unwrap();
    });
    analyze_rg_inner(source, &flags, callback)?;
    Ok(())
}

#[wasm_bindgen(js_name = perfRg)]
pub fn perf_rg(ast: &str, depth: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let game = prepare_ist(safe_parse_rg_ast(ast)?)?.0;
    let this = JsValue::null();
    game.perf(depth, &|count, time| {
        callback.call2(&this, &count.into(), &time.into()).unwrap();
    });

    Ok(())
}

#[wasm_bindgen(js_name = runRg)]
pub fn run_rg(ast: &str, plays: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let (game, interner) = prepare_ist(safe_parse_rg_ast(ast)?)?;
    let this = JsValue::null();
    let mut rng = thread_rng();
    game.run(
        &mut rng,
        &interner,
        plays,
        &Some(|lines: Vec<_>| {
            callback
                .call1(
                    &this,
                    &Array::from_iter(lines.into_iter().map(JsValue::from)),
                )
                .unwrap();
        }),
    );

    Ok(())
}

#[wasm_bindgen(js_name = serializeRg)]
pub fn serialize_rg(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let game = safe_parse_rg_ast(ast)?;
    Ok(game.to_string())
}
