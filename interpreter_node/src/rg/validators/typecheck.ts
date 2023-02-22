import * as ast from '../ast';

export function typecheck(gameDeclaration: ast.GameDeclaration) {
  const typeChecker = new ast.TypeChecker(gameDeclaration);
  for (const edge of gameDeclaration.edges) {
    typeChecker.checkEdge(edge);
  }

  return Promise.resolve();
}
