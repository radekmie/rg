import { useMemo, useState } from 'react';

import { usePromise } from './usePromise';
import { parse } from '../../parse';
import { Settings, noFlagsEnabled } from '../../types';
import * as utils from '../../utils';
import { presets } from '../const/presets';

export type State = {
  preset: string;
  settings: Settings;
  source: string;
  view: number;
};

const initialPreset = presets[presets.length - 1];
const initialState: State = {
  preset: initialPreset.name,
  settings: {
    extension: initialPreset.extension,
    flags: {
      ...noFlagsEnabled,
      compactSkipEdges: true,
      reuseFunctions: true,
      skipSelfAssignments: true,
      skipSelfComparisons: true,
    },
  },
  source: initialPreset.source,
  view: 0,
};

export function useApplicationState() {
  const [{ preset, settings, source, view }, setState] = useState(initialState);
  const game = usePromise(() => parse(source, settings), [settings, source]);
  const actions = useMemo(
    () => ({
      setPreset(name: string) {
        const preset = presets.find(game => game.name === name);
        utils.assert(preset, `Unknown preset "${name}".`);
        setState(state => ({
          ...state,
          preset: preset.name,
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
      setView(view: number) {
        setState(state => ({ ...state, view }));
      },
    }),
    [],
  );

  return { actions, game, preset, settings, source, view };
}
