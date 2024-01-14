import type * as monaco from 'monaco-editor';

import { getLanguageConfiguration } from './getLanguageConfiguration';

export const configuration = getLanguageConfiguration(';');
export const language: monaco.languages.IMonarchLanguage = {
  defaultToken: 'source',
  declarationKeywords: [
    'base',
    'distinct',
    'does',
    'goal',
    'init',
    'input',
    'legal',
    'next',
    'or',
    'role',
    'terminal',
    'true',
  ],
  operators: ['<=', ':-', '~', '&', '?'],
  symbols: /[<=:\-~&?]+/,
  tokenizer: {
    root: [
      [
        /[a-zA-Z0-9_]+/,
        {
          cases: {
            '@declarationKeywords': 'declarationKeyword',
            '@default': 'identifier',
          },
        },
      ],
      [/;.*$/, 'comment'],
      [/[()]/, '@brackets'],
      [/[&,]/, 'delimiter'],
      [/@symbols/, { cases: { '@operators': 'operator', '@default': '' } }],
      [/[ \t\r\n]+/, 'white'],
    ],
  },
};
