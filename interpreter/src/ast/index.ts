import { CstNode } from 'chevrotain';

import { GameDeclaration } from './types';
import visitor from './visitor';

export default function build(cstNode: CstNode) {
  const astNode: GameDeclaration = visitor.visitNode(cstNode);
  return astNode;
}
