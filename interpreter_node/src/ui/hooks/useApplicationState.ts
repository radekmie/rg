import { useDeferredValue, useMemo, useState } from 'react';

import { openGame } from '../../io';
import { Optimize, Settings } from '../../types';
import * as utils from '../../utils';
import { safe } from '../../utils';
import { presets } from '../const/presets';

export type State = {
  settings: Settings;
  source: string;
  view: View;
};

export enum View {
  AST,
  Automaton,
  Bench,
  CST,
  Graphviz,
  HighLevel,
  IST,
  LowLevel,
}

const initialPreset = presets[0];
const initialState: State = {
  settings: {
    extension: initialPreset.extension,
    optimize: Optimize.yes,
  },
  source: initialPreset.source,
  view: View.Automaton,
};

export function useApplicationState() {
  const [state, setState] = useState(initialState);
  const actions = useMemo(
    () => ({
      setPreset(name: string) {
        const preset = presets.find(game => game.name === name);
        utils.assert(preset, `Unknown preset "${name}".`);
        setState(state => ({
          ...state,
          settings: { ...state.settings, extension: preset.extension },
          source: preset.source,
        }));
      },
      setSettings(modifier: (prev: Settings) => Settings) {
        setState(state => ({ ...state, settings: modifier(state.settings) }));
      },
      setSource(source: string) {
        setState(state => ({ ...state, source }));
      },
      setView(view: View) {
        setState(state => ({ ...state, view }));
      },
    }),
    [],
  );

  // Use deferred source as `openGame` is potentially slow.
  const gameInput = useDeferredValue([state.source, state.settings] as const);
  const game = useMemo(() => safe(() => openGame(...gameInput)), [gameInput]);

  return { actions, game, state };
}
