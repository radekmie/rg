import { Callout, Intent, Radio, RadioGroup } from '@blueprintjs/core';
import { FormEvent, useCallback } from 'react';

import { Extension, Optimize, Settings } from '../../types';
import * as styles from './Settings.module.css';

export type SettingsProps = {
  intent: Intent;
  onChange: (fn: (value: Settings) => Settings) => void;
  value: Settings;
};

export function Settings({ intent, onChange, value }: SettingsProps) {
  const onExtension = useCallback(
    (event: FormEvent<HTMLInputElement>) => {
      const extension = event.currentTarget.value as Extension;
      onChange(value => ({ ...value, extension }));
    },
    [onChange],
  );

  const onOptimize = useCallback(
    (event: FormEvent<HTMLInputElement>) => {
      const optimize = event.currentTarget.value as Optimize;
      onChange(value => ({ ...value, optimize }));
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
      <RadioGroup
        className={styles.options}
        label="Optimize"
        onChange={onOptimize}
        selectedValue={value.optimize}
      >
        <Radio label="Yes" value={Optimize.yes} />
        <Radio label="No" value={Optimize.no} />
      </RadioGroup>
    </Callout>
  );
}
