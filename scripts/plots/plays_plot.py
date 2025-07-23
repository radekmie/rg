import pandas as pd
import matplotlib.pyplot as plt

def load_data():
    """Load the data from the CSV file."""
    """Columns: game, language, plays_optimized, plays_none"""
    return pd.read_csv("../data/results/plays.csv")


def transform_data(df):
    """Instead of optimized and none, % increase in plays."""
    df["plays_increase"] = (df["plays_optimized"] - df["plays_none"]) / df["plays_none"] * 100
    df['game_language'] = df['game'] + "." + df['language']
    return df[['game_language', 'plays_increase', 'language']]

def create_plot(df, language=None):
    if language:
        df = df[df['language'] == language]

    df = df.drop(columns=["language"])
    df = df.set_index("game_language")
    # sort by plays_increase
    df = df.sort_values(by="plays_increase", ascending=True)
    ax = df.plot(kind="bar", stacked=True, figsize=(20, 10))
    ax.set_ylabel("Number of Plays Increase (%)")
    ax.set_xlabel("Game")
    ax.set_title("Plays per 10s increase (in %)")
    ax.legend(title="Game")
    plt.xticks(rotation=90)
    plt.tight_layout()

    return ax

def create_plot_avg(df, ):
    df = df.drop(columns=["game_language"])
    df = df.groupby("language").mean().reset_index()

    df = df.set_index("language")
    # sort by plays_increase
    df = df.sort_values(by="plays_increase", ascending=True)
    ax = df.plot(kind="bar", stacked=True, figsize=(20, 10))
    ax.set_ylabel("Number of Plays Increase (%)")
    ax.set_xlabel("Language")
    ax.set_title("Average plays per 10s increase (in %)")
    ax.legend(title="Game")
    plt.xticks(rotation=90)
    plt.tight_layout()

    return ax



def main():
    df = load_data()
    df = transform_data(df)
    for language in ["hrg", "kif", "rbg", "rg", None]:
        ax = create_plot(df, language)
        ax.figure.savefig(f"results/plays_plot_{language}.png")
    ax = create_plot(df)
    average_ax = create_plot_avg(df)
    average_ax.figure.savefig("results/plays_plot_avg.png")

if __name__ == "__main__":
    main()