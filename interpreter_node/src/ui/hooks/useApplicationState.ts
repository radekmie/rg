import { gunzipSync, gzipSync, strFromU8, strToU8 } from 'fflate';
import { useEffect, useMemo, useState } from 'react';

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

function parseStateFromQuery() {
  try {
    const hash = window.location.hash.slice(1);
    const base64 = hash.replace(/\+/g, '-').replace(/\//g, '_');
    const decoded = atob(decodeURIComponent(base64));
    const decompressed = strFromU8(gunzipSync(strToU8(decoded, true)), true);
    const deserialized = JSON.parse(decompressed) as State;
    return {
      preset: deserialized.preset,
      settings: deserialized.settings,
      source: deserialized.source,
      view: deserialized.view,
    };
  } catch (_) {
    console.log(_);
    return initialState;
  }
}

function updateStateInQuery(state: State) {
  try {
    const serialized = JSON.stringify(state);
    const compressed = strFromU8(gzipSync(strToU8(serialized, true)), true);
    const encoded = encodeURIComponent(btoa(compressed));
    const base64 = encoded.replace(/-/g, '+').replace(/_/g, '/');
    window.location.hash = base64;
  } catch (_) {
    // It's alright.
  }
}

export function useApplicationState() {
  const [state, setState] = useState(parseStateFromQuery);
  useEffect(() => updateStateInQuery(state), [state]);

  const { preset, settings, source, view } = state;
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
