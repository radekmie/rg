import * as utils from '../../utils';
import * as ast from '../ast';
import * as ist from './types';

export function build(gameDeclaration: ast.GameDeclaration) {
  const game = ist.Game({
    constants: Object.create(null),
    edges: [],
    types: Object.create(null),
    variables: Object.create(null),
  });

  buildTypes(game, gameDeclaration.types);
  buildConstants(game, gameDeclaration.constants);
  buildVariables(game, gameDeclaration.variables);
  buildEdges(game, gameDeclaration.edges);

  // TODO: Check if begin and end exists and are have no binds.

  return game;
}

function buildConstants(
  game: ist.Game,
  constantDeclarations: ast.ConstantDeclaration[],
) {
  for (const constantDeclaration of constantDeclarations) {
    game.constants[constantDeclaration.identifier] = buildValue(
      game,
      buildTypeOrFail(game, constantDeclaration.type),
      constantDeclaration.value,
    );
  }
}

function buildEdgeName(game: ist.Game, edgeName: ast.EdgeName) {
  return ist.EdgeName({
    label: edgeName.parts
      .map(edgeNamePart => {
        switch (edgeNamePart.kind) {
          case 'Binding':
            return `(${edgeNamePart.identifier})`;
          case 'Literal':
            return edgeNamePart.identifier;
        }
      })
      .join(''),
    types: edgeName.parts
      .flatMap(edgeNamePart => {
        switch (edgeNamePart.kind) {
          case 'Binding':
            return [edgeNamePart];
          case 'Literal':
            return [];
        }
      })
      .reduce<Record<string, ist.Type>>((types, binding) => {
        types[binding.identifier] = buildTypeOrFail(game, binding.type);
        return types;
      }, Object.create(null)),
    values: Object.create(null),
  });
}

function buildEdgeLabel(
  game: ist.Game,
  edgeLabel: ast.EdgeLabel,
  binds: Set<string>,
) {
  switch (edgeLabel.kind) {
    case 'Assignment':
      return ist.Assignment({
        lhs: buildExpression(game, edgeLabel.lhs, binds),
        rhs: buildExpression(game, edgeLabel.rhs, binds),
      });
    case 'Comparison':
      return ist.Comparison({
        lhs: buildExpression(game, edgeLabel.lhs, binds),
        rhs: buildExpression(game, edgeLabel.rhs, binds),
        negated: edgeLabel.negated,
      });
    case 'Reachability':
      return ist.Reachability({
        lhs: buildEdgeName(game, edgeLabel.lhs),
        rhs: buildEdgeName(game, edgeLabel.rhs),
        negated: edgeLabel.negated,
      });
    case 'Skip':
      return ist.Skip({});
  }
}

function buildEdges(game: ist.Game, edgeDeclarations: ast.EdgeDeclaration[]) {
  for (const edgeDeclaration of edgeDeclarations) {
    // TODO: Type check binds.
    const lhs = buildEdgeName(game, edgeDeclaration.lhs);
    const rhs = buildEdgeName(game, edgeDeclaration.rhs);
    const binds = new Set([
      ...Object.keys(lhs.types),
      ...Object.keys(rhs.types),
    ]);

    game.edges.push(
      ist.Edge({
        lhs,
        rhs,
        label: buildEdgeLabel(game, edgeDeclaration.label, binds),
      }),
    );
  }
}

function buildExpression(
  game: ist.Game,
  expression: ast.Expression,
  binds: Set<string>,
): ist.Expression {
  switch (expression.kind) {
    case 'Access':
      return ist.Access({
        lhs: buildExpression(game, expression.lhs, binds),
        rhs: buildExpression(game, expression.rhs, binds),
      });
    case 'Cast':
      return ist.Cast({
        lhs: buildTypeOrFail(game, expression.lhs),
        rhs: buildExpression(game, expression.rhs, binds),
      });
    case 'Reference':
      if (binds.has(expression.identifier)) {
        return ist.BindReference({ identifier: expression.identifier });
      }
      if (expression.identifier in game.constants) {
        return ist.ConstantReference({ identifier: expression.identifier });
      }
      if (expression.identifier in game.variables) {
        return ist.VariableReference({ identifier: expression.identifier });
      }
      return ist.Literal({
        value: ist.Element({ value: expression.identifier }),
      });
  }
}

