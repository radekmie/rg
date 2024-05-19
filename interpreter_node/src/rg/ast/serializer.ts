import * as ast from './types';

export function serializeType(type: ast.Type): string {
  switch (type.kind) {
    case 'Arrow':
      return `${serializeType(type.lhs)} -> ${serializeType(type.rhs)}`;
    case 'Set':
      return `{ ${type.identifiers.join(', ')} }`;
    case 'TypeReference':
      return type.identifier;
  }
}
