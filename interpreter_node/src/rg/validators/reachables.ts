import * as ast from '../ast';

export function reachables({ edges }: ast.GameDeclaration) {
  if (!isReachable(edges, 'begin', 'end')) {
    throw new Error('The "end" node is not reachable from the "begin" node');
  }

  for (const edge of edges) {
    if (edge.label.kind === 'Reachability') {
      const from = ast.serializeEdgeName(edge.label.lhs);
      const to = ast.serializeEdgeName(edge.label.rhs);
      if (!isReachable(edges, from, to)) {
        throw new Error(`Incorrect reachability: "${ast.serializeEdge(edge)}"`);
      }
    }
  }
}

function isReachable(edges: ast.EdgeDeclaration[], from: string, to: string) {
  const seen = new Set<string>();
  const queue = [from];
  while (queue.length) {
    const position = queue.pop()!;
    for (const edge of edges) {
      if (position === ast.serializeEdgeName(edge.lhs)) {
        const rhs = ast.serializeEdgeName(edge.rhs);
        if (!seen.has(rhs)) {
          if (rhs === to) {
            return true;
          }

          seen.add(rhs);
          queue.push(rhs);
        }
      }
    }
  }

  return false;
}
