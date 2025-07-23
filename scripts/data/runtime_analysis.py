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
    df.to_csv("results/plays.csv", index=False)


def main():
    df = load_data()
    transform_data(df)

if __name__ == "__main__":
    main()