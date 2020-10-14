import lexer from './lexer';
import parser from './parser';
import visitor from './visitor';

export function parse(source: string) {
  const result = lexer.tokenize(source);
  if (result.errors.length > 0)
    throw Object.assign(new Error('Lexer error'), { errors: result.errors });

  parser.input = result.tokens;
  const cstNode = parser.game();

  if (parser.errors.length > 0)
    throw Object.assign(new Error('Parser error'), { errors: parser.errors });

  const astNode = visitor.visitNode(cstNode);
  return astNode;
}

import fs from 'fs';
import util from 'util';

const source = fs.readFileSync(process.argv[2], { encoding: 'utf8' });
const astNode = parse(source);
console.log(util.inspect(astNode, undefined, null));
