import pandas as pd

def load_data():
    """Load the data from the CSV file."""
    return pd.read_csv("../collect/results/stats.csv")

def measure_impact_per_game_language(df):
    with_none = df[df['flags'].apply(lambda x: x == "none")]
    with_all = df[df['flags'].apply(lambda x: x == "--enable-all-optimizations")]
    other = df[df['flags'].apply(lambda x: x not in ["none", "--enable-all-optimizations", "--compact-skip-edges", "--prune-unreachable-nodes"])]

    most_impactful = other.groupby(['game', 'language']).apply(
        lambda group: group.loc[group['edges'].idxmax()]
    ).reset_index(drop=True)

    print(with_none.head(10))
    print(with_all.head(10))
    print(most_impactful.head(10))

def measure_impact_per_language(df):
    # drop column 'game' and take average per language and flags of all numeric columns
    df = df.drop(columns=['game'])
    df = df.groupby(['language', 'flags']).mean().reset_index()
    
    with_none = df[df['flags'].apply(lambda x: x == "none")]
    with_all = df[df['flags'].apply(lambda x: x == "--enable-all-optimizations")]
    other = df[df['flags'].apply(lambda x: x not in ["none", "--enable-all-optimizations", "--compact-skip-edges", "--prune-unreachable-nodes"])]

    most_impactful = other.groupby('language').apply(
        lambda group: group.loc[group['edges'].idxmax()]
    ).reset_index(drop=True)

    print(with_none.head(10))
    print(with_all.head(10))
    print(most_impactful.head(10))

def best_vs_worst_per_game(df):
    with_none = df[df['flags'].apply(lambda x: x == "none")]
    with_all = df[df['flags'].apply(lambda x: x == "--enable-all-optimizations")]

    merged = pd.merge(with_none, with_all, on=['game', 'language'], suffixes=('_none', '_all'))
    merged['nodes_diff_percentage'] = -(
        (merged['nodes_all'] - merged['nodes_none']) / merged['nodes_none']
    ) * 100
    merged['edges_diff_percentage'] = -(
        (merged['edges_all'] - merged['edges_none']) / merged['edges_none']
    ) * 100
    merged['state_size_diff_percentage'] = -(
        (merged['state_size_all'] - merged['state_size_none']) / merged['state_size_none']
    ) * 100
    merged['variables_diff_percentage'] = -(
        (merged['variables_all'] - merged['variables_none']) / merged['variables_none']
    ) * 100
    merged['game_language'] = merged['game'] + "." + merged['language']
    games = []
    games.append('alquerque.py')
    games.append('alquerque_lud.py')
    games.append('amazons.hrg')
    games.append('amazons_split2.hrg')
    games.append('ataxx.hrg')
    games.append('backgammon.hrg')
    #games.append('battleships.hrg')
    games.append('bombardment.hrg')
    games.append('breakthrough.hrg')
    games.append('chess.hrg')
    games.append('chess_kingCapture.hrg')
    games.append('clobber.hrg')
    games.append('connect4.hrg')
    games.append('dashGuti.py')
    games.append('dashGuti_lud.py')
    games.append('dotsAndBoxes.hrg')
    games.append('englishDraughts.hrg')
    games.append('foxAndGeese.hrg')
    games.append('foxAndGeese_lud.hrg')
    games.append('golSkuish.py')
    games.append('golEkuish_lud.py')
    games.append('gomoku_standard.hrg')
    games.append('knightthrough.hrg')
    games.append('lauKataKati.py')
    games.append('lauKataKati_lud.py')
    games.append('oware.hrg')
    games.append('pentago.hrg')
    games.append('pentago_split.hrg')
    games.append('pretwa.py')
    games.append('pretwa_lud.py')
    games.append('ticTacDie.hrg')
    #games.append('twentyOne.hrg')
    games.append('ultimateTicTacToe.hrg')
    
    games.append('alquerque.rbg')
    games.append('alquerque_lud.rbg')
    games.append('amazons.rbg')
    games.append('amazons_split2.rbg')
    games.append('breakthrough.rbg')
    # games.append('chessGardner5x5_kingCapture.rbg')
    # games.append('chessLosAlamos6x6_kingCapture.rbg')
    # games.append('chessQuick5x6_kingCapture.rbg')
    # games.append('chessSilverman4x5_kingCapture.rbg')
    games.append('chess.rbg')
    games.append('chess_kingCapture.rbg')
    games.append('connect4.rbg')
    games.append('dashGuti.rbg')
    games.append('englishDraughts.rbg')
    games.append('foxAndHounds.rbg')
    games.append('golSkuish.rbg')
    games.append('gomoku_standard.rbg')
    games.append('hex.rbg')
    games.append('knightthrough.rbg')
    games.append('lauKataKati.rbg')
    games.append('pentago.rbg')
    games.append('pentago_split.rbg')
    games.append('pretwa.rbg')
    games.append('reversi.rbg')
    # games.append('skirmish.rbg')
    games.append('surakarta.rbg')
    games.append('theMillGame.rbg')
    games.append('theMillGame_lud.rbg')
    games.append('yavalath.rbg')
    merged = merged[merged['game_language'].isin(games)]
    merged = merged[['language', 'nodes_diff_percentage', 'edges_diff_percentage', 'state_size_diff_percentage', 'variables_diff_percentage']]
    merged = merged.groupby('language').mean().reset_index()
    merged.to_csv("avg_impact.csv", index=False)



def main():
    df = load_data()
    df = df.drop(columns=['repeat_nodes','repeat_or_unique_nodes', 'unique_nodes', 'typedefs'])
    # Measure the impact of the flags
    best_vs_worst_per_game(df)

if __name__ == "__main__":
    main()