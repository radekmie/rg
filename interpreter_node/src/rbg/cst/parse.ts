import { ILexingError, IRecognitionException } from 'chevrotain';

import { lexer } from './lexer';
import { parser } from './parser';

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
  parser.input = [];
  parser.reset();

  const { errors, tokens } = lexer.tokenize(source);
  if (errors.length > 0) {
    throw new LexerError(errors);
  }

  parser.input = tokens;
  const cstNode = parser.GameDescription();

  if (parser.errors.length > 0) {
    throw new ParserError(parser.errors);
  }

  return { cstNode, tokens };
}
