import * as utils from '../../utils';
import * as ast from '../ast';

// TODO could return information if [target] can't be reached at all (limited analysis)
/* TODO update implementation to specification:
 * Return a subgraph of [edges] that:
 * 1. contains [start] and [target]
 * 2. for any vertex except [target] contains all outgoing nodes
 * 3. contains no edges from [target]
 * 4. for any initial environment, at most one path can reach [target] from [start]
 *    - limited analysis, may reject some valid results here
 *
 * If such subgraph can't be found, return error.
 */
export function findAcceptablePaths(
  edges: ast.EdgeDeclaration[],
  start: ast.EdgeName,
  target: ast.EdgeName,
): ast.EdgeDeclaration[] | string {
  let x: ast.EdgeName = start;
  const visited: ast.EdgeName[] = [];
  const paths: ast.EdgeDeclaration[] = [];
  // TODO when is '===' instead of 'isEqual' enough?
  while (!utils.isEqual(x, target)) {
    // no loops because they increase the number of possible paths
    if (visited.some(y => utils.isEqual(y, x))) {
      return 'nope';
    }

    const reachable = edges.filter(e => utils.isEqual(e.lhs, x));

    if (reachable.length != 1) {
      return 'nope';
    }

    const edge = reachable[0];

    if (edge.label.kind === 'Assignment') {
      return 'nope';
    }

    paths.push(edge);
    visited.push(x);
    x = edge.rhs;
  }

  return paths;
}

export function freshVar() {
  // TODO
  return ast.EdgeName({ parts: [ast.Literal({ identifier: '__gen_1' })] });
}

// TODO? could reuse the subgraph instead (with extra variable and comparisons)
/* TODO need some naming convention to differentiate nodes from [edges] and [paths]
 * (original graph vs subgraph)
 */
export function substituteWithPaths(
  edges: ast.EdgeDeclaration[],
  originalEdge: ast.EdgeDeclaration,
  paths: ast.EdgeDeclaration[],
  pathsStart: ast.EdgeName,
  pathsEnd: ast.EdgeName
) {
  if (paths.length == 0) {
    originalEdge.label = ast.Skip({});
    return;
  }

  // TODO would replacing the original edge with skip be better? (paths would start from some new node)
  edges.splice(edges.indexOf(originalEdge), 1);

  // pairs: [original node (in subgraph), mapped node (replacing reachability edge)]
  let queue: [ast.EdgeName, ast.EdgeName][] = [[pathsStart, originalEdge.lhs]];

  while (queue.length > 0) {
    let packed = queue.shift()
    if (typeof packed === 'undefined') { throw new Error("queue was non-empty but didn't have element") }
    let [last, lastFresh] = packed
    if (utils.isEqual(last, pathsEnd)) { throw new Error("target shouldn't be in queue") }

    let branches = paths.filter(e => utils.isEqual(e.lhs, last))
    for (const fromSubgraph of branches) {
      const created = ast.EdgeDeclaration({
        lhs: lastFresh,
        // TODO could pass [fromSubgraph.rhs] to [freshVar] for more descriptive name
        rhs: utils.isEqual(fromSubgraph.rhs, pathsEnd) ? originalEdge.rhs : freshVar(), 
        label: fromSubgraph.label
      })
      edges.push(created)
      // TODO are you sure this isn't needed? (if not then explain)
      if(!utils.isEqual(fromSubgraph.rhs, pathsEnd))
        queue.push([fromSubgraph.rhs, created.rhs])
    }
  }
}

export function inlineReachability({ edges }: ast.GameDeclaration) {
  for (const e of edges) {
    // TODO can you handle negated reachability by simply negating all labels?
    if (e.label.kind === 'Reachability' && !e.label.negated) {
      const path = findAcceptablePaths(edges, e.label.lhs, e.label.rhs);
      if (typeof path === 'object') {
        substituteWithPaths(edges, e, path, e.label.lhs, e.label.rhs);
      } else {
        // TODO hey, maybe remove the edge then?
        console.log("can't find path, msg:" + path);
      }
    }
  }
}
