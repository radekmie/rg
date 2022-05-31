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
  hrg: [
    createChevrotainHighlighter(source => {
      try {
        const { cstNode, lexingResult } = hrg.cst.parse(source);
        const ast = hrg.ast.visit(cstNode);
        return {
          color1: ast.functions.map(({ identifier }) => identifier),
          color2: ast.domains.map(({ identifier }) => identifier),
          color3: ast.variables.map(({ identifier }) => identifier),
          lexingResult,
        };
      } catch (error) {
        if (
          error instanceof hrg.cst.LexerError ||
          error instanceof hrg.cst.ParserError
        ) {
          return { lexingResult: error.lexingResult };
        }

        throw error;
      }
    }),
  ],
  javascript: [javascript()],
  json: [json()],
  rg: [
    createChevrotainHighlighter(source => {
      try {
        const { cstNode, lexingResult } = rg.cst.parse(source);
        const ast = rg.ast.visit(cstNode);
        return {
          color1: ast.constants.map(({ identifier }) => identifier),
          color2: ast.types.map(({ identifier }) => identifier),
          color3: ast.variables.map(({ identifier }) => identifier),
          lexingResult,
        };
      } catch (error) {
        if (
          error instanceof rg.cst.LexerError ||
          error instanceof rg.cst.ParserError
        ) {
          return { lexingResult: error.lexingResult };
        }

        throw error;
      }
    }),
  ],
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
