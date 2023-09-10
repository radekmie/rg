import * as ast from '../ast/types';

const reservedSymbols = [
  '0',
  '1',
  'Bool',
  'Goals',
  'Player',
  'PlayerOrKeeper',
  'Score',
  'Visibility',
  'begin',
  'end',
  'goals',
  'keeper',
  'player',
  'score',
  'visible',
];

export function mangleSymbols(gameDeclaration: ast.GameDeclaration) {
  function memo<T extends object>(fn: (object: T) => void) {
    const seen = new WeakSet();
    return (object: T) => {
      if (!seen.has(object)) {
        seen.add(object);
        fn(object);
      }
    };
  }

  let symbolsIndex = 0;
  const symbolsMap: Record<string, string> = Object.create(null);
  for (const symbol of reservedSymbols) {
    symbolsMap[symbol] = symbol;
  }

  function symbol(identifier: string) {
    if (!(identifier in symbolsMap)) {
      const next = '_' + (++symbolsIndex).toString(36);
      symbolsMap[identifier] = symbolsMap[next] = next;
    }

    return symbolsMap[identifier];
  }

  const mangleSymbolsInEdgeLabel = memo((edgeLabel: ast.EdgeLabel) => {
    switch (edgeLabel.kind) {
      case 'Assignment':
        mangleSymbolsInExpression(edgeLabel.lhs);
        mangleSymbolsInExpression(edgeLabel.rhs);
        break;
      case 'Comparison':
        mangleSymbolsInExpression(edgeLabel.lhs);
        mangleSymbolsInExpression(edgeLabel.rhs);
        break;
      case 'Reachability':
        mangleSymbolsInEdgeName(edgeLabel.lhs);
        mangleSymbolsInEdgeName(edgeLabel.rhs);
        break;
      case 'Skip':
        break;
    }
  });

  const mangleSymbolsInExpression = memo((expression: ast.Expression) => {
    switch (expression.kind) {
      case 'Access':
        mangleSymbolsInExpression(expression.lhs);
        mangleSymbolsInExpression(expression.rhs);
        break;
      case 'Cast':
        mangleSymbolsInType(expression.lhs);
        mangleSymbolsInExpression(expression.rhs);
        break;
      case 'Reference':
        expression.identifier = symbol(expression.identifier);
        break;
    }
  });

  const mangleSymbolsInEdgeName = memo((edgeName: ast.EdgeName) => {
    edgeName.parts.forEach(mangleSymbolsInEdgeNamePart);
  });

  const mangleSymbolsInEdgeNamePart = memo((edgeNamePart: ast.EdgeNamePart) => {
    switch (edgeNamePart.kind) {
      case 'Binding':
        edgeNamePart.identifier = symbol(edgeNamePart.identifier);
        mangleSymbolsInType(edgeNamePart.type);
        break;
      case 'Literal':
        edgeNamePart.identifier = symbol(edgeNamePart.identifier);
        break;
    }
  });

  const mangleSymbolsInType = memo((type: ast.Type) => {
    switch (type.kind) {
      case 'Arrow':
        mangleSymbolsInType(type.lhs);
        mangleSymbolsInType(type.rhs);
        break;
      case 'Set':
        for (let index = 0; index < type.identifiers.length; ++index) {
          type.identifiers[index] = symbol(type.identifiers[index]);
        }
        break;
      case 'TypeReference':
        type.identifier = symbol(type.identifier);
        break;
    }
  });

  const mangleSymbolsInValue = memo((value: ast.Value) => {
    switch (value.kind) {
      case 'Map':
        value.entries.forEach(mangleSymbolsInValueEntry);
        break;
      case 'Element':
        value.identifier = symbol(value.identifier);
        break;
    }
  });

  const mangleSymbolsInValueEntry = memo((valueEntry: ast.ValueEntry) => {
    valueEntry.identifier &&= symbol(valueEntry.identifier);
    mangleSymbolsInValue(valueEntry.value);
  });

  for (const constantDeclaration of gameDeclaration.constants) {
    constantDeclaration.identifier = symbol(constantDeclaration.identifier);
    mangleSymbolsInType(constantDeclaration.type);
    mangleSymbolsInValue(constantDeclaration.value);
  }

  for (const edgeDeclaration of gameDeclaration.edges) {
    mangleSymbolsInEdgeName(edgeDeclaration.lhs);
    mangleSymbolsInEdgeName(edgeDeclaration.rhs);
    mangleSymbolsInEdgeLabel(edgeDeclaration.label);
  }

  for (const typeDeclaration of gameDeclaration.types) {
    typeDeclaration.identifier = symbol(typeDeclaration.identifier);
    mangleSymbolsInType(typeDeclaration.type);
  }

  for (const variableDeclaration of gameDeclaration.variables) {
    variableDeclaration.identifier = symbol(variableDeclaration.identifier);
    mangleSymbolsInType(variableDeclaration.type);
    mangleSymbolsInValue(variableDeclaration.defaultValue);
  }
}
