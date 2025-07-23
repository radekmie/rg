import pandas as pd


def load_data():
    return pd.read_csv('../collect/results/transforms.csv')


def tws(df):
    df = df.drop(columns=['count', 'changed'])
    df = df[df['flags'] == '--enable-all-optimizations']
    df = df.drop(columns=['flags', 'game'])

    df = df.groupby(['language', 'transform']).mean().reset_index()
    df['changed_time_perc'] = df['changed_time'] / df['total_time'] * 100
    df['changed_time_perc'] = df['changed_time_perc'].round(2)
    df = df.drop(columns=['changed_time', 'total_time'])

    # pivot to have tranfsorms as rows and languages as columns
    df = df.pivot(index='transform', columns='language', values='changed_time_perc')
    # reset index to have 'transform' as a column
    df = df.reset_index()
    # fill NaN values with 0
    df = df.fillna(0)
    return df

def tws_avg_per_lang(df):
    df = tws(df)
    df = df.drop(columns=['transform'])
    df = df.groupby('language').mean().reset_index()
    df = df.rename(columns={'changed_time_perc': 'tws_avg_perc'})
    df['tws_avg_perc'] = df['tws_avg_perc'].round(2)
    df = df.drop(columns=['changed_time', 'total_time'])
    return df


# On what % of games in each lang the transform is used
def used_in_lang(df):
    df = df.drop(columns=['count', 'changed_time', 'total_time'])
    df = df[df['flags'] == '--enable-all-optimizations']
    df = df.drop(columns=['flags', 'game'])
    df['changed'] = df['changed'].apply(lambda x: 1 if x else 0)
    df = df.groupby(['language', 'transform']).mean().reset_index()
    df['used_in_lang_perc'] = df['changed'] * 100
    df = df.drop(columns=['changed'])
    # limit precision to 2 decimal places
    df['used_in_lang_perc'] = df['used_in_lang_perc'].round(2)
    # pivot the table to have languages as columns
    df = df.pivot(index='transform', columns='language', values='used_in_lang_perc')
    # reset index to have 'transform' as a column
    df = df.reset_index()
    # fill NaN values with 0
    df = df.fillna(0)
    return df

def flag_influence(df):
    df = df.drop(columns=['count', 'changed_time', 'total_time', 'game'])
    df = df.groupby(['language', 'transform', 'flags']).mean().reset_index()
    df['changed'] = df['changed'].round(2)
    df_all = df[df['flags'] == '--enable-all-optimizations']
    df = df[df['flags'] != '--enable-all-optimizations']
    df = df[df['flags'] != 'none']
    df = df.merge(df_all, on=['language', 'transform'], suffixes=('', '_all'))
    df['changed_diff'] = (df['changed_all'] - df['changed']) / df['changed_all'] * 100
    df['changed_diff'] = df['changed_diff'].round(2).fillna(0)
    df = df.drop(columns=['changed_all', 'changed', 'flags_all'])
    return df


def main():
    df = load_data()
    df = df[
        df['transform'].apply(
            lambda x: x != 'add_builtins' and not x.startswith('check')
        )
    ]
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
    # used_in_lang(df).to_csv('results/used_in_lang.csv', index=False)
    tws(df).to_csv('results/tws_avg.csv', index=False)
    # flag_influence(df).to_csv('results/flag_influence.csv', index=False)
    # tws_avg_per_lang(df).to_csv('results/tws_avg_per_lang.csv', index=False)


if __name__ == '__main__':
    main()
