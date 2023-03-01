import * as ast from './types';
import * as utils from '../../utils';

export function areObviouslyExclusive(
  a: ast.EdgeLabel,
  b: ast.EdgeLabel,
): boolean {
  if (a.kind === 'Comparison' && b.kind === 'Comparison') {
    if (a.negated === b.negated) {
      return false;
    }

    const exact = utils.isEqual(a.lhs, b.lhs) && utils.isEqual(a.rhs, b.rhs);
    if (exact) {
      return true;
    }

    const cross = utils.isEqual(a.lhs, b.rhs) && utils.isEqual(a.rhs, b.lhs);
    if (cross) {
      return true;
    }

    // TODO: Are there more cases?
    return false;
  }

  if (a.kind === 'Reachability' && b.kind === 'Reachability') {
    return (
      a.negated !== b.negated &&
      utils.isEqual(a.lhs, b.lhs) &&
      utils.isEqual(a.rhs, b.rhs)
    );
  }

  return false;
}

export function bindings({ parts }: ast.EdgeName) {
  return parts.filter(function isBind(part): part is ast.Binding {
    return part.kind === 'Binding';
  });
}

export function collectEdgeNames(edges: ast.EdgeDeclaration[]): ast.EdgeName[] {
  return edges.flatMap(e => {
    const l = e.label;
    return [e.lhs, e.rhs].concat(
      l.kind === 'Reachability' ? [l.lhs, l.rhs] : [],
    );
  });
}

export function hasBindings({ parts }: ast.EdgeName) {
  return parts.some(part => part.kind === 'Binding');
}

export function incoming(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(({ rhs }) => utils.isEqual(edgeName, rhs));
}

export function isFollowing(x: ast.EdgeDeclaration, y: ast.EdgeDeclaration) {
  return utils.isEqual(x.rhs, y.lhs);
}

export function isReachabilityTarget(
  x: ast.EdgeName,
  edges: ast.EdgeDeclaration[],
) {
  return edges.some(
    e =>
      e.label.kind === 'Reachability' &&
      (utils.isEqual(e.label.lhs, x) || utils.isEqual(e.label.rhs, x)),
  );
}

export function isSkip(edgeLabel: ast.EdgeLabel): edgeLabel is ast.Skip {
  return edgeLabel.kind === 'Skip';
}

// TODO add tests (verify that it doesn't create duplicate nodes)
/** Generator for fresh node names: 'makeFreshEdgeName(edges)(referenceString?)' -- can be partially applied to check name conflicts only during initial application (assumes no other '__gen_' identifiers are added after that).
 * @param {ast.EdgeDeclaration[]} edges - the automaton for which fresh identifiers will be created
 * @returns {freshVarGenerator} generator for fresh nodes
 */
export function makeFreshEdgeName(edges: ast.EdgeDeclaration[]) {
  const pattern = (id: string, extra: string) => `__gen_${id}_${extra}`;
  const matcher = new RegExp(pattern('(?<num>d+)', '.*'));

  const generate = makeFreshName(
    pattern,
    collectEdgeNames(edges).map(name => {
      if (name.parts.length === 1 && name.parts[0].kind === 'Literal') {
        return name.parts[0].identifier.match(matcher)?.groups?.num;
      }
    }),
  );

  /** Creates fresh identifiers for given game context assuming no other nodes with '__gen_' prefix are added.
   * @param {string?} reference - string that will be appended (in some way) to the created node for reference
   * @returns {ast.EdgeName} node with a unique identifier
   */
  return function freshVarGenerator(reference = '') {
    const identifier = generate(reference);
    return ast.EdgeName({ parts: [ast.Literal({ identifier })] });
  };
}

export function makeFreshName(
  pattern: (id: string, extra: string) => string,
  names: (string | undefined)[],
) {
  let freshVarId = Number(
    names.reduce<string>(
      (acc, x) => (x === undefined || acc > x ? acc : x),
      '0',
    ),
  );

  return function freshNameGenerator(reference = '') {
    freshVarId += 1;
    return pattern(freshVarId.toString(), reference.replace(/[\W\s]/g, '_'));
  };
}

export function outgoing(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(({ lhs }) => utils.isEqual(edgeName, lhs));
}

export function typeValues(
  gameDeclaration: ast.GameDeclaration,
  type: ast.Type,
): ast.Value[] {
  switch (type.kind) {
    case 'Arrow':
      throw new Error('Not implemented (Arrow).');
    case 'Set':
      return type.identifiers.map(identifier => ast.Element({ identifier }));
    case 'TypeReference': {
      const typeDeclaration = utils.find(gameDeclaration.types, {
        identifier: type.identifier,
      });
      utils.assert(typeDeclaration, `Unresolved type "${type.identifier}".`);
      return typeValues(gameDeclaration, typeDeclaration.type);
    }
  }
}
