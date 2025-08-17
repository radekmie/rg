import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

def load_data():
    """Load the data from the CSV file."""
    """Columns: game, language, plays_optimized, plays_none"""
    return pd.read_csv("../data/results/flag_metrics_influence.csv")

def signed_log(x, eps=1e-6):
    return np.sign(x) * np.log10(np.abs(x) + eps)


def create_plot(df, language=None):
    if language:
        df = df[df['language'] == language]
        df = df.drop(columns=['language'])
    else:
        df = df.drop(columns=['language'])
        df = df.groupby(['flags']).mean().reset_index()

    df['flags'] = df['flags'].str.replace(r'--', '', regex=True)

    # for col in df.columns:
    #     if col != 'flags' and col != 'language':
    #         df[col] = signed_log(df[col])

    # Pivot to create a heatmap, where flags are columns and metrics are rows
    df = df.set_index('flags').T
    sns.set_theme(style="whitegrid")
    
    plt.figure(figsize=(12, 8))
    ax = sns.heatmap(df,robust=True, annot=False, fmt=".2f", cmap="coolwarm", cbar_kws={'label': 'Average Change (%)'}, center=0)
    ax.set_title(f"Optimization Influence on Metrics")
    ax.set_xlabel("Optimization")
    ax.set_ylabel("Metric") 
    plt.xticks(rotation=90, ha='right')
    plt.tight_layout()
    plt.show()

    return ax

def main():
    df = load_data()
    for language in [None]:
        ax = create_plot(df, language)
        ax.figure.savefig(f"results/flags_metrics_influence_plot_{language}.png")

if __name__ == "__main__":
    main()