import { ChangeEvent, useCallback } from 'react';

import * as styles from './Editor.module.css';

export type EditorProps = {
  onChange: (value: string) => void;
  value: string;
};

export function Editor({ onChange, value }: EditorProps) {
  const onChangeHandler = useCallback(
    (event: ChangeEvent<HTMLTextAreaElement>) => onChange(event.target.value),
    [onChange],
  );

  return (
    <textarea
      className={styles.editor}
      data-gramm={false}
      onChange={onChangeHandler}
      value={value}
    />
  );
}
