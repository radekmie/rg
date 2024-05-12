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
    case 'Tag':
      return `$ ${edgeLabel.symbol};`;
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

export function serializeValue(value: ast.Value): string {
  switch (value.kind) {
    case 'Element':
      return value.identifier;
    case 'Map':
      return `{ ${value.entries.map(serializeValueEntry).join(', ')} }`;
  }
}

export function serializeValueEntry(valueEntry: ast.ValueEntry) {
  return valueEntry.identifier
    ? `${valueEntry.identifier}: ${serializeValue(valueEntry.value)}`
    : `:${serializeValue(valueEntry.value)}`;
}
