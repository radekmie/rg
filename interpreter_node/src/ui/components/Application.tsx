import { Intent, Navbar, Tab, Tabs } from '@blueprintjs/core';

import { View, useActiveView } from '../hooks/useActiveView';
import { useGame } from '../hooks/useGame';
import { useSettings } from '../hooks/useSettings';
import * as styles from './Application.module.css';
import { Editor } from './Editor';
import { Graphviz } from './Graphviz';
import { PrettyPrint } from './PrettyPrint';
import { Settings } from './Settings';

export function Application() {
  const { settings, setSettings } = useSettings();
  const { game, source, setSource } = useGame(settings);
  const { activeView, setActiveView } = useActiveView();

  return (
    <>
      <Editor onChange={setSource} value={source} />
      <section className={styles.panel}>
        <Navbar>
          <Navbar.Group className={styles.wide}>
            <Tabs
              className={styles.wide}
              id="view"
              onChange={setActiveView}
              renderActiveTabPanelOnly
              selectedTabId={activeView}
            >
              <Tab disabled={!game.ok} id={View.Automaton} title="Automaton" />
              <Tab disabled={!game.ok} id={View.Graphviz} title="Graphviz" />
              <Tab disabled={!game.ok} id={View.HighLevel} title="HL" />
              <Tab disabled={!game.ok} id={View.LowLevel} title="LL" />
              <Tab disabled={!game.ok} id={View.AST} title="AST" />
              <Tab disabled={!game.ok} id={View.CST} title="CST" />
              <Tab disabled={!game.ok} id={View.IST} title="IST" />
            </Tabs>
          </Navbar.Group>
        </Navbar>
        <Settings onChange={setSettings} value={settings} />
        {game.ok ? (
          (() => {
            switch (activeView) {
              case View.AST:
                return <PrettyPrint value={game.value.ast} />;
              case View.Automaton:
                return <Graphviz source={game.value.graphviz} />;
              case View.CST:
                return <PrettyPrint value={game.value.cst} />;
              case View.Graphviz:
                return <PrettyPrint value={game.value.graphviz} />;
              case View.HighLevel:
                return <PrettyPrint value={game.value.source.hl} />;
              case View.IST:
                return <PrettyPrint value={game.value.ist} />;
              case View.LowLevel:
                return <PrettyPrint value={game.value.source.ll} />;
            }
          })()
        ) : (
          <PrettyPrint intent={Intent.DANGER} value={game.error} />
        )}
      </section>
    </>
  );
}
