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

import { Extension, Flag, Settings } from '../../types';
import { presets } from '../const/presets';
import { useApplicationState, View } from '../hooks/useApplicationState';
import * as styles from '../index.module.css';

const availablePresets = presets.map(game => game.name);

export type SettingsProps = {
  intent: Intent;
  settings: Settings;
  view: View;
} & Pick<
  ReturnType<typeof useApplicationState>['actions'],
  'setPreset' | 'setSettings' | 'setView'
>;

export function Settings({
  intent,
  setPreset,
  setSettings,
  setView,
  settings,
  view,
}: SettingsProps) {
  const onExtension = useCallback(
    ({ currentTarget: { value } }: FormEvent<HTMLInputElement>) => {
      setSettings(x => ({ ...x, extension: value as Extension }));
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
      setView(event.currentTarget.value as unknown as View);
    },
    [setView],
  );

  const isHrg = settings.extension === Extension.hrg;
  const isRbg = settings.extension === Extension.rbg;

  const availableFlags = useMemo<{ value: Flag; disabled?: boolean }[]>(
    () => [
      { value: 'compactSkipEdges' },
      { value: 'expandGeneratorNodes' },
      { value: 'mangleSymbols' },
      { value: 'removeSelfAssignments' },
      { value: 'reuseFunctions', disabled: !isHrg },
    ],
    [isHrg],
  );

  const availableViews = useMemo<{ value: View; disabled?: boolean }[]>(
    () => [
      { value: 'Bench' },
      { value: 'Automaton' },
      { value: 'Graphviz' },
      { value: 'Source (source).hrg', disabled: !isHrg },
      { value: 'Source (result).hrg', disabled: !isHrg },
      { value: 'Source (source).rbg', disabled: !isRbg },
      { value: 'Source (result).rbg', disabled: !isRbg },
      { value: 'Source (source).rg' },
      { value: 'Source (result).rg' },
      { value: 'CST.hrg', disabled: !isHrg },
      { value: 'CST.rbg', disabled: !isRbg },
      { value: 'AST.hrg', disabled: !isHrg },
      { value: 'AST.rbg', disabled: !isRbg },
      { value: 'CST.rg' },
      { value: 'AST.rg' },
      { value: 'IST.rg' },
    ],
    [isHrg, isRbg],
  );

  return (
    <>
      <section className={styles.grid}>
        <HTMLSelect
          className={styles.noOutline}
          onChange={onPreset}
          options={availablePresets}
        />
        <HTMLSelect
          className={styles.noOutline}
          disabled={intent !== Intent.SUCCESS}
          onChange={onView}
          options={availableViews}
          value={view}
        />
      </section>
      <Callout intent={intent}>
        <RadioGroup
          className={styles.options}
          label="Extension"
          onChange={onExtension}
          selectedValue={settings.extension}
        >
          <Radio label="hrg" value={Extension.hrg} />
          <Radio label="rbg (experimental)" value={Extension.rbg} />
          <Radio label="rg" value={Extension.rg} />
        </RadioGroup>
        {availableFlags.map(({ disabled, value }, index) => (
          <section className={styles.options} key={value}>
            <Label>{index === 0 ? 'Flags' : ''}</Label>
            <Checkbox
              checked={settings.flags[value]}
              disabled={disabled}
              label={`--${value}`}
              name={value}
              onChange={onFlag}
            />
          </section>
        ))}
      </Callout>
    </>
  );
}
