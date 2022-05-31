import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import CodeMirror, { ReactCodeMirrorProps } from '@uiw/react-codemirror';
import { useCallback } from 'react';

import * as hrg from '../../hrg';
import * as rg from '../../rg';
import * as styles from '../index.module.css';
import { createChevrotainHighlighter } from '../lib/createChevrotainHighlighter';
import { Autosize } from './Autosize';

const modeToExtensions = {
  hrg: [createChevrotainHighlighter(hrg.cst.lexer)],
  javascript: [javascript()],
  json: [json()],
  rg: [createChevrotainHighlighter(rg.cst.lexer)],
  text: [],
};

export type EditorProps = ReactCodeMirrorProps & {
  mode: keyof typeof modeToExtensions;
};

export function Editor({ mode, ...props }: EditorProps) {
  const onEditor = useCallback((ref?: { editor?: HTMLDivElement }) => {
    ref?.editor
      ?.querySelector('.cm-content')
      ?.setAttribute('data-enable-grammarly', 'false');
  }, []);

  return (
    <Autosize>
      {({ height }) => (
        <section className={styles.wrapHidden}>
          <CodeMirror
            extensions={modeToExtensions[mode]}
            height={`${height}px`}
            ref={onEditor}
            {...props}
          />
        </section>
      )}
    </Autosize>
  );
}
