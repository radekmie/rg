import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import CodeMirror, { ReactCodeMirrorProps } from '@uiw/react-codemirror';
import { useCallback } from 'react';

import { Autosize } from './Autosize';
import * as hrg from '../../hrg';
import * as rbg from '../../rbg';
import * as rg from '../../rg';
import * as styles from '../index.module.css';
import { createChevrotainHighlighter } from '../lib/createChevrotainHighlighter';

const modeToExtensions = {
  hrg: [
    createChevrotainHighlighter(source => {
      const { cstNode, tokens } = hrg.cst.parse(source);
      const ast = hrg.ast.visit(cstNode);
      return {
        color1: ast.functions.map(({ identifier }) => identifier),
        color2: ast.domains.map(({ identifier }) => identifier),
        color3: ast.variables.map(({ identifier }) => identifier),
        tokens,
      };
    }),
  ],
  javascript: [javascript()],
  json: [json()],
  rbg: [
    createChevrotainHighlighter(source => {
      const { cstNode, tokens } = rbg.cst.parse(source);
      const ast = rbg.ast.visit(cstNode);
      return {
        color1: ast.board.flatMap(({ edges }) => edges.map(edge => edge.label)),
        color2: ast.pieces,
        color3: ast.players.concat(ast.variables).map(({ name }) => name),
        tokens,
      };
    }),
  ],
  rg: [
    createChevrotainHighlighter(source => {
      const { cstNode, tokens } = rg.cst.parse(source);
      const ast = rg.ast.visit(cstNode);
      return {
        color1: ast.constants.map(({ identifier }) => identifier),
        color2: ast.types.map(({ identifier }) => identifier),
        color3: ast.variables.map(({ identifier }) => identifier),
        tokens,
      };
    }),
  ],
  text: [],
};

export type EditorProps = ReactCodeMirrorProps & {
  mode: keyof typeof modeToExtensions;
};

export function Editor({ mode, ...props }: EditorProps) {
  const onEditor = useCallback(
    (ref: { editor?: HTMLDivElement | null } | null) => {
      ref?.editor
        ?.querySelector('.cm-content')
        ?.setAttribute('data-enable-grammarly', 'false');
    },
    [],
  );

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
