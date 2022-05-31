import { ILexingError, IRecognitionException } from 'chevrotain';

import { lexer } from './lexer';
import { parser as parserInstance } from './parser';

export class LexerError extends Error {
  name = 'LexerError';
  constructor(public readonly errors: ILexingError[]) {
    super();
  }
}

export class ParserError extends Error {
  name = 'ParserError';
  constructor(public readonly errors: IRecognitionException[]) {
    super();
  }
}

export function parse(source: string) {
  parserInstance.input = [];

  const { errors, tokens } = lexer.tokenize(source);
  if (errors.length > 0) {
    throw new LexerError(errors);
  }

  parserInstance.input = tokens;
  const cstNode = parserInstance.GameDeclaration();

  if (parserInstance.errors.length > 0) {
    throw new ParserError(parserInstance.errors);
  }

  return { cstNode, tokens };
}
