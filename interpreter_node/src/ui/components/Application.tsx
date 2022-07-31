import { Intent } from '@blueprintjs/core';

import { Extension } from '../../types';
import { useApplicationState } from '../hooks/useApplicationState';
import * as styles from '../index.module.css';
import { Bench } from './Bench';
import { Editor } from './Editor';
import { Graphviz } from './Graphviz';
import { PrettyPrint } from './PrettyPrint';
import { Settings } from './Settings';

const extensionToMode = {
  [Extension.hrg]: 'hrg',
  [Extension.rg]: 'rg',
} as const;

export function Application() {
  const {
    actions: { setPreset, setSettings, setSource, setView },
    game,
    state: { settings, source, view },
  } = useApplicationState();

  return (
    <>
      <section className={styles.panel}>
        <Editor
          onChange={setSource}
          mode={extensionToMode[settings.extension]}
          value={source}
        />
      </section>
      <section className={styles.panel}>
        <Settings
          intent={game.ok ? Intent.SUCCESS : Intent.DANGER}
          setPreset={setPreset}
          setSettings={setSettings}
          setView={setView}
          settings={settings}
          view={view}
        />
        {game.ok ? (
          (() => {
            switch (view) {
              case 'Automaton':
                return <Graphviz source={game.value.graphvizRg} />;
              case 'Bench':
                return <Bench game={game.value.istRg} />;
              case 'Source (HL, optimized)':
              case 'Source (HL, original)':
              case 'Source (LL, optimized)':
              case 'Source (LL, original)':
                return (
                  <Editor
                    {...(
                      {
                        'Source (HL, optimized)': {
                          mode: 'hrg',
                          value: game.value.sourceHrgFormatted ?? '',
                        },
                        'Source (HL, original)': {
                          mode: 'hrg',
                          value: game.value.sourceHrg ?? '',
                        },
                        'Source (LL, optimized)': {
                          mode: 'rg',
                          value: game.value.sourceRgFormatted,
                        },
                        'Source (LL, original)': {
                          mode: 'rg',
                          value: game.value.sourceRg,
                        },
                      } as const
                    )[view]}
                  />
                );
              default:
                return (
                  <PrettyPrint
                    value={
                      {
                        'AST (HL)': game.value.astHrg,
                        'AST (LL)': game.value.astRg,
                        'CST (HL)': game.value.cstHrg,
                        'CST (LL)': game.value.cstRg,
                        'Graphviz (LL)': game.value.graphvizRg,
                        'IST (LL)': game.value.istRg,
                      }[view]
                    }
                  />
                );
            }
          })()
        ) : (
          <PrettyPrint value={game.error} />
        )}
      </section>
    </>
  );
}
