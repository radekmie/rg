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

fn safe_serialize_rg_ast(game: &RgGame<Arc<str>>) -> Result<String, String> {
    to_string(game).map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = analyzeHrg)]
pub fn analyze_hrg(source: &str, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let this = JsValue::null();
    macro_rules! step {
        ({ $($json:tt)+ }) => {{
            let step = json!({ $($json)+ });
            let message = to_string(&step).map_err(|error| error.to_string())?;
            callback.call1(&this, &JsValue::from(message)).unwrap();
        }};
    }

    let game = safe_parse_hrg_source(source)?;
    step!({ "kind": "ast", "language": "hrg", "value": game});
    let serialized = game.to_string();
    assert_eq!(safe_parse_hrg_source(&serialized)?, game);
    step!({ "kind": "source", "language": "hrg", "value": serialized, "title": "formatted"});

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

        // Pragmas (order doesn't matter).
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
    let flags = from_str::<Flags>(flags).unwrap();
    let this = JsValue::null();
    let callback = Some(|step| {
        callback.call1(&this, &JsValue::from(step)).unwrap();
    });
    analyze_rg_inner(source, &flags, callback)?;
    Ok(())
}

#[wasm_bindgen(js_name = parseGdl)]
pub fn parse_gdl(source: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let gdl = gdl::parser::game(source)
        .map_err(|error| error.to_string())?
        .1;
    let rg = gdl_to_rg::gdl_to_rg(&gdl);
    safe_serialize_rg_ast(&rg)
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
    game.run(&mut rng, &interner, plays, &|lines| {
        callback
            .call1(
                &this,
                &Array::from_iter(lines.into_iter().map(JsValue::from)),
            )
            .unwrap();
    });

    Ok(())
}

#[wasm_bindgen(js_name = serializeRg)]
pub fn serialize_rg(ast: &str) -> Result<String, String> {
    console_error_panic_hook::set_once();
    let game = safe_parse_rg_ast(ast)?;
    Ok(game.to_string())
}
