import * as utils from '../../utils';
import * as ast from './types';

export function areBindingsUnique(edges: ast.EdgeDeclaration[]) {
  const bindingToEdgeName = new Map<string, ast.EdgeName>();
  for (const { lhs, rhs } of edges) {
    for (const edgeName of [lhs, rhs]) {
      for (const { identifier } of bindings(edgeName)) {
        if (bindingToEdgeName.has(identifier)) {
          if (!utils.isEqual(edgeName, bindingToEdgeName.get(identifier))) {
            return false;
          }
        } else {
          bindingToEdgeName.set(identifier, edgeName);
        }
      }
    }
  }

  return true;
}

export function areObviouslyExclusive(
  a: ast.EdgeLabel,
  b: ast.EdgeLabel,
): boolean {
  if (a.kind === 'Comparison' && b.kind === 'Comparison') {
    const argsMatch = function () {
      return utils.isEqual(a.lhs, b.lhs) && utils.isEqual(a.rhs, b.rhs);
    };
    const argsMatchCrossed = function () {
      return utils.isEqual(a.lhs, b.rhs) && utils.isEqual(a.rhs, b.lhs);
    };
    return a.negated !== b.negated && (argsMatch() || argsMatchCrossed());
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

export function hasConnection(
  edges: ast.EdgeDeclaration[],
  lhs: ast.EdgeName,
  rhs: ast.EdgeName,
) {
  return edges.some(
    edge => utils.isEqual(edge.lhs, lhs) && utils.isEqual(edge.rhs, rhs),
  );
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

export function makeBindingsUnique(edges: ast.EdgeDeclaration[]) {
  let index = 0;
  for (const x of edges) {
    if (hasBindings(x.rhs)) {
      const mapping = utils.mapToObject(bindings(x.rhs), binding => [
        binding.identifier,
        `bind_${++index}`,
      ]);

      for (const y of edges) {
        if (x !== y) {
          if (isFollowing(x, y) || utils.isEqual(x.lhs, y.lhs)) {
            renameInEdgeLabel(y.label, mapping);
            renameInEdgeName(y.lhs, mapping);
          }

          if (isFollowing(y, x) || utils.isEqual(x.rhs, y.rhs)) {
            renameInEdgeLabel(y.label, mapping);
            renameInEdgeName(y.rhs, mapping);
          }
        }
      }

      renameInEdgeLabel(x.label, mapping);
      renameInEdgeName(x.lhs, mapping);
      renameInEdgeName(x.rhs, mapping);
    }
  }
}

// TODO add tests (verify that it doesn't create duplicate nodes)
/** Generator for fresh node names: 'makeFreshEdgeName(edges)(referenceString?)' -- can be partially applied to check name conflicts only during initial application (assumes no other '__gen_' identifiers are added after that).
 * @param {ast.EdgeDeclaration[]} edges - the automaton for which fresh identifiers will be created
 * @returns {freshVarGenerator} generator for fresh nodes
 */
export function makeFreshEdgeName(edges: ast.EdgeDeclaration[]): (reference: string | undefined) => ast.EdgeName {
  const pattern = (id: string, extra: string) => `__gen_${id}_${extra}`;
  const matcher = new RegExp(pattern('(?<num>d+)', '.*'));

  let g = makeFreshName(pattern, collectEdgeNames(edges).map(name => {
    if (name.parts.length === 1 && name.parts[0].kind === 'Literal') {
      return name.parts[0].identifier.match(matcher)?.groups?.num;
    }
  }));

  /** Creates fresh identifiers for given game context assuming no other nodes with '__gen_' prefix are added.
   * @name freshVarGenerator
   * @function
   * @param {string?} reference - string that will be appended (in some way) to the created node for reference
   * @returns {ast.EdgeName} node with a unique identifier
   */
  return function (reference = '') {
    return ast.EdgeName({
      parts: [
        ast.Literal({
          identifier: g(reference)
        }),
      ],
    });
  };
}

export function makeFreshName(
  pattern: (id: string, extra: string) => string,
  names: (string | undefined)[],
): (reference: string | undefined) => string {
  let freshVarId = Number(
    names.reduce<string>((acc, x) => (x === undefined || acc > x ? acc : x), '0'),
  );

  return function (reference = '') {
    freshVarId += 1;
    return pattern(
      freshVarId.toString(),
      reference.replace(/[\W\s]/g, '_'),
    )
  };
}

export function outgoing(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(({ lhs }) => utils.isEqual(edgeName, lhs));
}

export function renameInEdgeDeclaration(
  edge: ast.EdgeDeclaration,
  mapping: Record<string, string>,
) {
  renameInEdgeLabel(edge.label, mapping);
  renameInEdgeName(edge.lhs, mapping);
  renameInEdgeName(edge.rhs, mapping);
}

export function renameInEdgeLabel(
  edgeLabel: ast.EdgeLabel,
  mapping: Record<string, string>,
) {
  switch (edgeLabel.kind) {
    case 'Assignment':
      rebindExpression(edgeLabel.lhs, mapping);
      rebindExpression(edgeLabel.rhs, mapping);
      return;
    case 'Comparison':
      rebindExpression(edgeLabel.lhs, mapping);
      rebindExpression(edgeLabel.rhs, mapping);
      return;
    case 'Reachability':
      return;
    case 'Skip':
      return;
  }
}

export function renameInEdgeName(
  edgeName: ast.EdgeName,
  mapping: Record<string, string>,
) {
  for (const binding of bindings(edgeName)) {
    if (binding.identifier in mapping) {
      binding.identifier = mapping[binding.identifier];
    }
  }
}

export function rebindExpression(
  expression: ast.Expression,
  mapping: Record<string, string>,
) {
  switch (expression.kind) {
    case 'Access':
      rebindExpression(expression.lhs, mapping);
      rebindExpression(expression.rhs, mapping);
      return;
    case 'Cast':
      rebindExpression(expression.rhs, mapping);
      return;
    case 'Reference':
      if (expression.identifier in mapping) {
        expression.identifier = mapping[expression.identifier];
      }
      return;
  }
}

export function substituteInEdgeLabel(
  edgeLabel: ast.EdgeLabel,
  mapping: Record<string, ast.Expression>,
) {
  switch (edgeLabel.kind) {
    case 'Assignment':
      return ast.Assignment({
        lhs: substituteInExpression(edgeLabel.lhs, mapping),
        rhs: substituteInExpression(edgeLabel.rhs, mapping),
      });
    case 'Comparison':
      return ast.Comparison({
        lhs: substituteInExpression(edgeLabel.lhs, mapping),
        rhs: substituteInExpression(edgeLabel.rhs, mapping),
        negated: edgeLabel.negated,
      });
    case 'Reachability':
    case 'Skip':
      return edgeLabel;
  }
}

export function substituteInExpression(
  expression: ast.Expression,
  mapping: Record<string, ast.Expression>,
): ast.Expression {
  switch (expression.kind) {
    case 'Access':
      return ast.Access({
        lhs: substituteInExpression(expression.lhs, mapping),
        rhs: substituteInExpression(expression.rhs, mapping),
      });
    case 'Cast':
      return ast.Cast({
        lhs: expression.lhs,
        rhs: substituteInExpression(expression.rhs, mapping),
      });
    case 'Reference':
      return expression.identifier in mapping
        ? mapping[expression.identifier]
        : expression;
  }
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
