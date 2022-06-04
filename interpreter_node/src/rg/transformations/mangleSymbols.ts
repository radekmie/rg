import * as ast from '../ast';

const reservedSymbols = [
  'Player',
  'Score',
  'begin',
  'end',
  'keeper',
  'player',
  'score',
];

export function mangleSymbols(gameDeclaration: ast.GameDeclaration) {
  // eslint-disable-next-line @typescript-eslint/ban-types -- It's fine.
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
    return (symbolsMap[identifier] ??= `${(++symbolsIndex).toString(36)}`);
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
        type.lhs = symbol(type.lhs);
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
    switch (valueEntry.kind) {
      case 'DefaultEntry':
        mangleSymbolsInValue(valueEntry.value);
        break;
      case 'NamedEntry':
        valueEntry.identifier = symbol(valueEntry.identifier);
        mangleSymbolsInValue(valueEntry.value);
        break;
    }
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
