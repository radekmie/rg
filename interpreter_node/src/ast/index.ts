import { CstNode } from 'chevrotain';

import * as ast from './types';
import visitor from './visitor';

export function buildAST(cstNode: CstNode) {
  const astNode: ast.GameDeclaration = visitor.visitNode(cstNode);
  return astNode;
}

export function serializeAST(astNode: ast.GameDeclaration) {
  function serializeEdgeLabel(edgeLabel: ast.EdgeLabel): string {
    switch (edgeLabel.kind) {
      case 'Assignment':
        return [serializeExpression(edgeLabel.lhs), '=', serializeExpression(edgeLabel.rhs)].join(' ');
      case 'Comparison':
        return [serializeExpression(edgeLabel.lhs), edgeLabel.negated ? '!=' : '==', serializeExpression(edgeLabel.rhs)].join(' ');
      case 'Reachability':
        return [edgeLabel.negated ? '!' : '?', serializeEdgeName(edgeLabel.lhs), '->', serializeEdgeName(edgeLabel.rhs)].join(' ');
      case 'Skip':
        return '';
    }
  }

  function serializeEdgeName(edgeName: ast.EdgeName): string {
    return edgeName.parts.map(serializeEdgeNamePart).join('');
  }

  function serializeEdgeNamePart(edgeNamePart: ast.EdgeNamePart): string {
    switch (edgeNamePart.kind) {
      case 'Binding':
        return ['(', edgeNamePart.identifier, ':', serializeType(edgeNamePart.type), ')'].join(' ');
      case 'Literal':
        return edgeNamePart.identifier;
    }
  }

  function serializeExpression(expression: ast.Expression): string {
    switch (expression.kind) {
      case 'Access':
        return [serializeExpression(expression.lhs), '[', serializeExpression(expression.rhs), ']'].join(' ');
      case 'Cast':
        return [serializeType(expression.lhs), '(', serializeExpression(expression.rhs), ')'].join(' ');
      case 'Reference':
        return expression.identifier;
    }
  }

  function serializeType(type: ast.Type): string {
    switch (type.kind) {
      case 'Arrow':
        return [type.lhs, '->', serializeType(type.rhs)].join(' ');
      case 'Set':
        return ['{', type.identifiers.join(', '), '}'].join(' ');
      case 'TypeReference':
        return type.identifier;
    }
  }

  function serializeValue(value: ast.Value): string {
    switch (value.kind) {
      case 'Element':
        return value.identifier;
      case 'Map':
        return ['{', value.entries
          .map(valueEntry => {
            switch (valueEntry.kind) {
              case 'DefaultEntry':
                return `:${serializeValue(valueEntry.value)}`;
              case 'NamedEntry':
                return `${valueEntry.identifier}: ${serializeValue(valueEntry.value)}`;
            }
          })
          .join(', '), '}'].join(' ');
    }
  }

  return [
    ...astNode.types.map(typeDeclaration => `type ${typeDeclaration.identifier} = ${serializeType(typeDeclaration.type)};`),
    ...astNode.constants.map(constantDeclaration => `const ${constantDeclaration.identifier}: ${serializeType(constantDeclaration.type)} = ${serializeValue(constantDeclaration.value)};`),
    ...astNode.variables.map(variableDeclaration => `var ${variableDeclaration.identifier}: ${serializeType(variableDeclaration.type)} = ${serializeValue(variableDeclaration.defaultValue)};`),
    ...astNode.edges.map(edge => [serializeEdgeName(edge.lhs), ',', serializeEdgeName(edge.rhs), ':', serializeEdgeLabel(edge.label)].join(' '))
  ].join('\n');
}
