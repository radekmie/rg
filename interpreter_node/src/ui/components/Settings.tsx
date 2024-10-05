import { Callout, Checkbox, HTMLSelect, Intent } from '@blueprintjs/core';
import { ChangeEvent, FormEvent, useCallback, useMemo } from 'react';

import { Flag, Settings } from '../../types';
import { presets } from '../const/presets';
import { useApplicationState } from '../hooks/useApplicationState';
import * as styles from '../index.module.css';

const availablePresets = presets.map(game => game.name);
const availableOptions: { label: string; flags: Flag[] }[] = [
  {
    label: 'Optimizations',
    flags: [
      'compactComparisons',
      'compactSkipEdges',
      'inlineAssignment',
      'inlineReachability',
      'joinForkPrefixes',
      'joinForkSuffixes',
      'pruneSingletonTypes',
      'pruneUnreachableNodes',
      'pruneUnusedBindings',
      'pruneUnusedConstants',
      'pruneUnusedVariables',
      'reuseFunctions',
      'skipGeneratorComparisons',
      'skipSelfAssignments',
      'skipSelfComparisons',
      'skipUnusedTags',
    ],
  },
  {
    label: 'Pragmas',
    flags: [
      'calculateDisjoints',
      'calculateRepeats',
      'calculateSimpleApply',
      'calculateTagIndexes',
      'calculateUniques',
    ],
  },
  {
    label: 'Other',
    flags: [
      'addExplicitCasts',
      'expandGeneratorNodes',
      'mangleSymbols',
      'normalizeTypes',
    ],
  },
];

export type SettingsProps = {
  preset: string;
  settings: Settings;
  view: number;
} & Pick<ReturnType<typeof useApplicationState>, 'game'> &
  Pick<
    ReturnType<typeof useApplicationState>['actions'],
    'setPreset' | 'setSettings' | 'setView'
  >;

export function Settings({
  game,
  preset,
  setPreset,
  setSettings,
  setView,
  settings,
  view,
}: SettingsProps) {
  const onFlag = useCallback(
    ({ currentTarget: { checked, name } }: FormEvent<HTMLInputElement>) => {
      setSettings(x => ({
        ...x,
        flags: name.split(',').reduce(
          (flags, flag) => {
            flags[flag as Flag] = checked;
            return flags;
          },
          { ...x.flags },
        ),
      }));
    },
    [setSettings],
  );

  const onPreset = useCallback(
    ({ currentTarget: { value } }: ChangeEvent<HTMLSelectElement>) => {
      setPreset(value);
    },
    [setPreset],
  );

  const onView = useCallback(
    ({ currentTarget: { value } }: ChangeEvent<HTMLSelectElement>) => {
      setView(+value);
    },
    [setView],
  );

  const availableViews = useMemo(
    () =>
      (game.value?.steps ?? []).map((step, index) => {
        let label = {
          ast: 'AST.',
          automaton: 'Automaton',
          bench: 'Bench',
          cst: 'CST.',
          error: 'Error',
          graphviz: 'Automaton.graphviz',
          source: 'src.',
          stats: 'Stats',
        }[step.kind];

        if ('language' in step) {
          label += step.language;
        }

        if (step.title) {
          label += ` [${step.title}]`;
        }

        label = `${String(index + 1).padEnd(3)} ${label}`;

        return { label, value: index };
      }),
    [game],
  );

  return (
    <>
      <div className={styles.grid}>
        <HTMLSelect
          className={styles.select}
          onChange={onPreset}
          options={availablePresets}
          value={preset}
        />
        <HTMLSelect
          className={styles.select}
          onChange={onView}
          options={availableViews}
          value={view}
        />
      </div>
      <Callout
        className={styles.settings}
        icon={game.loading ? 'time' : undefined}
        intent={
          game.loading
            ? Intent.NONE
            : game.error ?? game.value?.error
            ? Intent.DANGER
            : Intent.SUCCESS
        }
      >
        {availableOptions.map(({ flags, label }) => {
          const checked = flags.filter(flag => settings.flags[flag]).length;
          return (
            <div className={styles.flags} key={label}>
              <Checkbox
                checked={checked === flags.length}
                indeterminate={!!checked && checked !== flags.length}
                label={label}
                name={flags.join()}
                onChange={onFlag}
              />
              {flags.map(flag => (
                <Checkbox
                  checked={settings.flags[flag]}
                  key={flag}
                  label={`--${flag}`}
                  name={flag}
                  onChange={onFlag}
                />
              ))}
            </div>
          );
        })}
      </Callout>
    </>
  );
}
