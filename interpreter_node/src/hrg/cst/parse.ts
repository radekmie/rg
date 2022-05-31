import { ILexingResult } from 'chevrotain';

import { lexer } from './lexer';
import { parser as parserInstance } from './parser';

export class LexerError extends Error {
  name = 'LexerError';
  constructor(
    public readonly lexingResult: ILexingResult,
    public readonly parser: typeof parserInstance,
  ) {
    super();
  }
}

export class ParserError extends Error {
  name = 'ParserError';
  constructor(
    public readonly lexingResult: ILexingResult,
    public readonly parser: typeof parserInstance,
  ) {
    super();
  }
}

export function parse(source: string) {
  parserInstance.input = [];

  const lexingResult = lexer.tokenize(source);
  if (lexingResult.errors.length > 0) {
    throw new LexerError(lexingResult, parserInstance);
  }

  parserInstance.input = lexingResult.tokens;
  const cstNode = parserInstance.GameDeclaration();

  if (parserInstance.errors.length > 0) {
    throw new ParserError(lexingResult, parserInstance);
  }

  return { cstNode, lexingResult };
}
