import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

def load_data():
    """Load the data from the CSV file."""
    """Columns: game, language, plays_optimized, plays_none"""
    return pd.read_csv("../data/results/tws_with_total_time_perc.csv")

def create_plot_big(df):
    df = df[df['language'] != 'rg']
    # df = df[df['language'] != 'kif']
    # Only keep relevant columns
    plot_df = df[['transform', 'language', 'total_time_perc', 'changed_time_perc']].copy()
    # Sort transforms by mean total_time_perc for consistent order
    order = plot_df.groupby('transform')['total_time_perc'].mean().sort_values(ascending=False).index.tolist()

    plot_df['transform'] = pd.Categorical(plot_df['transform'], categories=order, ordered=True)
    plot_df = plot_df.sort_values(['transform', 'language'])

    languages = plot_df['language'].unique()
    transforms = order
    transforms = transforms[:12]
    x = np.arange(len(transforms))
    width = 0.18
    # Use this palette
    palette = {'hrg': 'cornflowerblue', 'rbg': 'orange', 'kif': 'green'}
    fig, ax = plt.subplots(figsize=(max(12, len(transforms)*0.8), 7))

    for i, lang in enumerate(languages):
        lang_df = plot_df[plot_df['language'] == lang]
        # Align bars for missing transforms
        lang_df = lang_df.set_index('transform').reindex(transforms).reset_index()
        changed = lang_df['changed_time_perc']
        total = lang_df['total_time_perc']
        ax.bar(x + (i - 1.5)*width, changed, width, label=f'{lang} Changed', color=palette[lang], alpha=0.8)
        ax.bar(x + (i - 1.5)*width, total - changed, width, bottom=changed, label=f'{lang} Unchanged', color=palette[lang], alpha=0.3)

    ax.set_ylabel('% Time Spent')
    ax.set_xlabel('Transform')
    ax.set_title('Time Spent per Transform (by Language)')
    ax.set_xticks(x)
    ax.set_xticklabels(transforms, rotation=45, ha='right')
    # Custom legend: one for changed, one for unchanged, color per language
    from matplotlib.patches import Patch
    legend_patches = []
    for i, lang in enumerate(languages):
        legend_patches.append(Patch(facecolor=f'C{i}', alpha=0.8, label=f'{lang} Changed'))
    for i, lang in enumerate(languages):
        legend_patches.append(Patch(facecolor=f'C{i}', alpha=0.3, label=f'{lang} Unchanged'))
    ax.legend(handles=legend_patches, ncol=2)
    plt.tight_layout()
    return ax


def main():
    df = load_data()
    ax = create_plot_big(df)
    ax.figure.savefig(f"results/tws_plot_big.png")

if __name__ == "__main__":
    main()