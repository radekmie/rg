import pandas as pd

def foo(df, n):
    df_count = df[['game', 'language', 'flags']].groupby(['game', 'language']).count().reset_index()
    df = df.merge(df_count, on=['game', 'language'], suffixes=('', '_count'))
    df = df[df['flags_count'] == n]
    df = df.drop(columns=['flags_count'])
    return df

if __name__ == "__main__":
    df = pd.read_csv("results/stats.csv")
    df = foo(df, 23)
    df.to_csv("results/stats_filtered.csv", index=False)
    df = pd.read_csv("results/transforms.csv")
    df = foo(df, 625)
    df.to_csv("results/transforms_filtered.csv", index=False)
