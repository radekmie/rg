import * as utils from '../../utils';
import * as ast from '../ast';
import { serializeEdgeName } from '../ast/serializer';

function areObviouslyExclusive(a: ast.EdgeLabel, b: ast.EdgeLabel): boolean {
  if (a.kind == 'Comparison' && b.kind == 'Comparison') {
    // TODO
    return false
  }

  if (a.kind == 'Reachability' && b.kind == 'Reachability') {
    return a.negated !== b.negated && utils.isEqual(a.lhs, b.lhs) && utils.isEqual(a.rhs, b.rhs)
  }

  return false
}

// TODO could return information if [target] can't be reached at all (limited analysis)
/* Return a subgraph of [edges] that:
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
  const toExplore: ast.EdgeName[] = [start];
  const wasQueued: ast.EdgeName[] = [start];
  const currentPath: ast.EdgeName[] = [];
  const result: ast.EdgeDeclaration[] = [];
  while (toExplore.length > 0) {
    const current = toExplore[toExplore.length - 1]
    if (utils.isEqual(current, currentPath[currentPath.length - 1])) {
      toExplore.pop()
      currentPath.pop()
      continue
    }
    currentPath.push(current)

    const reachable = edges.filter(e => utils.isEqual(e.lhs, current))

    if (reachable.length > 2)
      return "can't ensure single path at runtime"

    if (reachable.length == 2 && !areObviouslyExclusive(reachable[0].label, reachable[1].label))
      return "can't ensure single path at runtime"

    for(const edge of reachable) {
      const next = edge.rhs

      if (edge.label.kind === 'Assignment') {
        return 'found assignment'
      }

      if (currentPath.some(ancestor => utils.isEqual(ancestor, next)))
        return 'found cycle'

      result.push(edge)

      if (!wasQueued.some(y => utils.isEqual(y, next))) {
        toExplore.push(next)
        wasQueued.push(next)
      }
    }
  }

  if (!wasQueued.some(x => utils.isEqual(x, target)))
    return "couldn't find path to target"

  return result;
}

export function freshVar(identifier: string = "") {
  // TODO ensure it's unique
  return ast.EdgeName({ parts: [ast.Literal({ identifier: '__gen_1_' + identifier })] });
}

// TODO? could reuse the subgraph instead (with extra variable and comparisons)
/* TODO need some naming convention to differentiate nodes from [edges] and [paths]
 * (original graph vs subgraph)
 */
// Replace [originalEdge] in [edges] with a copy of all edges in [paths] while mapping
// [pathsStart] to [originalEdge.lhs] and [pathsEnd] to [originalEdge.rhs]
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

  // TODO? wanted this to be a map but can't find how to create Map<object, ...> that works
  const mapping: [ast.EdgeName, ast.EdgeName][] = [[pathsStart, originalEdge.lhs], [pathsEnd, originalEdge.rhs]]
  function getMapping(inSubgraph: ast.EdgeName): ast.EdgeName | undefined {
    return mapping.find(x => utils.isEqual(x[0], inSubgraph))?.at(1)
  }
  // if the key is already present, function asserts that the present mapping is equal to [newName]
  function setMapping(inSubgraph: ast.EdgeName, newName: ast.EdgeName) {
    const place = mapping.findIndex(x => utils.isEqual(x[0], inSubgraph))
    if (place < 0) {
      mapping.push([inSubgraph, newName])
    } else {
      if (!utils.isEqual(mapping[place][1], newName)) {
        throw new Error("help me")
      }
    }
  }

  for (const e of paths) {
    const newEdge = ast.EdgeDeclaration({
      lhs: getMapping(e.lhs) || freshVar(serializeEdgeName(e.lhs)),
      rhs: getMapping(e.rhs) || freshVar(serializeEdgeName(e.rhs)),
      label: e.label
    })
    setMapping(e.lhs, newEdge.lhs)
    setMapping(e.rhs, newEdge.rhs)
    edges.push(newEdge)
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
