import pandas as pd


def load_data():
    return pd.read_csv('../collect/results/transforms.csv')

# TODO: Dodac kalkulacje calego trwania liczenia per gra


# Average % of time spent in this transform is well spent, per language
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

# Average time of all transforms in this language is usefull
def tws_avg_per_lang(df):
    df = tws(df)
    df = df.drop(columns=['transform'])
    df = df.groupby('language').mean().reset_index()
    df = df.rename(columns={'changed_time_perc': 'tws_avg_perc'})
    df['tws_avg_perc'] = df['tws_avg_perc'].round(2)
    df = df.drop(columns=['changed_time', 'total_time'])
    return df


def tws_with_total_time_perc(df):
    df = df.drop(columns=['count', 'changed'])
    df = df[df['flags'] == '--enable-all-optimizations']
    df_total = df[['game', 'language', 'total_time']]
    df_total = df_total.groupby(['game', 'language']).sum().reset_index()
    df = df.merge(df_total, on=['game', 'language'], suffixes=('', '_total'))
    df['total_time_perc'] = df['total_time'] / df['total_time_total'] * 100
    df = df.drop(columns=['flags', 'game', 'total_time'])

    df = df.groupby(['language', 'transform']).mean().reset_index()
    df['changed_time_perc'] = df['changed_time'] / df['total_time_total'] * 100
    df = df.round(2)
    df = df.drop(columns=['changed_time', 'total_time_total'])

    # pivot to have tranfsorms as rows and languages as columns
    # df = df.pivot(index='transform', columns='language', values='changed_time_perc')
    # reset index to have 'transform' as a column
    # df = df.reset_index()
    # fill NaN values with 0
    df = df.fillna(0)
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
    # used_in_lang(df).to_csv('results/used_in_lang.csv', index=False)
    # tws(df).to_csv('results/tws_avg.csv', index=False)
    # flag_influence(df).to_csv('results/flag_influence.csv', index=False)
    # tws_avg_per_lang(df).to_csv('results/tws_avg_per_lang.csv', index=False)
    tws_with_total_time_perc(df).to_csv('results/tws_with_total_time_perc.csv', index=False)


if __name__ == '__main__':
    main()
