import * as utils from '../../utils';
import { Result, success, failure } from '../../utils';
import * as ast from '../ast';
import { areObviouslyExclusive } from '../ast/lib';
import { serializeEdgeName } from '../ast/serializer';

// TODO could return information if [target] can't be reached at all (limited analysis)
/**
 * Return a subautomaton of [edges] that:
 * 1. contains [start] and [target]
 * 2. for any node except [target] contains all outgoing nodes
 * 3. contains no edges from [target]
 * 4. for any initial environment, at most one path can reach [target] from [start]
 *    - limited analysis, may reject some valid results here
 *
 * If such subautomaton can't be found, return error.
 */
export function findAcceptablePaths(
  edges: ast.EdgeDeclaration[],
  start: ast.EdgeName,
  target: ast.EdgeName,
): utils.Result<ast.EdgeDeclaration[], string> {
  const toExplore: ast.EdgeName[] = [start];
  const wasQueued: ast.EdgeName[] = [start];
  const currentPath: ast.EdgeName[] = [];
  const result: ast.EdgeDeclaration[] = [];
  while (toExplore.length > 0) {
    const current = toExplore[toExplore.length - 1];
    if (utils.isEqual(current, currentPath[currentPath.length - 1])) {
      toExplore.pop();
      currentPath.pop();
      continue;
    }
    currentPath.push(current);

    const reachable = edges.filter(e => utils.isEqual(e.lhs, current));

    if (reachable.length > 2) {
      return failure("can't ensure single path at runtime");
    }

    if (
      reachable.length === 2 &&
      !areObviouslyExclusive(reachable[0].label, reachable[1].label)
    ) {
      return failure("can't ensure single path at runtime");
    }

    for (const edge of reachable) {
      const next = edge.rhs;

      if (edge.label.kind === 'Assignment') {
        return failure('found assignment');
      }

      if (currentPath.some(ancestor => utils.isEqual(ancestor, next))) {
        return failure('found cycle');
      }

      result.push(edge);

      if (!wasQueued.some(y => utils.isEqual(y, next))) {
        toExplore.push(next);
        wasQueued.push(next);
      }
    }
  }

  if (!wasQueued.some(x => utils.isEqual(x, target))) {
    return failure("couldn't find path to target");
  }

  return success(result);
}

let freshVarId = 0;

/* TODO check whether nodes don't already contain [__gen_] prefix
 * Really the best option would be to somehow tie this state with the game description.
 * E.g.
 * > let v1 = gameDeclaration.freshVar("foo")
 * > let v2 = gameDeclaration.freshVar("bar")
 * That would also require other kinds of checks (e.g. when modyfing edges)
 */
export function freshVar(identifier = '') {
  freshVarId += 1;
  return ast.EdgeName({
    parts: [ast.Literal({ identifier: `__gen_${freshVarId}_` + identifier.replace(/[\W\s]/g, '_') })],
  });
}

/* TODO? could reuse the subgraph instead (with extra variable and comparisons)
 * Something like [--reuseFunctions]
 */
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
  pathsEnd: ast.EdgeName,
) {
  if (paths.length === 0) {
    originalEdge.label = ast.Skip({});
    return;
  }

  const originalLhs = originalEdge.lhs;
  originalEdge.label = ast.Skip({});
  originalEdge.lhs = freshVar('ignoreme');

  const mapping: [ast.EdgeName, ast.EdgeName][] = [
    [pathsStart, originalLhs],
    [pathsEnd, originalEdge.rhs],
  ];
  function getMapping(inSubgraph: ast.EdgeName): ast.EdgeName | undefined {
    return mapping.find(x => utils.isEqual(x[0], inSubgraph))?.[1];
  }
  // if the key is already present, function asserts that the present mapping is equal to [newName]
  function setMapping(inSubgraph: ast.EdgeName, newName: ast.EdgeName) {
    const found = mapping.find(x => utils.isEqual(x[0], inSubgraph));
    if (found === undefined) {
      mapping.push([inSubgraph, newName]);
    } else if (!utils.isEqual(found[1], newName)) {
      throw new Error('inlineReachability: tried to set a new mapping for a node that was already mapped');
    }
  }

  for (const e of paths) {
    const newEdge = ast.EdgeDeclaration({
      lhs: getMapping(e.lhs) || freshVar(serializeEdgeName(e.lhs)),
      rhs: getMapping(e.rhs) || freshVar(serializeEdgeName(e.rhs)),
      label: e.label,
    });
    setMapping(e.lhs, newEdge.lhs);
    setMapping(e.rhs, newEdge.rhs);
    edges.push(newEdge);
  }
}

export function inlineReachability({ edges }: ast.GameDeclaration) {
  for (const e of edges) {
    // TODO can you handle negated reachability by simply negating all labels?
    if (e.label.kind === 'Reachability' && !e.label.negated) {
      const path = findAcceptablePaths(edges, e.label.lhs, e.label.rhs);
      if (path.ok) {
        substituteWithPaths(edges, e, path.value, e.label.lhs, e.label.rhs);
      }
    }
  }
}
