import pandas as pd

def load_data():
    """Load the data from the CSV file."""
    return pd.read_csv("../collect/results/runtime_stats1.csv")


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
    # Categorize edge: <50, <100, <500, <1000, <2000, <4000
    df['edge_category'] = pd.cut(df['edges'], bins=[-1, 50, 100, 500, 1000, 2000, 4000, float('inf')], labels=['<50', '<100', '<500', '<1000', '<2000', '<4000', '>4000'])
    # Add 'Average' edge_category for each row (all edges for that language)
    df_total = df.copy()
    df_total['edge_category'] = 'Average'
    df = pd.concat([df, df_total], ignore_index=True)
    # Add 'Average' language for each row (all languages for that edge_category)
    df_lang_total = df.copy()
    df_lang_total['language'] = 'Average'
    df = pd.concat([df, df_lang_total], ignore_index=True)
    df = df.drop(columns=['game', 'edges'])
    df = df.groupby(['language', 'edge_category']).mean().reset_index()
    df = df.round(2)
    return df

def avg_per_edges_total(df):
    df = avg_per_edges(df)
    # Pivot to have row per lang, with edge_category as columns
    df = df.pivot(index='language', columns='edge_category', values='change')
    # Sort rows by language
    df = df.sort_index(ascending=False)
    labels=['<50', '<100', '<500', '<1000', '<2000', '<4000', '>4000', 'Average']
    # Sort columns based on labels
    df = df.round(1)
    df = df.reindex(columns=labels)
    return df

def main():
    df = load_data()
    df = df[df['language'] != 'rg']
    avg_per_edges_total(df).to_csv("results/plays_grouped_total.csv")

if __name__ == "__main__":
    main()