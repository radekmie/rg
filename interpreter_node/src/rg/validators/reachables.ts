import * as ast from '../ast';

type NextEdges = Record<string, string[]>;

export function reachables({ edges }: ast.GameDeclaration) {
  const nextEdges: NextEdges = Object.create(null);
  for (const edge of edges) {
    const lhs = ast.serializeEdgeName(edge.lhs);
    const rhs = ast.serializeEdgeName(edge.rhs);
    if (lhs in nextEdges) {
      nextEdges[lhs].push(rhs);
    } else {
      nextEdges[lhs] = [rhs];
    }
  }

  if (!isReachable(nextEdges, 'begin', 'end')) {
    throw new Error('The "end" node is not reachable from the "begin" node');
  }

  for (const edge of edges) {
    if (edge.label.kind === 'Reachability') {
      const from = ast.serializeEdgeName(edge.label.lhs);
      const to = ast.serializeEdgeName(edge.label.rhs);
      if (!isReachable(nextEdges, from, to)) {
        throw new Error(`Incorrect reachability: "${ast.serializeEdge(edge)}"`);
      }
    }
  }
}

function isReachable(nextEdges: NextEdges, from: string, to: string) {
  const seen = new Set<string>();
  const queue = [from];
  while (queue.length) {
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion -- Length is checked above.
    const lhs = queue.pop()!;
    if (lhs in nextEdges) {
      for (const rhs of nextEdges[lhs]) {
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
