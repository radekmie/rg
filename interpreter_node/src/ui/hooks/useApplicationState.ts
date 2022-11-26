import { useDeferredValue, useMemo, useState } from 'react';

import { parse } from '../../parse';
import { Settings } from '../../types';
import * as utils from '../../utils';
import { safe } from '../../utils';
import { presets } from '../const/presets';

export type State = {
  settings: Settings;
  source: string;
  view: View;
};

export type View =
  | 'AST.hrg'
  | 'AST.rbg'
  | 'AST.rg'
  | 'Automaton'
  | 'Bench'
  | 'CST.hrg'
  | 'CST.rbg'
  | 'CST.rg'
  | 'Graphviz'
  | 'IST.rg'
  | 'Source (result).hrg'
  | 'Source (result).rbg'
  | 'Source (result).rg'
  | 'Source (source).hrg'
  | 'Source (source).rbg'
  | 'Source (source).rg';

const initialPreset = presets[0];
const initialState: State = {
  settings: {
    extension: initialPreset.extension,
    flags: {
      compactSkipEdges: true,
      expandGeneratorNodes: false,
      mangleSymbols: false,
      removeSelfAssignments: true,
      reuseFunctions: true,
    },
  },
  source: initialPreset.source,
  view: 'Automaton',
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
  const game = useMemo(() => safe(() => parse(...gameInput)), [gameInput]);

  return { actions, game, state };
}
