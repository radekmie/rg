import * as monaco from 'monaco-editor';

export function getLanguageConfiguration(
  lineComment: string,
): monaco.languages.LanguageConfiguration {
  return {
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
    comments: { lineComment },
  };
}
