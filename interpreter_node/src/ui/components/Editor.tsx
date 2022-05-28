import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import CodeMirror, { ReactCodeMirrorProps } from '@uiw/react-codemirror';

import * as hrg from '../../hrg';
import * as rg from '../../rg';
import { createChevrotainHighlighter } from '../lib/createChevrotainHighlighter';
import * as styles from './Application.module.css';
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
  return (
    <Autosize>
      {({ height }) => (
        <section className={styles.wrapHidden}>
          <CodeMirror
            extensions={modeToExtensions[mode]}
            height={`${height}px`}
            {...props}
          />
        </section>
      )}
    </Autosize>
  );
}
