import * as ast from '../ast';

export function skipSelfAssignments({ edges }: ast.GameDeclaration) {
  for (const edge of edges) {
    if (isSelfAssignment(edge.label)) {
      edge.label = ast.Skip({});
    }
  }
}

function isEqualReference(lhs: ast.Expression, rhs: ast.Expression): boolean {
  if (lhs.kind === 'Cast') {
    return isEqualReference(lhs.rhs, rhs);
  }

  if (rhs.kind === 'Cast') {
    return isEqualReference(lhs, rhs.rhs);
  }

  switch (lhs.kind) {
    case 'Access':
      return (
        rhs.kind === 'Access' &&
        isEqualReference(lhs.lhs, rhs.lhs) &&
        isEqualReference(lhs.rhs, rhs.rhs)
      );
    case 'Reference':
      return rhs.kind === 'Reference' && lhs.identifier === rhs.identifier;
  }
}

function isSelfAssignment(edgeLabel: ast.EdgeLabel) {
  return (
    edgeLabel.kind === 'Assignment' &&
    isEqualReference(edgeLabel.lhs, edgeLabel.rhs)
  );
}
