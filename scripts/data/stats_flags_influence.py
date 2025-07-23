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
    df['game_lang'] = df['game'] + '.' + df['language']
    df = df[df['game_lang'].isin(games)]
    df = df.drop(columns=['game_lang'])
    process_flag_influence(df).to_csv("results/flag_metrics_influence.csv", index=False)

if __name__ == "__main__":
    main()