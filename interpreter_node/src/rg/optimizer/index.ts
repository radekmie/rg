import * as utils from '../../utils';
import * as ast from '../ast';

function bindings({ parts }: ast.EdgeName) {
  return parts.filter(function isBind(part): part is ast.Binding {
    return part.kind === 'Binding';
  });
}

function expand(gameDeclaration: ast.GameDeclaration, edgeName: ast.EdgeName) {
  const identifiers = bindings(edgeName).map(binding => binding.identifier);
  const edgeNames = bindings(edgeName)
    .map(binding =>
      expandType(gameDeclaration, binding.type).map(ast.serializeValue),
    )
    .reduce<string[][]>(utils.cartesian, [[]])
    .map<[Record<string, ast.Reference>, ast.EdgeName]>(values => [
      Object.fromEntries(
        identifiers.map((identifier, index) => [
          identifier,
          ast.Reference({ identifier: values[index] }),
        ]),
      ),
      ast.EdgeName({
        parts: [
          ast.Literal({
            identifier: edgeName.parts
              .map(part => {
                switch (part.kind) {
                  case 'Binding':
                    return values.shift()!;
                  case 'Literal':
                    return part.identifier;
                }
              })
              .join('__bind__'),
          }),
        ],
      }),
    ]);

  for (const edge of gameDeclaration.edges.slice()) {
    if (utils.isEqual(edge.lhs, edgeName)) {
      utils.remove(gameDeclaration.edges, edge);
      for (const [mapping, edgeName] of edgeNames) {
        gameDeclaration.edges.push(
          ast.EdgeDeclaration({
            lhs: edgeName,
            rhs: edge.rhs,
            label: substitute(edge.label, mapping),
          }),
        );
      }
    }

    if (utils.isEqual(edge.rhs, edgeName)) {
      utils.remove(gameDeclaration.edges, edge);
      for (const [mapping, edgeName] of edgeNames) {
        gameDeclaration.edges.push(
          ast.EdgeDeclaration({
            lhs: edge.lhs,
            rhs: edgeName,
            label: substitute(edge.label, mapping),
          }),
        );
      }
    }
  }
}

function expandType(
  gameDeclaration: ast.GameDeclaration,
  type: ast.Type,
): ast.Value[] {
  switch (type.kind) {
    case 'Arrow':
      throw new Error('Not implemented (Arrow).');
    case 'Set':
      return type.identifiers.map(identifier => ast.Element({ identifier }));
    case 'TypeReference': {
      const referencedType = utils.find(gameDeclaration.types, {
        identifier: type.identifier,
      });
      utils.assert(referencedType, `Unresolved type "${type.identifier}".`);
      return expandType(gameDeclaration, referencedType.type);
    }
  }
}

function hasBindings(edgeName: ast.EdgeName) {
  return bindings(edgeName).length !== 0;
}

function incoming(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(({ rhs }) => utils.isEqual(edgeName, rhs));
}

function isFollowing(x: ast.EdgeDeclaration, y: ast.EdgeDeclaration) {
  return utils.isEqual(x.rhs, y.lhs);
}

function isSkip({ kind }: ast.EdgeLabel) {
  return kind === 'Skip';
}

function rebind(
  edgeLabel: ast.EdgeLabel,
  edgeName: ast.EdgeName,
  mapping: Record<string, string>,
) {
  for (const binding of bindings(edgeName)) {
    if (binding.identifier in mapping) {
      binding.identifier = mapping[binding.identifier];
    }
  }

  switch (edgeLabel.kind) {
    case 'Assignment':
    case 'Comparison':
      rebindExpression(edgeLabel.lhs, mapping);
      rebindExpression(edgeLabel.rhs, mapping);
      return;
    case 'Reachability':
    case 'Skip':
      return;
  }
}

