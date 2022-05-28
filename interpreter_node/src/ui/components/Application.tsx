import {
  Alignment,
  HTMLSelect,
  Intent,
  Navbar,
  Tab,
  Tabs,
} from '@blueprintjs/core';
import { ChangeEvent, useCallback, useMemo } from 'react';

import { Extension } from '../../types';
import { presets } from '../const/presets';
import { View, useApplicationState } from '../hooks/useApplicationState';
import * as styles from './Application.module.css';
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

  const options = useMemo(() => presets.map(game => game.name), []);
  const onPreset = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setPreset(event.currentTarget.value);
    },
    [setPreset],
  );

  return (
    <>
      <section className={styles.panel}>
        <Navbar className={styles.clear}>
          <Navbar.Group>
            <HTMLSelect onChange={onPreset} options={options} />
          </Navbar.Group>
        </Navbar>
        <Editor
          onChange={setSource}
          mode={extensionToMode[settings.extension]}
          value={source}
        />
      </section>
      <section className={styles.panel}>
        <Navbar className={styles.clear}>
          <Navbar.Group align={Alignment.RIGHT}>
            <Tabs
              id="view"
              onChange={setView}
              renderActiveTabPanelOnly
              selectedTabId={view}
            >
              <Tab disabled={!game.ok} id={View.Bench} title="Bench" />
              <Tab disabled={!game.ok} id={View.Automaton} title="Automaton" />
              <Tab disabled={!game.ok} id={View.Graphviz} title="Graphviz" />
              <Tab disabled={!game.ok} id={View.HighLevel} title="HL" />
              <Tab disabled={!game.ok} id={View.LowLevel} title="LL" />
              <Tab disabled={!game.ok} id={View.CST} title="CST" />
              <Tab disabled={!game.ok} id={View.AST} title="AST" />
              <Tab disabled={!game.ok} id={View.IST} title="IST" />
            </Tabs>
          </Navbar.Group>
        </Navbar>
        <Settings
          intent={game.ok ? Intent.SUCCESS : Intent.DANGER}
          onChange={setSettings}
          value={settings}
        />
        {game.ok ? (
          (() => {
            switch (view) {
              case View.AST:
                return <PrettyPrint value={game.value.astRg} />;
              case View.Automaton:
                return <Graphviz source={game.value.graphvizRg} />;
              case View.Bench:
                return <Bench game={game.value.istRg} />;
              case View.CST:
                return <PrettyPrint value={game.value.cstRg} />;
              case View.Graphviz:
                return <PrettyPrint value={game.value.graphvizRg} />;
              case View.HighLevel:
                return <Editor mode="hrg" value={game.value.sourceHrg ?? ''} />;
              case View.IST:
                return <PrettyPrint value={game.value.istRg} />;
              case View.LowLevel:
                return <Editor mode="rg" value={game.value.sourceRg} />;
            }
          })()
        ) : (
          <PrettyPrint value={game.error} />
        )}
      </section>
    </>
  );
}
