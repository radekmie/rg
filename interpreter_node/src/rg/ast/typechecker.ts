import * as utils from '../../utils';
import * as ast from '../ast';

type Scope = Record<string, ast.EdgeDeclaration | ast.Expression | ast.Type>;

export class TypeChecker {
  private inferedTypes: Record<string, ast.Type> = Object.create(null);
  private normalizedTypes: Record<string, ast.Type> = Object.create(null);
  private scopes: Scope[] = [];

  constructor(private gameDeclaration: ast.GameDeclaration) {}

  addExplicitCasts(edge: ast.EdgeDeclaration) {
    if (edge.label.kind === 'Reachability' || edge.label.kind === 'Skip') {
      return;
    }

    this.scopeEntry({ edge });
    const lhsType = this.inferExpression(edge, edge.label.lhs);
    edge.label.lhs = this.addExplicitCastsInner(edge, edge.label.lhs, lhsType);
    edge.label.rhs = this.addExplicitCastsInner(edge, edge.label.rhs, lhsType);
    this.scopeExit(undefined);
  }

  private addExplicitCastsInner(
    edge: ast.EdgeDeclaration,
    expression: ast.Expression,
    type: ast.Type,
  ): ast.Expression {
    this.scopeEntry({ expression });

    // Add casts to subexpressions.
    switch (expression.kind) {
      case 'Access': {
        const lhsType = this.inferExpression(edge, expression.lhs);
        this.scopeEntry({ lhsType });
        this.assert(
          lhsType.kind === 'Arrow',
          'Only Arrow type can be accessed.',
        );

        const rhsType = ast.TypeReference({ identifier: lhsType.lhs });
        this.scopeExtend({ lhsType });
        expression = ast.Access({
          lhs: this.addExplicitCastsInner(edge, expression.lhs, lhsType),
          rhs: this.addExplicitCastsInner(edge, expression.rhs, rhsType),
        });
        this.scopeExit(undefined, 1);
        break;
      }
      case 'Cast': {
        expression = ast.Cast({
          lhs: expression.lhs,
          rhs: this.addExplicitCastsInner(edge, expression.rhs, type),
        });
        break;
      }
    }

    // Cast the whole expression if there's a type to reference to.
    const typeDeclaration = this.resolveTypeDeclaration(type);
    if (typeDeclaration) {
      expression = ast.Cast({
        lhs: ast.TypeReference({ identifier: typeDeclaration.identifier }),
        rhs: expression,
      });
    }

    // Strip duplicated casts.
    while (
      expression.kind === 'Cast' &&
      expression.rhs.kind === 'Cast' &&
      utils.isEqual(expression.lhs, expression.rhs.lhs)
    ) {
      expression = expression.rhs;
    }

    return this.scopeExit(expression, 1);
  }

