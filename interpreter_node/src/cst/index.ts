import lexer from './lexer';
import parser from './parser';

export default function parse(source: string) {
  const result = lexer.tokenize(source);
  if (result.errors.length > 0) {
    throw Object.assign(new Error('Lexer error'), { errors: result.errors });
  }

  parser.input = result.tokens;
  const cstNode = parser.GameDeclaration();

  if (parser.errors.length > 0) {
    throw Object.assign(new Error('Parser error'), { errors: parser.errors });
  }

  return cstNode;
}
