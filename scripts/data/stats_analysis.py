import pandas as pd

def load_data():
    """Load the data from the CSV file."""
    return pd.read_csv("../collect/results/stats.csv")

def measure_impact_per_game_language(df):
    with_none = df[df['flags'].apply(lambda x: x == "none")]
    with_all = df[df['flags'].apply(lambda x: x == "--enable-all-optimizations")]
    other = df[df['flags'].apply(lambda x: x not in ["none", "--enable-all-optimizations", "--compact-skip-edges", "--prune-unreachable-nodes"])]

    most_impactful = other.groupby(['game', 'language']).apply(
        lambda group: group.loc[group['edges'].idxmax()]
    ).reset_index(drop=True)

    print(with_none.head(10))
    print(with_all.head(10))
    print(most_impactful.head(10))

def measure_impact_per_language(df):
    # drop column 'game' and take average per language and flags of all numeric columns
    df = df.drop(columns=['game'])
    df = df.groupby(['language', 'flags']).mean().reset_index()
    
    with_none = df[df['flags'].apply(lambda x: x == "none")]
    with_all = df[df['flags'].apply(lambda x: x == "--enable-all-optimizations")]
    other = df[df['flags'].apply(lambda x: x not in ["none", "--enable-all-optimizations", "--compact-skip-edges", "--prune-unreachable-nodes"])]

    most_impactful = other.groupby('language').apply(
        lambda group: group.loc[group['edges'].idxmax()]
    ).reset_index(drop=True)

    print(with_none.head(10))
    print(with_all.head(10))
    print(most_impactful.head(10))

def best_vs_worst_per_game(df):
    with_none = df[df['flags'].apply(lambda x: x == "none")]
    with_all = df[df['flags'].apply(lambda x: x == "--enable-all-optimizations")]

    merged = pd.merge(with_none, with_all, on=['game', 'language'], suffixes=('_none', '_all'))
    merged['nodes_diff_percentage'] = -(
        (merged['nodes_all'] - merged['nodes_none']) / merged['nodes_none']
    ) * 100
    merged['edges_diff_percentage'] = -(
        (merged['edges_all'] - merged['edges_none']) / merged['edges_none']
    ) * 100
    merged['state_size_diff_percentage'] = -(
        (merged['state_size_all'] - merged['state_size_none']) / merged['state_size_none']
    ) * 100
    merged['variables_diff_percentage'] = -(
        (merged['variables_all'] - merged['variables_none']) / merged['variables_none']
    ) * 100
    merged = merged[['language', 'nodes_diff_percentage', 'edges_diff_percentage', 'state_size_diff_percentage', 'variables_diff_percentage']]
    merged = merged.groupby('language').mean().reset_index()
    merged.to_csv("avg_impact.csv", index=False)



def main():
    df = load_data()
    df = df.drop(columns=['repeat_nodes','repeat_or_unique_nodes', 'unique_nodes', 'typedefs'])
    # Measure the impact of the flags
    best_vs_worst_per_game(df)

if __name__ == "__main__":
    main()