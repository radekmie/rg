use clap::Args;
use serde::Deserialize;

#[derive(Args, Deserialize)]
pub struct Flags {
    /// add type casts to all expressions
    #[arg(long)]
    #[serde(rename = "addExplicitCasts")]
    pub add_explicit_casts: bool,

    /// calculate missing @disjoint and @disjointExhaustive pragmas automatically
    #[arg(long)]
    #[serde(rename = "calculateDisjoints")]
    pub calculate_disjoints: bool,

    /// calculate missing @repeat pragmas automatically
    #[arg(long)]
    #[serde(rename = "calculateRepeats")]
    pub calculate_repeats: bool,

    /// calculate missing @simpleApply and @simpleApplyExhaustive pragmas automatically
    #[arg(long)]
    #[serde(rename = "calculateSimpleApply")]
    pub calculate_simple_apply: bool,

    /// calculate missing @tagIndex and @tagMaxIndex pragmas automatically
    #[arg(long)]
    #[serde(rename = "calculateTagIndexes")]
    pub calculate_tag_indexes: bool,

    /// calculate missing @unique pragmas automatically
    #[arg(long)]
    #[serde(rename = "calculateUniques")]
    pub calculate_uniques: bool,

    /// optimize selective comparisons with negations
    #[arg(long)]
    #[serde(rename = "compactComparisons")]
    pub compact_comparisons: bool,

    /// optimize automaton by compacting skip edges
    #[arg(long)]
    #[serde(rename = "compactSkipEdges")]
    pub compact_skip_edges: bool,

    /// expand generator nodes
    #[arg(long)]
    #[serde(rename = "expandGeneratorNodes")]
    pub expand_generator_nodes: bool,

    /// inline assignment when possible
    #[arg(long)]
    #[serde(rename = "inlineAssignment")]
    pub inline_assignment: bool,

    /// inline reachability when possible
    #[arg(long)]
    #[serde(rename = "inlineReachability")]
    pub inline_reachability: bool,

    /// joins multiedges with exclusive labels
    #[arg(long)]
    #[serde(rename = "joinExclusiveEdges")]
    pub join_exclusive_edges: bool,

    /// join paths with identical labels from the same node
    #[arg(long)]
    #[serde(rename = "joinForkPrefixes")]
    pub join_fork_prefixes: bool,

    /// join paths with identical labels leading to the same node
    #[arg(long)]
    #[serde(rename = "joinForkSuffixes")]
    pub join_fork_suffixes: bool,

    /// join generator nodes
    #[arg(long)]
    #[serde(rename = "joinGenerators")]
    pub join_generators: bool,

    /// mangle all user-defined symbols
    #[arg(long)]
    #[serde(rename = "mangleSymbols")]
    pub mangle_symbols: bool,

    /// merge nested accesses to constant maps
    #[arg(long)]
    #[serde(rename = "mergeAccesses")]
    pub merge_accesses: bool,

    /// normalize all constants so Maps appear only in the top level
    #[arg(long)]
    #[serde(rename = "normalizeConstants")]
    pub normalize_constants: bool,

    /// normalize all types so Arrow types appear only in type definitions and are at most one level deep
    #[arg(long)]
    #[serde(rename = "normalizeTypes")]
    pub normalize_types: bool,

    /// inline constants and skip obvious comparisons
    #[arg(long)]
    #[serde(rename = "propagateConstants")]
    pub propagate_constants: bool,

    /// prune singleton types (i.e., Set types with one element)
    #[arg(long)]
    #[serde(rename = "pruneSingletonTypes")]
    pub prune_singleton_types: bool,

    /// prune unreachable nodes
    #[arg(long)]
    #[serde(rename = "pruneUnreachableNodes")]
    pub prune_unreachable_nodes: bool,

    /// prune unused bindings from nodes
    #[arg(long)]
    #[serde(rename = "pruneUnusedBindings")]
    pub prune_unused_bindings: bool,

    /// prune unused constants
    #[arg(long)]
    #[serde(rename = "pruneUnusedConstants")]
    pub prune_unused_constants: bool,

    /// prune unused variables
    #[arg(long)]
    #[serde(rename = "pruneUnusedVariables")]
    pub prune_unused_variables: bool,

    /// reuse subautomatons when translating function calls (.hrg only)
    #[arg(long)]
    #[serde(rename = "reuseFunctions")]
    pub reuse_functions: bool,

    /// skips all comparisons to a generator (e.g., `x, y(t: T): t == null`)
    #[arg(long)]
    #[serde(rename = "skipGeneratorComparisons")]
    pub skip_generator_comparisons: bool,

    /// replaces all self assignments (e.g., `x = x`) with skip edges
    #[arg(long)]
    #[serde(rename = "skipSelfAssignments")]
    pub skip_self_assignments: bool,

    /// replaces all self comparisons (e.g., `x == x`) with skip edges
    #[arg(long)]
    #[serde(rename = "skipSelfComparisons")]
    pub skip_self_comparisons: bool,

    /// replaces all tags in reachability with skip edges
    #[arg(long)]
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
            join_generators: true,
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
            reuse_functions: true,
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
            join_generators: false,
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
            reuse_functions: false,
            skip_generator_comparisons: false,
            skip_self_assignments: false,
            skip_self_comparisons: false,
            skip_unused_tags: false,
        }
    }
}
