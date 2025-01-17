import type * as monaco from 'monaco-editor';

import { getLanguageConfiguration } from './getLanguageConfiguration';

export const configuration = getLanguageConfiguration('//');
export const language: monaco.languages.IMonarchLanguage = {
  defaultToken: 'source',
  declarationKeywords: ['domain', 'graph'],
  keywords: [
    'branch',
    'else',
    'forall',
    'if',
    'in',
    'loop',
    'or',
    'repeat',
    'then',
    'where',
    'while',
  ],
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
  ],
  symbols: /[=><!~?:&|+\-*/^%]+/,
  tokenizer: {
    root: [
      [/\d+/, 'number'],
      [
        /\b(reusable)(?=\s+graph\b)/,
        'declarationKeyword', // "reusable" before "graph"
      ],
      [
        /[a-z0-9_][a-zA-Z0-9_]*/,
        {
          cases: {
            '@declarationKeywords': 'declarationKeyword',
            '@keywords': 'keyword',
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
