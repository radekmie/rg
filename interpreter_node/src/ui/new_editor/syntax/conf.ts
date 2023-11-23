import * as monaco from 'monaco-editor';
import { LanguageID } from '../../../types';
import * as rg from './rg';
import * as rbg from './rbg';
import * as hrg from './hrg';

export const conf: monaco.languages.LanguageConfiguration = {
  comments: {
    lineComment: '//',
  },
  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')'],
  ],
  autoClosingPairs: [
    { open: "{", close: "}" },
    { open: "[", close: "]" },
    { open: "(", close: ")" },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
  ],
}

export const theme: monaco.editor.IStandaloneThemeData =
{
  base: 'vs',
  inherit: true,
  colors: {},
  rules: [
    {
      token: 'comment',
      foreground: '6a737d',
      fontStyle: 'italic'
    },
    {
      token: 'declarationKeyword',
      foreground: '0000ff',
      fontStyle: 'bold'
    },
    {
      token: 'keyword',
      // some light purple
      foreground: 'a626a4',
    },
    {
      token: 'type',
      foreground: '2b91af'
    },
    {
      token: 'member',
      foreground: '000000'
    },
    {
      token: 'constant',
      foreground: 'c5060b',
    },
    {
      token: 'variable',
      foreground: '005cc5'
    },
    {
      token: 'function',
      foreground: '986801'
    },
    {
      token: 'macro',
      // dark red foreground
      foreground: 'ff0000'
    }

  ],
}

export const monarch = (id: LanguageID) => {
  switch (id) {
    case LanguageID.rg:
      return rg.language;
    case LanguageID.rbg:
      return rbg.language;
    case LanguageID.hrg:
      return hrg.language;
  }
}