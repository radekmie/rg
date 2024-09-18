pub mod ist;

use hrg::ast::Game as HrgGame;
use hrg::parsing::parser::parse_with_errors as unsafe_parse_hrg;
use ist::tools::{new_ist_interner, ISTInterner};
use js_sys::{Array, Function};
use map_id::MapId;
use rand::thread_rng;
use rg::ast::Game as RgGame;
use rg::parsing::parser::parse_with_errors as unsafe_parse_rg;
use serde::Deserialize;
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

pub fn safe_parse_rg_ast(ast: &str) -> Result<RgGame<Arc<str>>, String> {
    from_str::<RgGame<Arc<str>>>(ast).map_err(|error| error.to_string())
}

pub fn safe_parse_hrg_source(source: &str) -> Result<HrgGame<Arc<str>>, String> {
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

pub fn safe_parse_rg_source(source: &str) -> Result<RgGame<Arc<str>>, String> {
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

pub fn safe_serialize_rg_ast(game: &RgGame<Arc<str>>) -> Result<String, String> {
    to_string(game).map_err(|error| error.to_string())
}

#[derive(Deserialize)]
pub struct Flags {
    #[serde(rename = "addExplicitCasts")]
    add_explicit_casts: bool,
    #[serde(rename = "calculateRepeats")]
    calculate_repeats: bool,
    #[serde(rename = "calculateSimpleApply")]
    calculate_simple_apply: bool,
    #[serde(rename = "calculateTagIndexes")]
    calculate_tag_indexes: bool,
    #[serde(rename = "calculateUniques")]
    calculate_uniques: bool,
    #[serde(rename = "compactSkipEdges")]
    compact_skip_edges: bool,
    #[serde(rename = "compactComparisons")]
    compact_comparisons: bool,
    #[serde(rename = "expandGeneratorNodes")]
    expand_generator_nodes: bool,
    #[serde(rename = "inlineReachability")]
    inline_reachability: bool,
    #[serde(rename = "inlineAssignment")]
    inline_assignment: bool,
    #[serde(rename = "joinForkPrefixes")]
    join_fork_prefixes: bool,
    #[serde(rename = "joinForkSuffixes")]
    join_fork_suffixes: bool,
    #[serde(rename = "mangleSymbols")]
    mangle_symbols: bool,
    #[serde(rename = "normalizeTypes")]
    normalize_types: bool,
    #[serde(rename = "pruneSingletonTypes")]
    prune_singleton_types: bool,
    #[serde(rename = "pruneUnreachableNodes")]
    prune_unreachable_nodes: bool,
    #[serde(rename = "pruneUnusedConstants")]
    prune_unused_constants: bool,
    #[serde(rename = "pruneUnusedVariables")]
    prune_unused_variables: bool,
    #[serde(rename = "skipGeneratorComparisons")]
    skip_generator_comparisons: bool,
    #[serde(rename = "skipSelfAssignments")]
    skip_self_assignments: bool,
    #[serde(rename = "skipSelfComparisons")]
    skip_self_comparisons: bool,
    #[serde(rename = "skipUnusedTags")]
    skip_unused_tags: bool,
}

impl Flags {
    pub fn all() -> Self {
        Self {
            add_explicit_casts: true,
            calculate_repeats: true,
            calculate_simple_apply: true,
            calculate_tag_indexes: true,
            calculate_uniques: true,
            compact_comparisons: true,
            compact_skip_edges: true,
            expand_generator_nodes: true,
            inline_assignment: true,
            inline_reachability: true,
            join_fork_prefixes: true,
            join_fork_suffixes: true,
            mangle_symbols: true,
            normalize_types: true,
            prune_singleton_types: true,
            prune_unreachable_nodes: true,
            prune_unused_constants: true,
            prune_unused_variables: true,
            skip_generator_comparisons: true,
            skip_self_assignments: true,
            skip_self_comparisons: true,
            skip_unused_tags: true,
        }
    }

    pub fn none() -> Self {
        Self {
            add_explicit_casts: false,
            calculate_repeats: false,
            calculate_simple_apply: false,
            calculate_tag_indexes: false,
            calculate_uniques: false,
            compact_comparisons: false,
            compact_skip_edges: false,
            expand_generator_nodes: false,
            inline_assignment: false,
            inline_reachability: false,
            join_fork_prefixes: false,
            join_fork_suffixes: false,
            mangle_symbols: false,
            normalize_types: false,
            prune_singleton_types: false,
            prune_unreachable_nodes: false,
            prune_unused_constants: false,
            prune_unused_variables: false,
            skip_generator_comparisons: false,
            skip_self_assignments: false,
            skip_self_comparisons: false,
            skip_unused_tags: false,
        }
    }
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

        pass!(normalize_types);
        pass!(skip_self_assignments);
        pass!(skip_self_comparisons);
        pass!(skip_unused_tags);
        pass!(skip_generator_comparisons);
        pass!(compact_skip_edges);
        pass!(add_explicit_casts);
        pass!(expand_generator_nodes);
        pass!(join_fork_prefixes);
        pass!(join_fork_suffixes);
        pass!(compact_comparisons);
        pass!(inline_reachability);
        pass!(inline_assignment);
        pass!(prune_singleton_types);
        pass!(prune_unreachable_nodes);
        pass!(prune_unused_variables);
        pass!(prune_unused_constants);
        pass!(mangle_symbols);
        pass!(calculate_repeats);
        pass!(calculate_simple_apply);
        pass!(calculate_tag_indexes);
        pass!(calculate_uniques);

        break;
    }

    assert_eq!(check!(safe_parse_rg_source(&game.to_string())), game);

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
    game.perf(depth, &|count| {
        callback.call1(&this, &count.into()).unwrap();
    });

    Ok(())
}

#[wasm_bindgen(js_name = runRg)]
pub fn run_rg(ast: &str, plays: usize, callback: &Function) -> Result<(), String> {
    console_error_panic_hook::set_once();
    let (game, interner) = prepare_ist(safe_parse_rg_ast(ast)?)?;
    let this = JsValue::null();
    let mut rng = thread_rng();
    game.run(&mut rng, plays, &|(plays, moves, turns, goals)| {
        let mut sorted_goals = goals
            .iter()
            .map(|(value, count)| (value.map_id(&mut |id| interner.recall(id).unwrap()), count))
            .collect::<Vec<_>>();
        sorted_goals.sort();
        callback
            .apply(
                &this,
                &Array::from_iter::<[&JsValue; 4]>([
                    &plays.into(),
                    &moves.into(),
                    &turns.into(),
                    &sorted_goals
                        .into_iter()
                        .map(|(value, count)| {
                            format!(
                                "    {:5.2}% of {}",
                                *count as f32 * 1e2 / plays as f32,
                                value
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .into(),
                ]),
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
