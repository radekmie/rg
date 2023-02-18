import * as ist from './types';
import * as utils from '../../utils';
import * as ast from '../ast';
import * as transformators from '../transformators';

export function build(gameDeclaration: ast.GameDeclaration) {
  gameDeclaration = ast.GameDeclaration({
    constants: gameDeclaration.constants,
    edges: utils.clone(gameDeclaration.edges),
    pragmas: gameDeclaration.pragmas,
    types: gameDeclaration.types,
    variables: gameDeclaration.variables,
  });
  transformators.expandGeneratorNodes(gameDeclaration);

  const game = ist.Game({
    constants: Object.create(null),
    edges: Object.create(null),
    pragmas: [],
    types: Object.create(null),
    variables: Object.create(null),
  });

  buildPragmas(game, gameDeclaration.pragmas);
  buildTypes(game, gameDeclaration.types);
  buildConstants(game, gameDeclaration.constants);
  buildVariables(game, gameDeclaration.variables);
  buildEdges(game, gameDeclaration.edges);

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

function buildEdgeName({ parts }: ast.EdgeName) {
  utils.assert(parts.length === 1, 'Expected simple EdgeName.');
  utils.assert(parts[0].kind === 'Literal', 'Expected simple EdgeName.');
  return parts[0].identifier;
}

function buildEdgeLabel(game: ist.Game, edgeLabel: ast.EdgeLabel) {
  switch (edgeLabel.kind) {
    case 'Assignment':
      return ist.Assignment({
        lhs: buildExpression(game, edgeLabel.lhs),
        rhs: buildExpression(game, edgeLabel.rhs),
      });
    case 'Comparison':
      return ist.Comparison({
        lhs: buildExpression(game, edgeLabel.lhs),
        rhs: buildExpression(game, edgeLabel.rhs),
        negated: edgeLabel.negated,
      });
    case 'Reachability':
      return ist.Reachability({
        lhs: buildEdgeName(edgeLabel.lhs),
        rhs: buildEdgeName(edgeLabel.rhs),
        negated: edgeLabel.negated,
      });
    case 'Skip':
      return ist.Skip({});
  }
}

function buildEdges(game: ist.Game, edgeDeclarations: ast.EdgeDeclaration[]) {
  for (const edgeDeclaration of edgeDeclarations) {
    const lhs = buildEdgeName(edgeDeclaration.lhs);
    const rhs = buildEdgeName(edgeDeclaration.rhs);
    const label = buildEdgeLabel(game, edgeDeclaration.label);

    game.edges[lhs] ??= [];
    game.edges[lhs].push(ist.Edge({ label, next: rhs }));
  }
}

function buildExpression(
  game: ist.Game,
  expression: ast.Expression,
): ist.Expression {
  switch (expression.kind) {
    case 'Access':
      return ist.Access({
        lhs: buildExpression(game, expression.lhs),
        rhs: buildExpression(game, expression.rhs),
      });
    case 'Cast':
      return buildExpression(game, expression.rhs);
    case 'Reference':
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

function buildPragma(pragma: ast.Pragma) {
  switch (pragma.kind) {
    case 'Distinct':
      return ist.Distinct({ edgeName: buildEdgeName(pragma.edgeName) });
  }
}

function buildPragmas(game: ist.Game, pragmas: ast.Pragma[]) {
  for (const pragma of pragmas) {
    game.pragmas.push(buildPragma(pragma));
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
