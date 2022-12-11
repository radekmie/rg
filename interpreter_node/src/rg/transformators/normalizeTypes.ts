import * as ast from '../ast';

export function normalizeTypes(gameDeclaration: ast.GameDeclaration) {
  const typeChecker = new ast.TypeChecker(gameDeclaration);

  // Type declarations are already normalized except for the arrow types.
  for (const { type } of gameDeclaration.types) {
    if (type.kind === 'Arrow') {
      type.rhs = typeChecker.normalizeType(type.rhs);
    }
  }

  // Normalize other types.
  for (const field of ['constants', 'variables'] as const) {
    for (const declaration of gameDeclaration[field]) {
      declaration.type = typeChecker.normalizeType(declaration.type);
    }
  }
}
