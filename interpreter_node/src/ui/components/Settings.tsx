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

import { Extension, Settings } from '../../types';
import { presets } from '../const/presets';
import { useApplicationState, View } from '../hooks/useApplicationState';
import * as styles from '../index.module.css';

const availablePresets = presets.map(game => game.name);
const configurableFlags = [
  'compactSkipEdges',
  'expandGeneratorNodes',
  'mangleSymbols',
  'reuseFunctions',
] as const;

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
  const availableViews = useMemo(
    () => [
      { value: 'Bench' },
      { value: 'Automaton' },
      { value: 'Source (HL, original)', disabled: !isHrg },
      { value: 'Source (HL, formatted)', disabled: !isHrg },
      { value: 'CST (HL)', disabled: !isHrg },
      { value: 'AST (HL)', disabled: !isHrg },
      { value: 'Source (LL, original)' },
      { value: 'Source (LL, formatted)' },
      { value: 'CST (LL)' },
      { value: 'AST (LL)' },
      { value: 'IST (LL)' },
      { value: 'Graphviz (LL)' },
    ],
    [isHrg],
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
          <Radio label="rg" value={Extension.rg} />
        </RadioGroup>
        {configurableFlags.map((flag, index) => (
          <section className={styles.options} key={flag}>
            <Label>{index === 0 ? 'Flags' : ''}</Label>
            <Checkbox
              checked={settings.flags[flag]}
              label={`--${flag}`}
              name={flag}
              onChange={onFlag}
            />
          </section>
        ))}
      </Callout>
    </>
  );
}
