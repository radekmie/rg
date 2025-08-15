import pandas as pd


def load_data():
    return pd.read_csv('../collect/results/stats.csv')


def process_flag_influence(df: pd.DataFrame):
    df = df.drop(columns=['game', 'repeat_nodes','repeat_or_unique_nodes', 'unique_nodes', 'typedefs'])
    df = df.groupby(['language', 'flags']).mean().reset_index()
    metrics = list(filter(lambda x: x not in ['language', 'flags'], df.columns))
    df_all = df[df['flags'] == '--enable-all-optimizations']
    df = df[df['flags'] != '--enable-all-optimizations']
    df = df[df['flags'] != 'none']
    df = df.merge(df_all, on=['language'], suffixes=('', '_all'))
    for metric in metrics:
        df[metric] = (df[f'{metric}_all'] - df[metric]) / df[metric] * 100
        df[metric] = df[metric].fillna(0).round(2)
    df = df.drop(columns=[f'{metric}_all' for metric in metrics] + ['flags_all'])
    return df

def main():
    df = load_data()
    process_flag_influence(df).to_csv("results/flag_metrics_influence.csv", index=False)

if __name__ == "__main__":
    main()