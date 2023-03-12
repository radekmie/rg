import { javascript } from '@codemirror/lang-javascript';
import { json } from '@codemirror/lang-json';
import CodeMirror, { ReactCodeMirrorProps } from '@uiw/react-codemirror';
import { useCallback } from 'react';

import { Autosize } from './Autosize';
import * as hrg from '../../hrg';
import * as rbg from '../../rbg';
import * as wasm from '../../wasm';
import * as styles from '../index.module.css';
import { createAsyncHighlighter } from '../lib/createAsyncHighlighter';

function extractIdentifier(object: { identifier: string }) {
  return object.identifier;
}

function extractLabel(object: { label: string }) {
  return object.label;
}

function extractName(object: { name: string }) {
  return object.name;
}

const hrgKeywords = new Set([
  'branch',
  'domain',
  'else',
  'graph',
  'if',
  'in',
  'loop',
  'or',
  'then',
  'when',
  'where',
  'while',
]);

const rbgKeywords = new Set([
  'board',
  'pieces',
  'players',
  'rules',
  'variables',
]);

const rgKeywords = new Set(['any', 'const', 'type', 'var']);

const modeToExtensions = {
  hrg: [
    createAsyncHighlighter(source => {
      const { cstNode } = hrg.cst.parse(source);
      const { domains, functions, variables } = hrg.ast.visit(cstNode);
      return Promise.resolve([
        hrgKeywords,
        new Set(functions.map(extractIdentifier)),
        new Set(domains.map(extractIdentifier)),
        new Set(variables.map(extractIdentifier)),
      ]);
    }),
  ],
  javascript: [javascript()],
  json: [json()],
  rbg: [
    createAsyncHighlighter(source => {
      const { cstNode } = rbg.cst.parse(source);
      const { board, pieces, players, variables } = rbg.ast.visit(cstNode);
      return Promise.resolve([
        rbgKeywords,
        new Set(board.flatMap(({ edges }) => edges.map(extractLabel))),
        new Set(pieces),
        new Set(players.concat(variables).map(extractName)),
      ]);
    }),
  ],
  rg: [
    createAsyncHighlighter(async source => {
      const ast = await wasm.parseRg(source);
      const { constants, types, variables } = ast;
      return [
        rgKeywords,
        new Set(constants.map(extractIdentifier)),
        new Set(types.map(extractIdentifier)),
        new Set(variables.map(extractIdentifier)),
      ];
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
