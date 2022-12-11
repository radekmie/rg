import * as ast from '../ast';

export function addExplicitCasts(gameDeclaration: ast.GameDeclaration) {
  const typeChecker = new ast.TypeChecker(gameDeclaration);
  for (const edge of gameDeclaration.edges) {
    typeChecker.addExplicitCasts(edge);
  }
}
