import * as astTypes from '../ast/types';
import * as utils from '../utils';
import * as istTypes from './types';

export function build(gameDeclaration: astTypes.GameDeclaration) {
  const game = istTypes.Game({
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
  game: istTypes.Game,
  constantDeclarations: astTypes.ConstantDeclaration[],
) {
  for (const constantDeclaration of constantDeclarations) {
    game.constants[constantDeclaration.identifier] = buildValue(
      game,
      buildTypeOrFail(game, constantDeclaration.type),
      constantDeclaration.value,
    );
  }
}

function buildEdgeName(game: istTypes.Game, edgeName: astTypes.EdgeName) {
  return istTypes.EdgeName({
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
      .reduce((types, binding) => {
        types[binding.identifier] = buildTypeOrFail(game, binding.type);
        return types;
      }, Object.create(null)),
    values: edgeName.parts
      .flatMap(edgeNamePart => {
        switch (edgeNamePart.kind) {
          case 'Binding':
            return [edgeNamePart];
          case 'Literal':
            return [];
        }
      })
      .reduce((values, binding) => {
        values[binding.identifier] = null;
        return values;
      }, Object.create(null)),
  });
}

function buildEdgeLabel(game: istTypes.Game, edgeLabel: astTypes.EdgeLabel) {
  switch (edgeLabel.kind) {
    case 'Assignment':
      return istTypes.Assignment({
        lhs: buildExpression(game, edgeLabel.lhs),
        rhs: buildExpression(game, edgeLabel.rhs),
      });
    case 'Comparison':
      return istTypes.Comparison({
        lhs: buildExpression(game, edgeLabel.lhs),
        rhs: buildExpression(game, edgeLabel.rhs),
        negated: edgeLabel.negated,
      });
    case 'Reachability':
      return istTypes.Reachability({
        lhs: buildEdgeName(game, edgeLabel.lhs),
        rhs: buildEdgeName(game, edgeLabel.rhs),
        mode: edgeLabel.mode,
      });
    case 'Skip':
      return istTypes.Skip({});
  }
}

function buildEdges(
  game: istTypes.Game,
  edgeDeclarations: astTypes.EdgeDeclaration[],
) {
  for (const edgeDeclaration of edgeDeclarations) {
    game.edges.push(
      istTypes.Edge({
        // TODO: Type check.
        lhs: buildEdgeName(game, edgeDeclaration.lhs),
        rhs: buildEdgeName(game, edgeDeclaration.rhs),
        label: buildEdgeLabel(game, edgeDeclaration.label),
      }),
    );
  }
}

function buildExpression(
  game: istTypes.Game,
  expression: astTypes.Expression,
): istTypes.Expression {
  switch (expression.kind) {
    case 'Access':
      return istTypes.Access({
        lhs: buildExpression(game, expression.lhs),
        rhs: buildExpression(game, expression.rhs),
      });
    case 'Cast':
      return istTypes.Cast({
        lhs: buildTypeOrFail(game, expression.lhs),
        rhs: buildExpression(game, expression.rhs),
      });
    case 'Reference':
      return istTypes.Reference({ identifier: expression.identifier });
  }
}

function buildType(
  game: istTypes.Game,
  type: astTypes.Type,
): istTypes.Type | null {
  switch (type.kind) {
    case 'Arrow': {
      if (!(type.lhs in game.types)) return null;
      const lhs = game.types[type.lhs];
      const rhs = buildType(game, type.rhs);
      if (rhs === null) return null;
      return istTypes.Arrow({ lhs, rhs });
    }
    case 'Set':
      return istTypes.Set({ identifiers: type.identifiers });
    case 'TypeReference':
      if (type.identifier in game.types) return game.types[type.identifier];
      return null;
  }
}

function buildTypeOrFail(
  game: istTypes.Game,
  type: astTypes.Type,
): istTypes.Type {
  const builtType = buildType(game, type);
  utils.assert(builtType !== null, 'Unresolved type.');
  return builtType;
}

function buildTypes(
  game: istTypes.Game,
  typeDeclarations: astTypes.TypeDeclaration[],
) {
  const unresolvedTypeDeclarations: astTypes.TypeDeclaration[] = [];
  for (const typeDeclaration of typeDeclarations) {
    const resolved = buildType(game, typeDeclaration.type);
    if (resolved === null) unresolvedTypeDeclarations.push(typeDeclaration);
    else game.types[typeDeclaration.identifier] = resolved;
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
  game: istTypes.Game,
  type: istTypes.Type,
  value: astTypes.Value,
): istTypes.Value {
  switch (value.kind) {
    case 'Element':
      if (value.identifier in game.constants)
        return game.constants[value.identifier];
      return istTypes.Element({ value: value.identifier });
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

      return istTypes.Map({
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
          .reduce((values, namedEntry) => {
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
  game: istTypes.Game,
  variableDeclarations: astTypes.VariableDeclaration[],
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

    game.variables[variableDeclaration.identifier] = istTypes.Variable({
      type: buildTypeOrFail(game, variableDeclaration.type),
      defaultValue: buildValue(
        game,
        buildTypeOrFail(game, variableDeclaration.type),
        variableDeclaration.defaultValue,
      ),
    });
  }
}
