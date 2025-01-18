import pandas as pd


def load_data():
    """Load the data from the CSV file."""
    return pd.read_csv("stats_output.csv")

flags = [
        "--compact-comparisons",
        "--compact-skip-edges",
        "--inline-assignment",
        "--inline-reachability",
        "--join-exclusive-edges",
        "--join-fork-prefixes",
        "--join-fork-suffixes",
        "--join-generators",
        "--merge-accesses",
        "--normalize-constants",
        "--propagate-constants",
        "--prune-singleton-types",
        "--prune-unreachable-nodes",
        "--prune-unused-bindings",
        "--prune-unused-constants",
        "--prune-unused-variables",
        "--skip-generator-comparisons",
        "--skip-self-assignments",
        "--skip-self-comparisons",
        "--skip-unused-tags"
    ]


def main():
    df = load_data()
    df['flags'] = df['flags'].apply(lambda x: [] if x == "none" else x.split(" "))

    best = df.groupby(['game', 'language']).apply(
        lambda group: group.loc[group['edges'].idxmin()]
    ).reset_index(drop=True)

    worst = df[df['flags'].apply(lambda x: len(x) == 0)]

    other = df[~df['flags'].apply(lambda x: len(x) == 0) & df['flags'].apply(lambda x: "--enable-all-optimizations" not in x and "--compact-skip-edges" in x)]

    most_impactful = other.groupby(['game', 'language']).apply(
        lambda group: group.loc[group['edges'].idxmax()]
    ).reset_index(drop=True)
    most_impactful['missing_flags'] = most_impactful['flags'].apply(lambda x: [flag for flag in flags if flag not in x])

    # For each game and language, show how most_impactful compares to the best in edge count
    comparison = pd.merge(most_impactful, best, on=['game', 'language'], suffixes=('_most_impactful', '_best'))
    comparison['edges_diff'] = comparison['edges_most_impactful'] - comparison['edges_best']
    comparison['edges_diff_percent'] = 100 - comparison['edges_best'] / comparison['edges_most_impactful'] * 100

    # rows where improvement > 0
    print(comparison[comparison['edges_diff'] > 0][['game', 'language', 'edges_diff', 'edges_diff_percent', 'missing_flags']].sort_values('edges_diff_percent', ascending=False).head(60))

if __name__ == "__main__":
    main()