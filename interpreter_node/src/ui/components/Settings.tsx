import {
  Callout,
  Checkbox,
  HTMLSelect,
  Intent,
  Label,
  Radio,
  RadioGroup,
} from '@blueprintjs/core';
import { ChangeEvent, FormEvent, useCallback, useMemo } from 'react';

import { Flag, Language, Settings, noFlagsEnabled } from '../../types';
import { presets } from '../const/presets';
import { useApplicationState } from '../hooks/useApplicationState';
import * as styles from '../index.module.css';

const availablePresets = presets.map(game => game.name);
const availableFlags = Object.keys(noFlagsEnabled);

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
  const onExtension = useCallback(
    ({ currentTarget: { value } }: FormEvent<HTMLInputElement>) => {
      setSettings(x => ({ ...x, extension: value as Language }));
    },
    [setSettings],
  );

  const onFlag = useCallback(
    ({ currentTarget: { checked, name } }: FormEvent<HTMLInputElement>) => {
      setSettings(x => ({ ...x, flags: { ...x.flags, [name]: checked } }));
    },
    [setSettings],
  );

  const onPreset = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setPreset(event.currentTarget.value);
    },
    [setPreset],
  );

  const onView = useCallback(
    (event: ChangeEvent<HTMLSelectElement>) => {
      setView(+event.currentTarget.value);
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
          className={styles.noOutline}
          onChange={onPreset}
          options={availablePresets}
          value={preset}
        />
        <HTMLSelect
          className={styles.noOutline}
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
        <RadioGroup
          className={styles.options}
          label="Extension"
          onChange={onExtension}
          selectedValue={settings.extension}
        >
          {Object.entries(Language).map(([label, value]) => (
            <Radio key={value} label={label} value={value} />
          ))}
        </RadioGroup>
        {availableFlags.map((flag, index) => (
          <div className={styles.options} key={flag}>
            <Label>{index === 0 ? 'Flags' : ''}</Label>
            <Checkbox
              checked={settings.flags[flag as Flag]}
              label={`--${flag}`}
              name={flag}
              onChange={onFlag}
            />
          </div>
        ))}
      </Callout>
    </>
  );
}
