import type * as monaco from 'monaco-editor';

import { getLanguageConfiguration } from './getLanguageConfiguration';

export const configuration = getLanguageConfiguration('//');
export const language: monaco.languages.IMonarchLanguage = {
  defaultToken: 'source',
  declarationKeywords: ['type', 'const', 'var'],
  operators: ['=', '==', '!=', '!', '?', '->', '$'],
  symbols: /[=!?\->]+/,
  tokenizer: {
    root: [
      [/\d+/, 'number'],
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
      [/@[\w]+/, 'macro'],
    ],
  },
};
