import type { languages } from 'monaco-editor';

export const language = <languages.IMonarchLanguage>{
  defaultToken: 'source',
  declarationKeywords: ['domain', 'graph'],
  keywords: [
    'branch',
    'else',
    'if',
    'in',
    'loop',
    'or',
    'then',
    'when',
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
      [/[A-Z][a-zA-Z0-9_]*/, 'type.identifier'], // to show class names nicely
      [/[{}()[\]]/, '@brackets'],
      [/[:;,.]/, 'delimiter'],
      [
        /@symbols/,
        {
          cases: {
            '@operators': 'operator',
            '@default': '',
          },
        },
      ],
      [/\/\/.*$/, 'comment'],
      [/[ \t\r\n]+/, 'white'],
    ],
  },
};
