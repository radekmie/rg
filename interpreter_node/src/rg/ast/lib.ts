import * as ast from './types';
import * as utils from '../../utils';

export function incoming(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  // Fast path for simple edge names.
  if (edgeName.parts.length === 1 && edgeName.parts[0].kind === 'Literal') {
    const { identifier } = edgeName.parts[0];
    return edges.filter(
      ({ rhs: { parts } }) =>
        parts.length === 1 &&
        parts[0].kind === 'Literal' &&
        parts[0].identifier === identifier,
    );
  }

  return edges.filter(({ rhs }) => utils.isEqual(edgeName, rhs));
}

export function outgoing(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  // Fast path for simple edge names.
  if (edgeName.parts.length === 1 && edgeName.parts[0].kind === 'Literal') {
    const { identifier } = edgeName.parts[0];
    return edges.filter(
      ({ lhs: { parts } }) =>
        parts.length === 1 &&
        parts[0].kind === 'Literal' &&
        parts[0].identifier === identifier,
    );
  }

  return edges.filter(({ lhs }) => utils.isEqual(edgeName, lhs));
}
