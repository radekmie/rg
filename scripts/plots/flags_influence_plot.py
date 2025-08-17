import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import numpy as np

def load_data():
    """Load the data from the CSV file."""
    """Columns: game, language, plays_optimized, plays_none"""
    return pd.read_csv("../data/results/flag_influence.csv")


def signed_log(x, eps=1e-6):
    return np.sign(x) * np.log10(np.abs(x) + eps)


def create_plot(df, language=None):
    if language:
        df = df[df['language'] == language]
        df = df.drop(columns=['language'])
    else:
        df = df.drop(columns=['language'])
        df = df.groupby(['flags', 'transform']).mean().reset_index()
    

    df['flags'] = df['flags'].str.replace(r'--', '', regex=True)
    df['flags'] = df['flags'].str.replace(r'-', '_', regex=True)

    # df['changed_diff'] = signed_log(df['changed_diff'])

    # Columns: transform, flags, changed_diff
    # Create a heatmap without values on tiles
    sns.set_theme(style="whitegrid")
    pivot_table = df.pivot_table(
        index='transform', 
        columns='flags', 
        values='changed_diff',
        fill_value=None
    )
    plt.figure(figsize=(12, 8))
    ax = sns.heatmap(pivot_table, robust=True, annot=False, cmap="coolwarm", cbar_kws={'label': 'Average runs difference (%)'}, center=0)
    ax.set_title(f"Enabled Optimization Influence on Other Transformations")
    ax.set_xlabel("Enabled Optimization")
    ax.set_ylabel("Transformation") 
    plt.xticks(rotation=90, ha='right')
    plt.tight_layout()
    plt.show()    

    return ax

def main():
    df = load_data()
    for language in [None]:
        ax = create_plot(df, language)
        ax.figure.savefig(f"results/flags_influence_plot_{language}.png")

if __name__ == "__main__":
    main()