import * as ast from '../ast/types';

function compactAutomaton(edges: ast.EdgeDeclaration[]) {
  // eslint-disable-next-line no-constant-condition -- Fixpoint approach is easier.
  while (true) {
    const edgesLength = edges.length;
    for (const edgeA of edges) {
      if (edgeA.label.kind === 'Skip') {
        if (
          edgeA.lhs.parts.length === 1 &&
          edgeA.lhs.parts[0].kind === 'Literal'
        ) {
          for (const edgeB of edges) {
            if (JSON.stringify(edgeA.lhs) === JSON.stringify(edgeB.rhs)) {
              edgeB.rhs = edgeA.rhs;
              if (edges.includes(edgeA)) {
                edges.splice(edges.indexOf(edgeA), 1);
              }
            }
          }
        }
      }
    }

    if (edgesLength === edges.length) {
      break;
    }
  }
}

export function optimize(gameDeclaration: ast.GameDeclaration) {
  compactAutomaton(gameDeclaration.edges);
}
