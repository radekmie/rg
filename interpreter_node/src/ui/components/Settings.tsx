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

import { Flag, Language, Settings } from '../../types';
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
      setView(event.currentTarget.value as unknown as View);
    },
    [setView],
  );

  const isHrg = settings.extension === Language.hrg;
  const isRbg = settings.extension === Language.rbg;

  const availableFlags = useMemo<{ value: Flag; disabled?: boolean }[]>(
    () => [
      { value: 'addExplicitCasts' },
      { value: 'compactSkipEdges' },
      { value: 'expandGeneratorNodes' },
      { value: 'inlineReachability' },
      { value: 'joinForkSuffixes' },
      { value: 'mangleSymbols' },
      { value: 'normalizeTypes' },
      { value: 'reuseFunctions', disabled: !isHrg },
      { value: 'skipSelfAssignments' },
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
      { value: 'AST.rg' },
    ],
    [isHrg, isRbg],
  );

  return (
    <>
      <div className={styles.grid}>
        <HTMLSelect
          className={styles.noOutline}
          onChange={onPreset}
          options={availablePresets}
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
        icon={intent === Intent.NONE ? 'time' : undefined}
        intent={intent}
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
        {availableFlags.map(({ disabled, value }, index) => (
          <div className={styles.options} key={value}>
            <Label>{index === 0 ? 'Flags' : ''}</Label>
            <Checkbox
              checked={settings.flags[value]}
              disabled={disabled}
              label={`--${value}`}
              name={value}
              onChange={onFlag}
            />
          </div>
        ))}
      </Callout>
    </>
  );
}
