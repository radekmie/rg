import pandas as pd

def load_data():
    """Load the data from the CSV file."""
    return pd.read_csv("../collect/results/stats.csv")

def diff_metrics(df):
    with_none = df[df['flags'].apply(lambda x: x == "none")]
    with_all = df[df['flags'].apply(lambda x: x == "--enable-all-optimizations")]
    metrics = ['nodes', 'edges', 'assign_edges', 'comparison_edges', 'skip_edges', 'reachability_edges', 'tag_edges', 'average_branching', 'variables', 'constants', 'main_automaton_nodes', 'max_branching', 'reachability_subautomatons', 'state_size']
    merged = pd.merge(with_none, with_all, on=['game', 'language'], suffixes=('_none', '_all'))
   
    for metric in metrics:
        merged[f'{metric}_diff'] = (merged[f'{metric}_all'] - merged[f'{metric}_none']) / merged[f'{metric}_none'] * 100
        merged[f'{metric}_diff'] = merged[f'{metric}_diff'].fillna(0)
        merged[f'{metric}_diff'] = merged[f'{metric}_diff'].round(2)
        # fill nan values with 0
        merged = merged.drop(columns=[f'{metric}_all'])
        merged = merged.rename(columns={f'{metric}_none': metric})

    merged = merged.drop(columns=['flags_none', 'flags_all'])
    # sort columns alphabetically
    merged = merged.reindex(sorted(merged.columns), axis=1)
    # make game, language first two columns
    cols = ['game', 'language'] + [col for col in merged.columns if col not in ['game', 'language']]
    merged = merged[cols]
    return merged

def diff_metrics_avg(df):
    df = diff_metrics(df)
    df = df.drop(columns=['game'])
    df = df.groupby('language').mean().reset_index()
    df = df.round(2)
    df = df.set_index('language')
    return df


def main():
    df = load_data()
    df = df.drop(columns=['repeat_nodes','repeat_or_unique_nodes', 'unique_nodes', 'typedefs'])
    # Measure the impact of the flags
    diff_metrics(df).to_csv("results/diff_metrics.csv", index=False)
    diff_metrics_avg(df).to_csv("results/diff_metrics_avg.csv")

if __name__ == "__main__":
    main()