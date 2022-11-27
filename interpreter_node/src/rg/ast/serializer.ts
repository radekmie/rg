import * as ast from './types';

export function graphviz(gameDeclaration: ast.GameDeclaration) {
  const graphvizEdges = gameDeclaration.edges.map(edge => {
    const lhs = serializeEdgeName(edge.lhs);
    const rhs = serializeEdgeName(edge.rhs);
    const label = serializeEdgeLabel(edge.label);
    return `  "${lhs}" -> "${rhs}" [label="${label}"];`;
  });

  return `digraph {\n${graphvizEdges.join('\n')}\n}`;
}

export function serializeConstantDeclaration({
  identifier,
  type,
  value,
}: ast.ConstantDeclaration) {
  return `const ${identifier}: ${serializeType(type)} = ${serializeValue(
    value,
  )};`;
}

export function serializeEdge({ label, lhs, rhs }: ast.EdgeDeclaration) {
  return `${serializeEdgeName(lhs)}, ${serializeEdgeName(
    rhs,
  )}: ${serializeEdgeLabel(label)}`;
}

export function serializeEdgeLabel(edgeLabel: ast.EdgeLabel): string {
  switch (edgeLabel.kind) {
    case 'Assignment':
      return `${serializeExpression(edgeLabel.lhs)} = ${serializeExpression(
        edgeLabel.rhs,
      )};`;
    case 'Comparison':
      return `${serializeExpression(edgeLabel.lhs)} ${
        edgeLabel.negated ? '!=' : '=='
      } ${serializeExpression(edgeLabel.rhs)};`;
    case 'Reachability':
      return `${edgeLabel.negated ? '!' : '?'} ${serializeEdgeName(
        edgeLabel.lhs,
      )} -> ${serializeEdgeName(edgeLabel.rhs)};`;
    case 'Skip':
      return ';';
  }
}

export function serializeEdgeName({ parts }: ast.EdgeName) {
  // Fast path for the most common case.
  if (parts.length === 1 && parts[0].kind === 'Literal') {
    return parts[0].identifier;
  }

  return parts.map(serializeEdgeNamePart).join('');
}

export function serializeEdgeNamePart(edgeNamePart: ast.EdgeNamePart) {
  switch (edgeNamePart.kind) {
    case 'Binding':
      return `(${edgeNamePart.identifier}: ${serializeType(
        edgeNamePart.type,
      )})`;
    case 'Literal':
      return edgeNamePart.identifier;
  }
}

export function serializeExpression(expression: ast.Expression): string {
  switch (expression.kind) {
    case 'Access':
      return `${serializeExpression(expression.lhs)}[${serializeExpression(
        expression.rhs,
      )}]`;
    case 'Cast':
      return `${serializeType(expression.lhs)}(${serializeExpression(
        expression.rhs,
      )})`;
    case 'Reference':
      return expression.identifier;
  }
}

export function serializeGameDeclaration(gameDeclaration: ast.GameDeclaration) {
  return [
    ...gameDeclaration.types.map(serializeTypeDeclaration),
    ...gameDeclaration.constants.map(serializeConstantDeclaration),
    ...gameDeclaration.variables.map(serializeVariableDeclaration),
    ...gameDeclaration.edges.map(serializeEdge),
  ].join('\n');
}

export function serializeType(type: ast.Type): string {
  switch (type.kind) {
    case 'Arrow':
      return `${type.lhs} -> ${serializeType(type.rhs)}`;
    case 'Set':
      return `{ ${type.identifiers.join(', ')} }`;
    case 'TypeReference':
      return type.identifier;
  }
}

export function serializeTypeDeclaration({
  identifier,
  type,
}: ast.TypeDeclaration) {
  return `type ${identifier} = ${serializeType(type)};`;
}

export function serializeValue(value: ast.Value): string {
  switch (value.kind) {
    case 'Element':
      return value.identifier;
    case 'Map':
      return `{ ${value.entries.map(serializeValueEntry).join(', ')} }`;
  }
}

export function serializeValueEntry(valueEntry: ast.ValueEntry) {
  switch (valueEntry.kind) {
    case 'DefaultEntry':
      return `:${serializeValue(valueEntry.value)}`;
    case 'NamedEntry':
      return `${valueEntry.identifier}: ${serializeValue(valueEntry.value)}`;
  }
}

export function serializeVariableDeclaration({
  defaultValue,
  identifier,
  type,
}: ast.VariableDeclaration) {
  return `var ${identifier}: ${serializeType(type)} = ${serializeValue(
    defaultValue,
  )};`;
}
