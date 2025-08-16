import pandas as pd

def load_data():
    """Load the data from the CSV file."""
    return pd.read_csv("../collect/results/runtime_stats.csv")


def transform_data(df):
    # For each game and language, create column plays_none and plays_optimized
    df = df.pivot_table(index=['game', 'language'],
                        columns='flags',
                        values='plays',
                        aggfunc='sum').reset_index()
    df = df.rename(columns={'none': 'plays_none', '--enable-all-optimizations': 'plays_optimized'})
    return df

def with_stats(df):
    df = transform_data(df) 
    stats_df = pd.read_csv("../collect/results/stats.csv")
    stats_df = stats_df[stats_df['flags'] == 'none']
    stats_df = stats_df[['game', 'language', 'edges']]
    df = df.merge(stats_df, on=['game', 'language'], how='left')
    df['change'] = (df['plays_optimized'] - df['plays_none']) / df['plays_none'] * 100
    df = df.drop(columns=['plays_none', 'plays_optimized'])
    df = df.round(2)
    return df

def avg_per_edges(df):
    df = with_stats(df)
    # Categorize edge: <50, >50, >100, >500, >1000, >2000
    df['edge_category'] = pd.cut(df['edges'], bins=[-1, 50, 100, 500, 1000, 2000, float('inf')], labels=['<50', '50-100', '>100', '>500', '>1000', '>2000'])
    df = df.drop(columns=['game', 'edges'])
    df = df.groupby(['language', 'edge_category']).mean().reset_index()
    return df

def main():
    df = load_data()
    # transform_data(df).to_csv("results/plays.csv", index=False)
    avg_per_edges(df).to_csv("results/plays_grouped.csv", index=False)

if __name__ == "__main__":
    main()