  assert(condition: unknown, message: string): asserts condition {
    utils.assert(condition, () => {
      const lines = [message];
      if (this.gameDeclaration.types.length) {
        lines.push('  Type definitions:');
        this.gameDeclaration.types
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
  }

  checkEdge(edge: ast.EdgeDeclaration) {
    if (edge.label.kind === 'Reachability' || edge.label.kind === 'Skip') {
      return;
    }

    this.scopeEntry({ edge });
    const lhs = this.inferExpression(edge, edge.label.lhs);
    const rhs = this.inferExpression(edge, edge.label.rhs);
    this.scopeEntry({ lhsType: lhs, rhsType: rhs });

    switch (edge.label.kind) {
      case 'Assignment':
        this.assert(this.isAssignable(lhs, rhs), 'Assignment type mismatch.');
        break;
      case 'Comparison':
        this.assert(
          this.isAssignable(lhs, rhs) || this.isAssignable(rhs, lhs),
          'Comparison type mismatch.',
        );
        break;
    }

    this.scopeExit(undefined, 2);
  }

  inferExpression(edge: ast.EdgeDeclaration, expression: ast.Expression) {
    if (ast.lib.hasBindings(edge.lhs) || ast.lib.hasBindings(edge.rhs)) {
      return this.inferExpressionInner(edge, expression);
    }

    const key = ast.serializeExpression(expression);
    this.inferedTypes[key] ??= this.inferExpressionInner(edge, expression);
    return this.inferedTypes[key];
  }

  private inferExpressionInner(
    edge: ast.EdgeDeclaration,
    expression: ast.Expression,
  ): ast.Arrow | ast.Set {
    this.scopeEntry({ expression });
    switch (expression.kind) {
      case 'Access': {
        const mapType = this.inferExpression(edge, expression.lhs);
        this.scopeEntry({ mapType });
        this.assert(
          mapType.kind === 'Arrow',
          'Only Arrow type can be accessed.',
        );

        const keyType = this.inferExpression(edge, expression.rhs);
        this.scopeExtend({ keyType });
        this.assert(
          keyType.kind === 'Set',
          'Only Set type can be an accessor.',
        );

        const mapKeyType = ast.TypeReference({ identifier: mapType.lhs });
        this.assert(
          this.isAssignable(mapKeyType, keyType),
          'Incorrect accessor type.',
        );

        return this.scopeExit(this.resolveTypeReference(mapType.rhs), 2);
      }
      case 'Cast': {
        const rhsType = this.inferExpression(edge, expression.rhs);
        this.scopeEntry({
          castType: expression.lhs,
          expression: expression.rhs,
          expressionType: rhsType,
        });

        this.assert(
          this.isAssignable(expression.lhs, rhsType),
          'Incorrect Cast.',
        );

        return this.scopeExit(this.resolveTypeReference(expression.lhs), 2);
      }
      case 'Reference': {
        const { identifier } = expression;
        for (const edgeName of [edge.lhs, edge.rhs]) {
          const bindings = ast.lib.bindings(edgeName);
          const binding = utils.find(bindings, { identifier });
          if (binding) {
            return this.scopeExit(this.resolveTypeReference(binding.type));
          }
        }

        const { constants, variables } = this.gameDeclaration;
        const constant = utils.find(constants, { identifier });
        if (constant) {
          return this.scopeExit(this.resolveTypeReference(constant.type));
        }

        const variable = utils.find(variables, { identifier });
        if (variable) {
          return this.scopeExit(this.resolveTypeReference(variable.type));
        }

        return ast.Set({ identifiers: [identifier] });
      }
    }
  }

  isAssignable(lhs: ast.Type, rhs: ast.Type): boolean {
    lhs = this.resolveTypeReference(lhs);
    rhs = this.resolveTypeReference(rhs);

    // Fast path for defined types.
    if (lhs === rhs) {
      return true;
    }

    switch (lhs.kind) {
      case 'Arrow':
        return (
          rhs.kind === 'Arrow' &&
          this.isAssignable(lhs.rhs, rhs.rhs) &&
          this.isAssignable(
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

  normalizeType(type: ast.Type) {
    const key = ast.serializeType(type);
    this.normalizedTypes[key] ??= this.normalizeTypeInner(type);
    return this.normalizedTypes[key];
  }

  private normalizeTypeInner(type: ast.Type): ast.Type {
    // References are already normalized.
    if (type.kind === 'TypeReference') {
      return type;
    }

    // Normalize subtype.
    if (type.kind === 'Arrow') {
      type = ast.Arrow({ lhs: type.lhs, rhs: this.normalizeType(type.rhs) });
    }

    // If there's a declaration of this type already, just use it.
    const typeDeclaration = this.resolveTypeDeclaration(type);
    if (typeDeclaration) {
      return ast.TypeReference({ identifier: typeDeclaration.identifier });
    }

    // Generate unique identifier for the normalize type.
    const identifier = utils.generateIdentifier(
      this.gameDeclaration.types,
      type.kind === 'Arrow' && type.rhs.kind === 'TypeReference'
        ? `${type.lhs}_${type.rhs.identifier}`
        : 'NormalizedType1',
    );

    // Create a declaration and reference it.
    this.gameDeclaration.types.push(ast.TypeDeclaration({ identifier, type }));
    return ast.TypeReference({ identifier });
  }

  resolveTypeDeclaration(type: ast.Type) {
    return this.gameDeclaration.types.find(
      typeDeclaration =>
        this.isAssignable(type, typeDeclaration.type) &&
        this.isAssignable(typeDeclaration.type, type),
    );
  }

  resolveTypeReference(type: ast.Type) {
    let iterations = 0;
    while (type.kind === 'TypeReference') {
      const typeDeclaration = utils.find(this.gameDeclaration.types, {
        identifier: type.identifier,
      });

      ++iterations;
      this.scopeEntry({ type });
      this.assert(typeDeclaration, 'Unresolved TypeReference.');
      type = typeDeclaration.type;
    }

    return this.scopeExit(type, iterations);
  }

  private scopeEntry(scope: Scope) {
    this.scopes.push(scope);
  }

  private scopeExtend(scope: Scope) {
    Object.assign(this.scopes[this.scopes.length - 1], scope);
  }

  private scopeExit<T>(value: T, scopes = 1) {
    while (scopes--) {
      this.scopes.pop();
    }

    return value;
  }
}
