import * as utils from '../../utils';
import * as ast from '../ast';

type Context = {
  $assert(condition: unknown, message: string): asserts condition;
  $extend(scope: Scope): Context;
  $scope(scope: Scope): Context;
  game: ast.GameDeclaration;
  scopes: Scope[];
};

type Scope = Record<string, ast.EdgeDeclaration | ast.Expression | ast.Type>;

function inferExpression(
  context: Context,
  edge: ast.EdgeDeclaration,
  expression: ast.Expression,
): ast.Type {
  context = context.$scope({ expression });
  switch (expression.kind) {
    case 'Access': {
      const mapType = inferExpression(context, edge, expression.lhs);
      context = context.$scope({ mapType });
      context.$assert(
        mapType.kind === 'Arrow',
        'Only Arrow type can be accessed.',
      );

      const keyType = inferExpression(context, edge, expression.rhs);
      context = context.$extend({ keyType });
      context.$assert(
        keyType.kind === 'Set',
        'Only Set type can be an accessor.',
      );

      const mapKeyType = ast.TypeReference({ identifier: mapType.lhs });
      context.$assert(
        isAssignable(context, mapKeyType, keyType),
        'Incorrect accessor type.',
      );

      return resolveTypeReference(context, mapType.rhs);
    }
    case 'Cast': {
      const rhsType = inferExpression(context, edge, expression.rhs);
      context = context.$scope({
        castType: expression.lhs,
        expression: expression.rhs,
        expressionType: rhsType,
      });

      context.$assert(
        isAssignable(context, expression.lhs, rhsType),
        'Incorrect Cast.',
      );

      return resolveTypeReference(context, expression.lhs);
    }
    case 'Reference': {
      const { identifier } = expression;

      for (const edgeName of [edge.lhs, edge.rhs]) {
        const bindings = ast.lib.bindings(edgeName);
        const binding = utils.find(bindings, { identifier });
        if (binding) {
          return resolveTypeReference(context, binding.type);
        }
      }

      const constant = utils.find(context.game.constants, { identifier });
      if (constant) {
        return resolveTypeReference(context, constant.type);
      }

      const variable = utils.find(context.game.variables, { identifier });
      if (variable) {
        return resolveTypeReference(context, variable.type);
      }

      return ast.Set({ identifiers: [identifier] });
    }
  }
}

function isAssignable(context: Context, lhs: ast.Type, rhs: ast.Type): boolean {
  lhs = resolveTypeReference(context, lhs);
  rhs = resolveTypeReference(context, rhs);

  switch (lhs.kind) {
    case 'Arrow':
      return (
        rhs.kind === 'Arrow' &&
        isAssignable(context, lhs.rhs, rhs.rhs) &&
        isAssignable(
          context,
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

function resolveTypeReference(context: Context, type: ast.Type) {
  while (type.kind === 'TypeReference') {
    const typeDeclaration = utils.find(context.game.types, {
      identifier: type.identifier,
    });

    context = context.$scope({ type });
    context.$assert(typeDeclaration, 'Unresolved TypeReference.');
    type = typeDeclaration.type;
  }

  return type;
}

function typecheckEdge(context: Context, edge: ast.EdgeDeclaration) {
  if (edge.label.kind === 'Reachability' || edge.label.kind === 'Skip') {
    return;
  }

  const { lhs, rhs } = edge.label;
  const lhsType = inferExpression(context, edge, lhs);
  const rhsType = inferExpression(context, edge, rhs);
  context = context.$scope({ lhsType, rhsType });

  switch (edge.label.kind) {
    case 'Assignment':
      context.$assert(
        isAssignable(context, lhsType, rhsType),
        'Assignment type mismatch.',
      );
      break;
    case 'Comparison':
      context.$assert(
        isAssignable(context, lhsType, rhsType) ||
          isAssignable(context, rhsType, lhsType),
        'Comparison type mismatch.',
      );
      break;
  }
}

export function typecheck(game: ast.GameDeclaration) {
  const context: Context = {
    $assert(condition, message) {
      utils.assert(condition, () => {
        const lines = [message];
        if (this.game.types.length) {
          lines.push('  Type definitions:');
          this.game.types
            .slice()
            .sort((a, b) => a.identifier.localeCompare(b.identifier))
            .forEach(({ identifier, type }) => {
              lines.push(`    ${identifier}: ${ast.serializeType(type)}`);
            });
        }

        this.scopes.forEach((scope, index) => {
          const prefix = ''.padEnd(index * 2);
          Object.entries(scope).forEach(([key, value]) => {
            lines.push(
              `  ${prefix}${key}: ${
                value.kind === 'EdgeDeclaration'
                  ? ast.serializeEdge(value)
                  : value.kind === 'Arrow' ||
                    value.kind === 'Set' ||
                    value.kind === 'TypeReference'
                  ? ast.serializeType(value)
                  : ast.serializeExpression(value)
              }`,
            );
          });
        });

        return lines.join('\n');
      });
    },
    $extend(scope) {
      return {
        ...this,
        scopes: this.scopes
          .slice(0, -1)
          .concat({ ...this.scopes.slice(-1)[0], ...scope }),
      };
    },
    $scope(scope) {
      return { ...this, scopes: this.scopes.concat(scope) };
    },
    game,
    scopes: [],
  };

  for (const edge of game.edges) {
    typecheckEdge(context.$scope({ edge }), edge);
  }
}
