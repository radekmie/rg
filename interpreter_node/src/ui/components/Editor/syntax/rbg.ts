import type * as monaco from 'monaco-editor';

import { getLanguageConfiguration } from './getLanguageConfiguration';

export const configuration = getLanguageConfiguration('//');
export const language: monaco.languages.IMonarchLanguage = {
  defaultToken: 'source',
  declarationKeywords: ['board', 'pieces', 'players', 'rules', 'variables'],
  operators: [
    '=',
    '==',
    '!=',
    '!',
    '+',
    '-',
    '*',
    '/',
    '&&',
    '||',
    '$',
    '>',
    '<',
    '>=',
    '<=',
    '->>',
    '->',
    '?',
    '$',
  ],
  symbols: /[=><!~?:&|+\-*/^%]+/,
  tokenizer: {
    root: [
      [
        /[a-z0-9_][a-zA-Z0-9_]*/,
        {
          cases: {
            '@declarationKeywords': 'declarationKeyword',
            '@default': 'identifier',
          },
        },
      ],
      [/\/\/.*$/, 'comment'],
      [/[A-Z][a-zA-Z0-9_]*/, 'type.identifier'], // to show class names nicely
      [/[{}()[\]]/, '@brackets'],
      [/[:;,.]/, 'delimiter'],
      [/@symbols/, { cases: { '@operators': 'operator', '@default': '' } }],
      [/[ \t\r\n]+/, 'white'],
    ],
  },
};
