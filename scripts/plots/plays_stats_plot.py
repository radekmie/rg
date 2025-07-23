import pandas as pd
import matplotlib.pyplot as plt

def load_data():
    """Load the data from the CSV file."""
    """Columns: game, language, plays_optimized, plays_none"""
    return (pd.read_csv("../data/results/plays.csv"), pd.read_csv("../collect/results/stats.csv"))


def transform_data(df):
    """Instead of optimized and none, % increase in plays."""
    df["plays_increase"] = (df["plays_optimized"] - df["plays_none"]) / df["plays_none"] * 100
    return df[['game', 'plays_increase', 'language', 'nodes']]

def create_plot(df, language=None):
    if language:
        df = df[df['language'] == language]

    # create scatter plot where X is n of edges, Y is plays_increase
    df = df.drop(columns=["language"])
    df = df.set_index("game")
    df = df.sort_values(by="plays_increase", ascending=True)
    ax = df.plot(kind="scatter", x="nodes", y="plays_increase", figsize=(20, 10), title="Plays Increase vs Edges")
    ax.set_ylabel("Number of Plays Increase (%)")
    ax.set_xlabel("Number of Edges")
    ax.set_title("Plays per 10s increase (in %) vs Edges")
    plt.xticks(rotation=90)
    plt.tight_layout()

    return ax


def main():
    (df, stats_df) = load_data()
    stats_df = stats_df[stats_df['flags'] == 'none']
    stats_df = stats_df[['game', 'language', 'nodes']]
    df = pd.merge(df, stats_df, on=['game', 'language'], how='left')
    df = transform_data(df)
    print(df.head())

    for language in ["hrg", "kif", "rbg", "rg", None]:
        ax = create_plot(df, language)
        ax.figure.savefig(f"results/plays_stats_plot_{language}.png")
    ax = create_plot(df)
    # average_ax = create_plot_avg(df)
    # average_ax.figure.savefig("results/plays_plot_avg.png")

if __name__ == "__main__":
    main()