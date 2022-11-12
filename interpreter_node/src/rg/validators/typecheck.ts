import * as utils from '../../utils';
import * as ast from '../ast';

function inferExpression(
  game: ast.GameDeclaration,
  edge: ast.EdgeDeclaration,
  expression: ast.Expression,
): ast.Type {
  switch (expression.kind) {
    case 'Access': {
      const map = inferExpression(game, edge, expression.lhs);
      utils.assert(map.kind === 'Arrow', 'Only Arrow type can be accessed.');
      const key = inferExpression(game, edge, expression.rhs);
      utils.assert(key.kind === 'Set', 'Only Set type can be a key.');
      const keys = ast.TypeReference({ identifier: map.lhs });
      utils.assert(isAssignable(game, keys, key), 'Incorrect Access.');
      return ast.lib.resolveTypeReference(game, map.rhs);
    }
    case 'Cast': {
      const rhs = inferExpression(game, edge, expression.rhs);
      utils.assert(isAssignable(game, expression.lhs, rhs), 'Incorrect Cast.');
      return ast.lib.resolveTypeReference(game, expression.lhs);
    }
    case 'Reference': {
      const { identifier } = expression;

      for (const edgeName of [edge.lhs, edge.rhs]) {
        const bindings = ast.lib.bindings(edgeName);
        const binding = utils.find(bindings, { identifier });
        if (binding) {
          return ast.lib.resolveTypeReference(game, binding.type);
        }
      }

      const constant = utils.find(game.constants, { identifier });
      if (constant) {
        return ast.lib.resolveTypeReference(game, constant.type);
      }

      const variable = utils.find(game.variables, { identifier });
      if (variable) {
        return ast.lib.resolveTypeReference(game, variable.type);
      }

      return ast.Set({ identifiers: [identifier] });
    }
  }
}

function isAssignable(
  game: ast.GameDeclaration,
  lhs: ast.Type,
  rhs: ast.Type,
): boolean {
  lhs = ast.lib.resolveTypeReference(game, lhs);
  rhs = ast.lib.resolveTypeReference(game, rhs);

  switch (lhs.kind) {
    case 'Arrow':
      return (
        rhs.kind === 'Arrow' &&
        isAssignable(game, lhs.rhs, rhs.rhs) &&
        isAssignable(
          game,
          ast.TypeReference({ identifier: rhs.lhs }),
          ast.TypeReference({ identifier: lhs.lhs }),
        )
      );
    case 'Set':
      return (
        rhs.kind === 'Set' && utils.isSubset(rhs.identifiers, lhs.identifiers)
      );
  }
}

function typecheckEdge(game: ast.GameDeclaration, edge: ast.EdgeDeclaration) {
  switch (edge.label.kind) {
    case 'Assignment': {
      const lhs = inferExpression(game, edge, edge.label.lhs);
      const rhs = inferExpression(game, edge, edge.label.rhs);
      utils.assert(isAssignable(game, lhs, rhs), 'Type mismatch (Assignment).');
      break;
    }
    case 'Comparison': {
      const lhs = inferExpression(game, edge, edge.label.lhs);
      const rhs = inferExpression(game, edge, edge.label.rhs);
      utils.assert(
        isAssignable(game, lhs, rhs) || isAssignable(game, rhs, lhs),
        'Type mismatch (Comparison).',
      );
      break;
    }
    case 'Reachability':
      break;
    case 'Skip':
      break;
  }
}

export function typecheck(game: ast.GameDeclaration) {
  for (const edge of game.edges) {
    typecheckEdge(game, edge);
  }
}
