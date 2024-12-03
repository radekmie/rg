use clap::Args;
use serde::Deserialize;

#[derive(Args, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    //
    // Options
    //
    /// add type casts to all expressions
    #[arg(long)]
    pub add_explicit_casts: bool,

    /// expand generator nodes
    #[arg(long)]
    pub expand_generator_nodes: bool,

    /// mangle all user-defined symbols
    #[arg(long)]
    pub mangle_symbols: bool,

    /// normalize all constants so Maps appear only in the top level
    #[arg(long)]
    pub normalize_constants: bool,

    /// normalize all types so Arrow types appear only in type definitions and are at most one level deep
    #[arg(long)]
    pub normalize_types: bool,

    /// reuse subautomatons when translating function calls (.hrg only)
    #[arg(long)]
    pub reuse_functions: bool,

    //
    // Optimizations
    //
    /// enables all optimization flags
    #[arg(long, help_heading = "Optimizations", display_order = 0)]
    #[serde(skip)]
    enable_all_optimizations: bool,

    /// optimize selective comparisons with negations
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub compact_comparisons: bool,

    /// optimize automaton by compacting skip edges
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub compact_skip_edges: bool,

    /// inline assignment when possible
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub inline_assignment: bool,

    /// inline reachability when possible
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub inline_reachability: bool,

    /// joins multiedges with exclusive labels
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub join_exclusive_edges: bool,

    /// join paths with identical labels from the same node
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub join_fork_prefixes: bool,

    /// join paths with identical labels leading to the same node
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub join_fork_suffixes: bool,

    /// join generator nodes
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub join_generators: bool,

    /// merge nested accesses to constant maps
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub merge_accesses: bool,

    /// inline constants and skip obvious comparisons
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub propagate_constants: bool,

    /// prune singleton types (i.e., Set types with one element)
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub prune_singleton_types: bool,

    /// prune unreachable nodes
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub prune_unreachable_nodes: bool,

    /// prune unused bindings from nodes
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub prune_unused_bindings: bool,

    /// prune unused constants
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub prune_unused_constants: bool,

    /// prune unused variables
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub prune_unused_variables: bool,

    /// skips all comparisons to a generator (e.g., `x, y(t: T): t == null`)
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub skip_generator_comparisons: bool,

    /// replaces all self assignments (e.g., `x = x`) with skip edges
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub skip_self_assignments: bool,

    /// replaces all self comparisons (e.g., `x == x`) with skip edges
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub skip_self_comparisons: bool,

    /// replaces all tags in reachability with skip edges
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub skip_unused_tags: bool,

    //
    // Pragmas
    //
    /// enables all pragma flags
    #[arg(long, help_heading = "Pragmas", display_order = 0)]
    #[serde(skip)]
    enable_all_pragmas: bool,

    /// calculate missing @disjoint and @disjointExhaustive pragmas automatically
    #[arg(
        long,
        help_heading = "Pragmas",
        conflicts_with = "enable_all_pragmas",
        default_value_if("enable_all_pragmas", "true", Some("true"))
    )]
    pub calculate_disjoints: bool,

    /// calculate missing @repeat pragmas automatically
    #[arg(
        long,
        help_heading = "Pragmas",
        conflicts_with = "enable_all_pragmas",
        default_value_if("enable_all_pragmas", "true", Some("true"))
    )]
    pub calculate_repeats: bool,

    /// calculate missing @simpleApply and @simpleApplyExhaustive pragmas automatically
    #[arg(
        long,
        help_heading = "Pragmas",
        conflicts_with = "enable_all_pragmas",
        default_value_if("enable_all_pragmas", "true", Some("true"))
    )]
    pub calculate_simple_apply: bool,

    /// calculate missing @tagIndex and @tagMaxIndex pragmas automatically
    #[arg(
        long,
        help_heading = "Pragmas",
        conflicts_with = "enable_all_pragmas",
        default_value_if("enable_all_pragmas", "true", Some("true"))
    )]
    pub calculate_tag_indexes: bool,

    /// calculate missing @unique pragmas automatically
    #[arg(
        long,
        help_heading = "Pragmas",
        conflicts_with = "enable_all_pragmas",
        default_value_if("enable_all_pragmas", "true", Some("true"))
    )]
    pub calculate_uniques: bool,
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
            ..Self::default()
        }
    }

    pub fn none() -> Self {
        Self::default()
    }
}
