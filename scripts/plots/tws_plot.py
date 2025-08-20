import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

def load_data():
    """Load the data from the CSV file."""
    """Columns: game, language, plays_optimized, plays_none"""
    return pd.read_csv("../data/results/tws_avg.csv")

def create_plot_big(df):
    # I have 4 columsn: transform, rbg, hrg, kif (3 values)
    # Give me a bar plot, 3 bars for each transform
    df_melted = df.melt(id_vars=['transform'], value_vars=['rbg', 'hrg', 'kif'], var_name='language', value_name='changed_time_perc')
    df_melted = df_melted[df_melted['changed_time_perc'] > 1.0]
    plt.figure(figsize=(12, 6))
    # Let hrg be light blue, rbg orange and kif green
    # Change hrg to HRG, rbg to RBG, and kif to GDL
    lang_map = {'hrg': 'HRG', 'rbg': 'RBG', 'kif': 'GDL'}
    palette = {'HRG': 'cornflowerblue', 'RBG': 'orange', 'GDL': 'green'}
    df_melted['language'] = df_melted['language'].map(lang_map)
    sns.barplot(data=df_melted, x='transform', y='changed_time_perc', hue='language', palette=palette, errorbar=None)
    plt.title('Time Well Spent on Each Transformation')
    plt.ylabel('% Time Spent on Changing Passes')
    plt.xlabel('Transformation')
    plt.xticks(rotation=90)
    plt.legend(title='Language')
    plt.tight_layout()
    return plt.gca()

def main():
    df = load_data()
    ax = create_plot_big(df)
    ax.figure.savefig(f"results/tws_plot.png")

if __name__ == "__main__":
    main()