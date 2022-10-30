import * as ast from './types';

export function serializeAction(action: ast.Action) {
  switch (action.kind) {
    case 'Assignment':
      return `[$ ${action.variable} = ${serializeValue(action.value)}]`;
    case 'Check':
      return `{${action.negated ? '!' : '?'} ${serializeRule(action.rule)}}`;
    case 'Comparison': {
      const lhs = serializeValue(action.lhs);
      const rhs = serializeValue(action.rhs);
      return `{$ ${lhs} ${action.operator} ${rhs} }`;
    }
    case 'Off':
      return `[${action.piece}]`;
    case 'On':
      return `{${action.pieces.join(', ')}}`;
    case 'Shift':
      return action.label;
    case 'Switch':
      return `->${action.player ?? '>'}`;
  }
}

export function serializeAtom(atom: ast.Atom) {
  const content =
    atom.content.kind === 'Rule'
      ? `(${serializeRule(atom.content)})`
      : serializeAction(atom.content);
  return atom.power ? `${content} *` : content;
}

export function serializeEdge(edge: ast.Edge) {
  return `${edge.label}: ${edge.node}`;
}

export function serializeGame(game: ast.Game) {
  return [
    `#pieces = ${game.pieces.join(', ')}`,
    `#variables = ${game.variables.map(serializeVariable).join(', ')}`,
    `#players = ${game.players.map(serializeVariable).join(', ')}`,
    `#board =\n  ${game.board.map(serializeNode).join('\n  ')}`,
    `#rules = ${serializeRule(game.rules)}`,
  ].join('\n');
}

export function serializeNode(node: ast.Node) {
  const edges = node.edges.map(serializeEdge).join(', ');
  return `${node.node} [${node.piece}] { ${edges} }`;
}

export function serializeRule(rule: ast.Rule): string {
  return rule.elements
    .map(atoms => atoms.map(serializeAtom).join(' '))
    .join(' + ');
}

export function serializeValue(value: ast.Value): string {
  switch (typeof value) {
    case 'number':
      return `${value}`;
    case 'string':
      return value;
  }

  const lhs = serializeValue(value.lhs);
  const rhs = serializeValue(value.rhs);
  return `(${lhs} ${value.operator} ${rhs})`;
}

export function serializeVariable(variable: ast.Variable) {
  return `${variable.name}(${variable.bound})`;
}
