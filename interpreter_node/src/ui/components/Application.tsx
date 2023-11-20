import { Intent } from '@blueprintjs/core';

import { Bench } from './Bench';
import { Editor } from './Editor';
import { ReactMonacoEditor } from '../new_editor/Editor';
import { Graphviz } from './Graphviz';
import { Loader } from './Loader';
import { PrettyPrint } from './PrettyPrint';
import { Settings } from './Settings';
import { AnalyzedGame } from '../../parse';
import { Extension } from '../../types';
import { useApplicationState } from '../hooks/useApplicationState';
import * as styles from '../index.module.css';

const extensionToMode = {
  [Extension.hrg]: 'hrg',
  [Extension.rbg]: 'rbg',
  [Extension.rg]: 'rg',
} as const;

const valueForView = {
  Automaton: (game: AnalyzedGame) => game.graphvizRg,
  Bench: (game: AnalyzedGame) => game.astRg,
  Graphviz: (game: AnalyzedGame) =>
    ({ mode: 'text', value: game.graphvizRg } as const),
  'AST.hrg': (game: AnalyzedGame) => game.astHrg,
  'AST.rbg': (game: AnalyzedGame) => game.astRbg,
  'AST.rg': (game: AnalyzedGame) => game.astRg,
  'CST.hrg': (game: AnalyzedGame) => game.cstHrg,
  'CST.rbg': (game: AnalyzedGame) => game.cstRbg,
  'Source (result).hrg': (game: AnalyzedGame) =>
    ({ mode: 'hrg', value: game.sourceHrgFormatted ?? '' } as const),
  'Source (source).hrg': (game: AnalyzedGame) =>
    ({ mode: 'hrg', value: game.sourceHrg ?? '' } as const),
  'Source (result).rbg': (game: AnalyzedGame) =>
    ({ mode: 'rbg', value: game.sourceRbgFormatted ?? '' } as const),
  'Source (source).rbg': (game: AnalyzedGame) =>
    ({ mode: 'rbg', value: game.sourceRbg ?? '' } as const),
  'Source (result).rg': (game: AnalyzedGame) =>
    ({ mode: 'rg', value: game.sourceRgFormatted } as const),
  'Source (source).rg': (game: AnalyzedGame) =>
    ({ mode: 'rg', value: game.sourceRg } as const),
};

export function Application() {
  const {
    actions: { setPreset, setSettings, setSource, setView },
    game,
    settings,
    source,
    path,
    view,
  } = useApplicationState();
  return (
    <>
      <section className={styles.panel}>
        <ReactMonacoEditor
          path={path}
          source={source}
          onChange={setSource}
        />
      </section>
      <section className={styles.panel}>
        <Settings
          intent={
            game.loading
              ? Intent.NONE
              : game.error
                ? Intent.DANGER
                : Intent.SUCCESS
          }
          setPreset={setPreset}
          setSettings={setSettings}
          setView={setView}
          settings={settings}
          view={view}
        />
        {game.error ? (
          <PrettyPrint value={game.error} />
        ) : game.value ? (
          (() => {
            switch (view) {
              case 'Automaton':
                return <Graphviz source={valueForView[view](game.value)} />;
              case 'Bench':
                return (
                  <Bench gameDeclaration={valueForView[view](game.value)} />
                );
              case 'Graphviz':
              case 'Source (result).hrg':
              case 'Source (result).rbg':
              case 'Source (result).rg':
              case 'Source (source).hrg':
              case 'Source (source).rbg':
              case 'Source (source).rg':
                return <Editor {...valueForView[view](game.value)} />;
              default:
                return <PrettyPrint value={valueForView[view](game.value)} />;
            }
          })()
        ) : (
          <Loader />
        )}
      </section>
    </>
  );
}
