import { ILexingError, IRecognitionException } from 'chevrotain';

import { lexer } from './lexer';
import { parser as parserInstance } from './parser';

export class LexerError extends Error {
  constructor(public readonly errors: ILexingError[]) {
    super();
    Object.defineProperty(this, 'name', { value: 'LexerError' });
  }
}

export class ParserError extends Error {
  constructor(public readonly errors: IRecognitionException[]) {
    super();
    Object.defineProperty(this, 'name', { value: 'ParserError' });
  }
}

export function parse(source: string) {
  parserInstance.input = [];

  const { errors, tokens } = lexer.tokenize(source);
  if (errors.length > 0) {
    throw new LexerError(errors);
  }

  parserInstance.input = tokens;
  const cstNode = parserInstance.GameDescription();

  if (parserInstance.errors.length > 0) {
    throw new ParserError(parserInstance.errors);
  }

  return { cstNode, tokens };
}
