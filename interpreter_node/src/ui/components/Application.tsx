import { ReactNode } from 'react';

import { Bench } from './Bench';
import { Editor } from './Editor';
import { Graphviz } from './Graphviz';
import { Loader } from './Loader';
import { PrettyPrint } from './PrettyPrint';
import { Settings } from './Settings';
import { useApplicationState } from '../hooks/useApplicationState';
import * as styles from '../index.module.css';

function extensionSwitcher(preset: string, extension: string) {
  return preset.replace(/\.[^.]*?$/, extension);
}

export function Application() {
  const {
    actions: { setPreset, setSettings, setSource, setView },
    game,
    settings,
    source,
    preset,
    view: viewSelected,
  } = useApplicationState();

  const view = Math.min(viewSelected, (game.value?.steps.length ?? 1) - 1);
  let content: ReactNode;
  if (game.loading) {
    content = <Loader />;
  } else if (game.error) {
    content = <PrettyPrint value={game.error} />;
  } else if (game.value) {
    const step = game.value.steps[view];
    switch (step.kind) {
      case 'automaton':
        content = <Graphviz source={step.value} />;
        break;
      case 'bench':
        content = <Bench gameDeclaration={step.value} />;
        break;
      case 'source':
        content = (
          <Editor
            path={extensionSwitcher(preset, `.${view}.${step.language}`)}
            source={step.value}
            readOnly
          />
        );
        break;
      default:
        content = <PrettyPrint value={step.value} />;
        break;
    }
  } else {
    content = null;
  }

  return (
    <>
      <section className={styles.panel}>
        <Editor onChange={setSource} path={preset} source={source} />
      </section>
      <section className={styles.panel}>
        <Settings
          game={game}
          preset={preset}
          setPreset={setPreset}
          setSettings={setSettings}
          setView={setView}
          settings={settings}
          view={view}
        />
        {content}
      </section>
    </>
  );
}
