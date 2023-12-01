import * as monaco from 'monaco-editor';

import * as hrg from './hrg';
import * as rbg from './rbg';
import * as rg from './rg';
import { Language } from '../../../../types';

export const conf: monaco.languages.LanguageConfiguration = {
  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
  ],
  comments: { lineComment: '//' },
};

export const theme: monaco.editor.IStandaloneThemeData = {
  base: 'vs',
  inherit: true,
  colors: {},
  rules: [
    { token: 'comment', foreground: '6a737d', fontStyle: 'italic' },
    { token: 'declarationKeyword', foreground: '0000ff', fontStyle: 'bold' },
    { token: 'keyword', foreground: 'a626a4' },
    { token: 'type', foreground: '2b91af' },
    { token: 'member', foreground: '000000' },
    { token: 'constant', foreground: 'c5060b' },
    { token: 'variable', foreground: '005cc5' },
    { token: 'function', foreground: '986801' },
    { token: 'macro', foreground: 'ff0000' },
  ],
};

export function monarch(id: Language) {
  switch (id) {
    case Language.rg:
      return rg.language;
    case Language.rbg:
      return rbg.language;
    case Language.hrg:
      return hrg.language;
  }
}