function buildType(game: ist.Game, type: ast.Type): ist.Type | null {
  switch (type.kind) {
    case 'Arrow': {
      if (!(type.lhs in game.types)) {
        return null;
      }
      const lhs = game.types[type.lhs];
      const rhs = buildType(game, type.rhs);
      if (rhs === null) {
        return null;
      }
      return ist.Arrow({ lhs, rhs });
    }
    case 'Set':
      return ist.Set({
        values: type.identifiers.map(value => ist.Element({ value })),
      });
    case 'TypeReference':
      if (type.identifier in game.types) {
        return game.types[type.identifier];
      }
      return null;
  }
}

function buildTypeOrFail(game: ist.Game, type: ast.Type): ist.Type {
  const builtType = buildType(game, type);
  utils.assert(builtType !== null, `Unresolved type ${JSON.stringify(type)}.`);
  return builtType;
}

function buildTypes(game: ist.Game, typeDeclarations: ast.TypeDeclaration[]) {
  const unresolvedTypeDeclarations: ast.TypeDeclaration[] = [];
  for (const typeDeclaration of typeDeclarations) {
    const resolved = buildType(game, typeDeclaration.type);
    if (resolved === null) {
      unresolvedTypeDeclarations.push(typeDeclaration);
    } else {
      game.types[typeDeclaration.identifier] = resolved;
    }
  }

  if (unresolvedTypeDeclarations.length > 0) {
    utils.assert(
      unresolvedTypeDeclarations.length < typeDeclarations.length,
      `Unresolved type: ${unresolvedTypeDeclarations[0].identifier}.`,
    );

    buildTypes(game, unresolvedTypeDeclarations);
  }
}

function buildValue(
  game: ist.Game,
  type: ist.Type,
  value: ast.Value,
): ist.Value {
  switch (value.kind) {
    case 'Element':
      if (value.identifier in game.constants) {
        return game.constants[value.identifier];
      }
      return ist.Element({ value: value.identifier });
    case 'Map': {
      utils.assert(type.kind === 'Arrow', 'Incorrect Map type found.');

      const defaultEntries = value.entries.flatMap(entry => {
        switch (entry.kind) {
          case 'DefaultEntry':
            return [entry];
          case 'NamedEntry':
            return [];
        }
      });

      utils.assert(
        defaultEntries.length === 1,
        'Exactly one default entry required.',
      );

      return ist.Map({
        defaultValue: buildValue(game, type.rhs, defaultEntries[0].value),
        values: value.entries
          .flatMap(entry => {
            switch (entry.kind) {
              case 'DefaultEntry':
                return [];
              case 'NamedEntry':
                return [entry];
            }
          })
          .reduce<Record<string, ist.Value>>((values, namedEntry) => {
            utils.assert(
              !(namedEntry.identifier in values),
              'Duplicated named entry.',
            );

            values[namedEntry.identifier] = buildValue(
              game,
              type.rhs,
              namedEntry.value,
            );

            return values;
          }, Object.create(null)),
      });
    }
  }

  throw value;
}

function buildVariables(
  game: ist.Game,
  variableDeclarations: ast.VariableDeclaration[],
) {
  for (const variableDeclaration of variableDeclarations) {
    utils.assert(
      !(variableDeclaration.identifier in game.constants),
      `Variable duplicates constant ${variableDeclaration.identifier}.`,
    );
    utils.assert(
      !(variableDeclaration.identifier in game.variables),
      `Duplicated variable ${variableDeclaration.identifier}.`,
    );

    game.variables[variableDeclaration.identifier] = ist.Variable({
      type: buildTypeOrFail(game, variableDeclaration.type),
      defaultValue: buildValue(
        game,
        buildTypeOrFail(game, variableDeclaration.type),
        variableDeclaration.defaultValue,
      ),
    });
  }
}
