import {
  Callout,
  Checkbox,
  Intent,
  Label,
  Radio,
  RadioGroup,
} from '@blueprintjs/core';
import { FormEvent, useCallback } from 'react';

import { Extension, Settings } from '../../types';
import * as styles from './Settings.module.css';

const configurableFlags = ['compactSkipEdges'] as const;

export type SettingsProps = {
  intent: Intent;
  onChange: (fn: (value: Settings) => Settings) => void;
  value: Settings;
};

export function Settings({ intent, onChange, value }: SettingsProps) {
  const onExtension = useCallback(
    ({ currentTarget: { value } }: FormEvent<HTMLInputElement>) => {
      onChange(x => ({ ...x, extension: value as Extension }));
    },
    [onChange],
  );

  const onFlag = useCallback(
    ({ currentTarget: { checked, name } }: FormEvent<HTMLInputElement>) => {
      onChange(x => ({ ...x, flags: { ...x.flags, [name]: checked } }));
    },
    [onChange],
  );

  return (
    <Callout intent={intent}>
      <RadioGroup
        className={styles.options}
        label="Extension"
        onChange={onExtension}
        selectedValue={value.extension}
      >
        <Radio label="hrg" value={Extension.hrg} />
        <Radio label="rg" value={Extension.rg} />
      </RadioGroup>
      <section className={styles.options}>
        <Label>Flags</Label>
        {configurableFlags.map(flag => (
          <Checkbox
            checked={value.flags[flag]}
            key={flag}
            label={flag}
            name={flag}
            onChange={onFlag}
          />
        ))}
      </section>
    </Callout>
  );
}