function rebindExpression(
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

function substitute(
  edgeLabel: ast.EdgeLabel,
  mapping: Record<string, ast.Reference>,
) {
  switch (edgeLabel.kind) {
    case 'Assignment':
      return ast.Assignment({
        lhs: substituteExpression(edgeLabel.lhs, mapping),
        rhs: substituteExpression(edgeLabel.rhs, mapping),
      });
    case 'Comparison':
      return ast.Comparison({
        lhs: substituteExpression(edgeLabel.lhs, mapping),
        rhs: substituteExpression(edgeLabel.rhs, mapping),
        negated: edgeLabel.negated,
      });
    case 'Reachability':
    case 'Skip':
      return edgeLabel;
  }
}

function substituteExpression(
  expression: ast.Expression,
  mapping: Record<string, ast.Reference>,
): ast.Expression {
  switch (expression.kind) {
    case 'Access':
      return ast.Access({
        lhs: substituteExpression(expression.lhs, mapping),
        rhs: substituteExpression(expression.rhs, mapping),
      });
    case 'Cast':
      return ast.Cast({
        lhs: expression.lhs,
        rhs: substituteExpression(expression.rhs, mapping),
      });
    case 'Reference':
      return expression.identifier in mapping
        ? mapping[expression.identifier]
        : expression;
  }
}

function outgoing(edges: ast.EdgeDeclaration[], edgeName: ast.EdgeName) {
  return edges.filter(({ lhs }) => utils.isEqual(edgeName, lhs));
}

// eslint-disable-next-line complexity -- It's fine.
export function compactSkipEdges({ edges }: ast.GameDeclaration) {
  // Rename all bindings so bind names are globally unique.
  let index = 0;
  for (const x of edges) {
    if (hasBindings(x.rhs)) {
      const mapping = Object.fromEntries(
        bindings(x.rhs).map(binding => [binding.identifier, `bind_${++index}`]),
      );

      for (const y of edges) {
        if (isFollowing(x, y)) {
          rebind(y.label, y.lhs, mapping);
          continue;
        }

        if (isFollowing(y, x)) {
          rebind(y.label, y.rhs, mapping);
          continue;
        }
      }

      rebind(x.label, x.rhs, mapping);
    }
  }

  // Before:
  //       x       y
  //   a ----> b ----> c
  //
  // After:
  //       x
  //   a ----> c
  //
  // Conditions:
  //   1. y = Skip
  //   2. b has no other outgoing edges
  //   3. b has no bindings
  for (const y of edges.slice()) {
    if (isSkip(y.label) && !hasBindings(y.lhs)) {
      for (const x of edges.slice()) {
        if (isFollowing(x, y) && outgoing(edges, y.lhs).every(z => z === y)) {
          utils.remove(edges, y);
          x.rhs = y.rhs;
        }
      }
    }
  }

  // Before:
  //       x       y
  //   a ----> b ----> c
  //
  // After:
  //       y
  //   a ----> c
  //
  // Conditions:
  //   1. x = Skip
  //   2. b has no other incoming edges
  //   3. b has no bindings
  for (const x of edges.slice()) {
    if (isSkip(x.label) && !hasBindings(x.rhs)) {
      for (const y of edges.slice()) {
        if (isFollowing(x, y) && incoming(edges, x.rhs).every(z => z === x)) {
          utils.remove(edges, x);
          y.lhs = x.lhs;
        }
      }
    }
  }
}

export function expandGeneratorNodes(gameDeclaration: ast.GameDeclaration) {
  const edgeNames = gameDeclaration.edges
    .flatMap(edge => [edge.lhs, edge.rhs])
    .reduce<ast.EdgeName[]>((edgeNames, edgeName) => {
      if (!edgeNames.some(other => utils.isEqual(other, edgeName))) {
        edgeNames.push(edgeName);
      }

      return edgeNames;
    }, []);

  for (const edgeName of edgeNames) {
    if (hasBindings(edgeName)) {
      expand(gameDeclaration, edgeName);
    }
  }
}
