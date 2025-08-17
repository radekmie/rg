import pandas as pd

def load_data():
    """Load the data from the CSV file."""
    return pd.read_csv("../collect/results/transforms.csv")



def with_stats(df):
    df = df[df['flags'] == '--enable-all-optimizations']
    stats_df = pd.read_csv("../collect/results/stats.csv")
    stats_df = stats_df[stats_df['flags'] == 'none']
    stats_df = stats_df[['game', 'language', 'edges']]
    print(stats_df.sort_values(by=['language', 'edges'], ascending=False).head(50))
    df['total_time'] = df['total_time'] / 1000
    df = df.drop(columns=['flags', 'changed', 'transform', 'count', 'changed_time'])
    df = df.groupby(['game', 'language']).sum().reset_index()
    df = df.merge(stats_df, on=['game', 'language'], how='left')
    return df

def avg_per_edges(df):
    df = with_stats(df)
    # Categorize edge: <50, >50, >100, >500, >1000, >2000
    df['edge_category'] = pd.cut(df['edges'], bins=[-1, 50, 100, 500, 1000, 2000, 4000, float('inf')], labels=['<50', '>50', '>100', '>500', '>1000', '>2000', '>4000'])
    # Add edge_category Total with everything
    # Add 'Total' edge_category for each row (all edges for that language)
    df_total = df.copy()
    df_total['edge_category'] = 'Total'
    df = pd.concat([df, df_total], ignore_index=True)
    # Add 'Total' language for each row (all languages for that edge_category)
    df_lang_total = df.copy()
    df_lang_total['language'] = 'Total'
    df = pd.concat([df, df_lang_total], ignore_index=True)
    df = df.drop(columns=['game', 'edges'])
    df = df.groupby(['language', 'edge_category']).mean().reset_index()
    df = df.round(2)
    return df

def avg_per_edges_total(df):
    df = avg_per_edges(df)
    # Pivot to have row per lang, with edge_category as columns
    df = df.pivot(index='language', columns='edge_category', values='total_time')
    # Sort rows by language
    df = df.sort_index(ascending=False)
    labels=['<50', '>50', '>100', '>500', '>1000', '>2000', '>4000', 'Total']
    # Sort columns based on labels
    df = df.round(1)
    df = df.reindex(columns=labels)
    print(df)
    return df

def main():
    df = load_data()
    df = df[df['language'] != 'rg']
    avg_per_edges_total(df).to_csv("results/transforms_grouped_total.csv")

if __name__ == "__main__":
    main()