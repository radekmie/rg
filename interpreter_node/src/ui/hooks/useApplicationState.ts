import { useMemo, useState } from 'react';

import { usePromise } from './usePromise';
import { parse } from '../../parse';
import { Settings, noFlagsEnabled } from '../../types';
import * as utils from '../../utils';
import { presets } from '../const/presets';

export type State = {
  path: string;
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
  | 'Graphviz'
  | 'Source (result).hrg'
  | 'Source (result).rbg'
  | 'Source (result).rg'
  | 'Source (source).hrg'
  | 'Source (source).rbg'
  | 'Source (source).rg';

const initialPreset = presets[0];
const initialState: State = {
  path: initialPreset.name,
  settings: {
    extension: initialPreset.extension,
    flags: {
      ...noFlagsEnabled,
      compactSkipEdges: true,
      reuseFunctions: true,
      skipSelfAssignments: true,
    },
  },
  source: initialPreset.source,
  view: 'Automaton',
};

export function useApplicationState() {
  const [{ path, settings, source, view }, setState] = useState(initialState);
  const game = usePromise(() => parse(source, settings), [settings, source]);
  const actions = useMemo(
    () => ({
      setPreset(name: string) {
        const preset = presets.find(game => game.name === name);
        utils.assert(preset, `Unknown preset "${name}".`);
        setState(state => ({
          ...state,
          path: preset.name,
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

  return { actions, game, path, settings, source, view };
}
