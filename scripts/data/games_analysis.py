import pandas as pd


def with_stats():
    df = pd.read_csv("../collect/results/stats.csv")
    df = df[df['flags'] == 'none']
    df = df[['game', 'language', 'edges']]
    # Categorize edge: <50, <50, <100, <500, <1000, <2000
    df['edge_category'] = pd.cut(df['edges'], bins=[-1, 50, 100, 500, 1000, 2000, 4000, float('inf')], labels=['<50', '<100', '<500', '<1000', '<2000', '<4000', '>4000'])
    # Add edge_category Total with everything
    # Add 'Total' edge_category for each row (all edges for that language)
    df_total = df.copy()
    df_total['edge_category'] = 'Total'
    df = pd.concat([df, df_total], ignore_index=True)
    # Add 'Total' language for each row (all languages for that edge_category)
    df_lang_total = df.copy()
    df_lang_total['language'] = 'Total'
    df = pd.concat([df, df_lang_total], ignore_index=True)
    df = df.drop(columns=['game'])
    df = df.groupby(['language', 'edge_category']).count().reset_index()
    # Pivot to have row per lang, with edge_category as columns
    df = df.pivot(index='language', columns='edge_category', values='edges')
    # Sort rows by language
    df = df.sort_index(ascending=False)
    # Change values to integers
    df = df.fillna(0).astype(int)
    labels=['<50', '<100', '<500', '<1000', '<2000', '<4000', '>4000','Total']
    # Sort columns based on labels
    df = df.reindex(columns=labels)
    print(df)
    return df

def print_stats():
    df = pd.read_csv("../collect/results/stats.csv")
    df = df[df['flags'] == 'none']
    df = df[['game', 'language', 'edges']]
    df = df.sort_values(by=['edges'])
    df.to_csv("results/sorted.csv", index=False)

def main():
    with_stats().to_csv("results/stats_grouped.csv")
    print_stats()

if __name__ == "__main__":
    main()