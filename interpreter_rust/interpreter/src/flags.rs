use serde::Deserialize;

#[derive(Deserialize)]
pub struct Flags {
    #[serde(rename = "addExplicitCasts")]
    pub add_explicit_casts: bool,
    #[serde(rename = "calculateDisjoints")]
    pub calculate_disjoints: bool,
    #[serde(rename = "calculateRepeats")]
    pub calculate_repeats: bool,
    #[serde(rename = "calculateSimpleApply")]
    pub calculate_simple_apply: bool,
    #[serde(rename = "calculateTagIndexes")]
    pub calculate_tag_indexes: bool,
    #[serde(rename = "calculateUniques")]
    pub calculate_uniques: bool,
    #[serde(rename = "compactComparisons")]
    pub compact_comparisons: bool,
    #[serde(rename = "compactSkipEdges")]
    pub compact_skip_edges: bool,
    #[serde(rename = "expandGeneratorNodes")]
    pub expand_generator_nodes: bool,
    #[serde(rename = "inlineAssignment")]
    pub inline_assignment: bool,
    #[serde(rename = "inlineReachability")]
    pub inline_reachability: bool,
    #[serde(rename = "joinExclusiveEdges")]
    pub join_exclusive_edges: bool,
    #[serde(rename = "joinForkPrefixes")]
    pub join_fork_prefixes: bool,
    #[serde(rename = "joinForkSuffixes")]
    pub join_fork_suffixes: bool,
    #[serde(rename = "mangleSymbols")]
    pub mangle_symbols: bool,
    #[serde(rename = "mergeAccesses")]
    pub merge_accesses: bool,
    #[serde(rename = "normalizeConstants")]
    pub normalize_constants: bool,
    #[serde(rename = "normalizeTypes")]
    pub normalize_types: bool,
    #[serde(rename = "propagateConstants")]
    pub propagate_constants: bool,
    #[serde(rename = "pruneSingletonTypes")]
    pub prune_singleton_types: bool,
    #[serde(rename = "pruneUnreachableNodes")]
    pub prune_unreachable_nodes: bool,
    #[serde(rename = "pruneUnusedBindings")]
    pub prune_unused_bindings: bool,
    #[serde(rename = "pruneUnusedConstants")]
    pub prune_unused_constants: bool,
    #[serde(rename = "pruneUnusedVariables")]
    pub prune_unused_variables: bool,
    #[serde(rename = "skipGeneratorComparisons")]
    pub skip_generator_comparisons: bool,
    #[serde(rename = "skipSelfAssignments")]
    pub skip_self_assignments: bool,
    #[serde(rename = "skipSelfComparisons")]
    pub skip_self_comparisons: bool,
    #[serde(rename = "skipUnusedTags")]
    pub skip_unused_tags: bool,
}

impl Flags {
    pub fn all() -> Self {
        Self {
            add_explicit_casts: true,
            calculate_disjoints: true,
            calculate_repeats: true,
            calculate_simple_apply: true,
            calculate_tag_indexes: true,
            calculate_uniques: true,
            compact_comparisons: true,
            compact_skip_edges: true,
            expand_generator_nodes: true,
            inline_assignment: true,
            inline_reachability: true,
            join_exclusive_edges: true,
            join_fork_prefixes: true,
            join_fork_suffixes: true,
            mangle_symbols: true,
            merge_accesses: true,
            normalize_constants: true,
            normalize_types: true,
            propagate_constants: true,
            prune_singleton_types: true,
            prune_unreachable_nodes: true,
            prune_unused_bindings: true,
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
            calculate_disjoints: false,
            calculate_repeats: false,
            calculate_simple_apply: false,
            calculate_tag_indexes: false,
            calculate_uniques: false,
            compact_comparisons: false,
            compact_skip_edges: false,
            expand_generator_nodes: false,
            inline_assignment: false,
            inline_reachability: false,
            join_exclusive_edges: false,
            join_fork_prefixes: false,
            join_fork_suffixes: false,
            mangle_symbols: false,
            merge_accesses: false,
            normalize_constants: false,
            normalize_types: false,
            propagate_constants: false,
            prune_singleton_types: false,
            prune_unreachable_nodes: false,
            prune_unused_bindings: false,
            prune_unused_constants: false,
            prune_unused_variables: false,
            skip_generator_comparisons: false,
            skip_self_assignments: false,
            skip_self_comparisons: false,
            skip_unused_tags: false,
        }
    }
}
