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
  }
}

export function serializeEdgeName(edgeName: ast.EdgeName): string {
  return edgeName.parts.map(serializeEdgeNamePart).join('');
}

export function serializeEdgeNamePart(edgeNamePart: ast.EdgeNamePart): string {
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
    ...gameDeclaration.types.map(
      typeDeclaration =>
        `type ${typeDeclaration.identifier} = ${serializeType(
          typeDeclaration.type,
        )};`,
    ),
    '',
    ...gameDeclaration.constants.map(
      constantDeclaration =>
        `const ${constantDeclaration.identifier}: ${serializeType(
          constantDeclaration.type,
        )} = ${serializeValue(constantDeclaration.value)};`,
    ),
    '',
    ...gameDeclaration.variables.map(
      variableDeclaration =>
        `var ${variableDeclaration.identifier}: ${serializeType(
          variableDeclaration.type,
        )} = ${serializeValue(variableDeclaration.defaultValue)};`,
    ),
    '',
    ...gameDeclaration.edges
      .map(
        edge =>
          `${serializeEdgeName(edge.lhs)}, ${serializeEdgeName(
            edge.rhs,
          )}: ${serializeEdgeLabel(edge.label)}`,
      )
      .reduce<string[]>(
        (xs, x) => (
          xs.length &&
            xs[xs.length - 1]
              .split('(')[0]
              .split(',')[0]
              .split('_')
              .slice(0, -1)
              .join('_') !==
              x.split('(')[0].split(',')[0].split('_').slice(0, -1).join('_') &&
            xs.push(''),
          xs.push(x),
          xs
        ),
        [],
      ),
  ].join('\n');
}

export function serializeType(type: ast.Type): string {
  switch (type.kind) {
    case 'Arrow':
      return [type.lhs, '->', serializeType(type.rhs)].join(' ');
    case 'Set':
      return ['{', type.identifiers.join(', '), '}'].join(' ');
    case 'TypeReference':
      return type.identifier;
  }
}

export function serializeValue(value: ast.Value): string {
  switch (value.kind) {
    case 'Element':
      return value.identifier;
    case 'Map':
      return [
        '{',
        value.entries
          .map(valueEntry => {
            switch (valueEntry.kind) {
              case 'DefaultEntry':
                return `:${serializeValue(valueEntry.value)}`;
              case 'NamedEntry':
                return `${valueEntry.identifier}: ${serializeValue(
                  valueEntry.value,
                )}`;
            }
          })
          .join(', '),
        '}',
      ].join(' ');
  }
}
