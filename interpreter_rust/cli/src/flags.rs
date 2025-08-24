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

    /// expand assignments with `*` to all possible values
    #[arg(long)]
    pub expand_assignment_any: bool,

    /// expand tag variable
    #[arg(long)]
    pub expand_tag_variable: bool,

    /// mangle all user-defined symbols
    #[arg(long)]
    pub mangle_symbols: bool,

    /// normalize all constants so Maps appear only in the top level
    #[arg(long)]
    pub normalize_constants: bool,

    /// normalize all types so Arrow types appear only in type definitions and are at most one level deep
    #[arg(long)]
    pub normalize_types: bool,

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

    /// optimize automaton by compacting reachability checks
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub compact_reachability: bool,

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

    /// prune self loops (i.e., non-modifying edges with the same lhs and rhs)
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub prune_self_loops: bool,

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

    /// reorders conditions in the automaton to optimize the execution.
    /// Can change the semantics of the game.
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub reorder_conditions: bool,

    /// skip tag edges marked with `@artificialTag`
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub skip_artificial_tags: bool,

    /// replaces all redundant tags with skip edges
    #[arg(
        long,
        help_heading = "Optimizations",
        conflicts_with = "enable_all_optimizations",
        default_value_if("enable_all_optimizations", "true", Some("true"))
    )]
    pub skip_redundant_tags: bool,

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

    /// calculate missing @iterator pragmas automatically
    #[arg(
        long,
        help_heading = "Pragmas",
        conflicts_with = "enable_all_pragmas",
        default_value_if("enable_all_pragmas", "true", Some("true"))
    )]
    pub calculate_iterators: bool,

    /// calculate missing @repeat and @unique pragmas automatically
    #[arg(
        long,
        help_heading = "Pragmas",
        conflicts_with = "enable_all_pragmas",
        default_value_if("enable_all_pragmas", "true", Some("true"))
    )]
    pub calculate_repeats_and_uniques: bool,

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
}

impl Flags {
    pub fn optimized() -> Self {
        Self {
            add_explicit_casts: false,
            calculate_disjoints: true,
            calculate_iterators: true,
            calculate_repeats_and_uniques: true,
            calculate_simple_apply: true,
            calculate_tag_indexes: true,
            compact_comparisons: true,
            compact_reachability: true,
            compact_skip_edges: true,
            enable_all_optimizations: true,
            enable_all_pragmas: true,
            expand_assignment_any: false,
            expand_tag_variable: false,
            inline_assignment: true,
            inline_reachability: true,
            join_exclusive_edges: true,
            join_fork_prefixes: true,
            join_fork_suffixes: true,
            mangle_symbols: false,
            merge_accesses: true,
            normalize_constants: true,
            normalize_types: true,
            propagate_constants: true,
            prune_self_loops: true,
            prune_singleton_types: true,
            prune_unreachable_nodes: true,
            prune_unused_constants: true,
            prune_unused_variables: true,
            reorder_conditions: true,
            skip_artificial_tags: true,
            skip_redundant_tags: true,
            skip_self_assignments: true,
            skip_self_comparisons: true,
            skip_unused_tags: true,
        }
    }
}
