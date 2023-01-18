import * as ast from '../ast';

const builtinTypes: ((
  gameDeclaration: ast.GameDeclaration,
  typeChecker: ast.TypeChecker,
) => ast.TypeDeclaration)[] = [
  // |- Bool
  () =>
    ast.TypeDeclaration({
      identifier: 'Bool',
      type: ast.Set({ identifiers: ['0', '1'] }),
    }),
  // Player ^ Score |- Goals
  (gameDeclaration, typeChecker: ast.TypeChecker) => {
    const Player = typeChecker.resolveType('Player');
    typeChecker.assert(Player, 'Type `Player` is missing.');

    const Score = typeChecker.resolveType('Score');
    typeChecker.assert(Score, 'Type `Score` is missing.');

    return ast.TypeDeclaration({
      identifier: 'Goals',
      type: ast.Arrow({
        lhs: 'Player',
        rhs: ast.TypeReference({ identifier: 'Score' }),
      }),
    });
  },
  // Player |- Visibility
  (gameDeclaration, typeChecker: ast.TypeChecker) => {
    const Player = typeChecker.resolveType('Player');
    typeChecker.assert(Player, 'Type `Player` is missing.');

    return ast.TypeDeclaration({
      identifier: 'Visibility',
      type: ast.Arrow({
        lhs: 'Player',
        rhs: ast.TypeReference({ identifier: 'Bool' }),
      }),
    });
  },
  // Player ^ isSet(Player) |- PlayerOrKeeper
  (gameDeclaration, typeChecker: ast.TypeChecker) => {
    const Player = typeChecker.resolveType('Player');
    typeChecker.assert(Player, 'Type `Player` is missing.');
    typeChecker.assert(Player.type.kind === 'Set', 'Type `Player` is invalid.');

    return ast.TypeDeclaration({
      identifier: 'PlayerOrKeeper',
      type: ast.Set({
        identifiers: Player.type.identifiers.includes('keeper')
          ? Player.type.identifiers
          : Player.type.identifiers.concat('keeper'),
      }),
    });
  },
];

const builtinVariables: ((
  gameDeclaration: ast.GameDeclaration,
  typeChecker: ast.TypeChecker,
) => ast.VariableDeclaration)[] = [
  // Goals ^ Score ^ isSet(Score) |- goals
  (gameDeclaration, typeChecker: ast.TypeChecker) => {
    const Goals = typeChecker.resolveType('Goals');
    typeChecker.assert(Goals, 'Type `Goals` is missing.');

    const Score = typeChecker.resolveType('Score');
    typeChecker.assert(Score, 'Type `Score` is missing.');
    typeChecker.assert(Score.type.kind === 'Set', 'Type `Score` is invalid.');

    return ast.VariableDeclaration({
      identifier: 'goals',
      type: ast.TypeReference({ identifier: 'Goals' }),
      defaultValue: ast.Map({
        entries: [
          ast.DefaultEntry({
            value: ast.Element({
              identifier: Score.type.identifiers[0],
            }),
          }),
        ],
      }),
    });
  },
  // PlayerOrKeeper |- player
  (gameDeclaration, typeChecker: ast.TypeChecker) => {
    const PlayerOrKeeper = typeChecker.resolveType('PlayerOrKeeper');
    typeChecker.assert(PlayerOrKeeper, 'Type `PlayerOrKeeper` is missing.');

    return ast.VariableDeclaration({
      identifier: 'player',
      type: ast.TypeReference({ identifier: 'PlayerOrKeeper' }),
      defaultValue: ast.Element({ identifier: 'keeper' }),
    });
  },
  // Visibility |- visibility
  (gameDeclaration, typeChecker: ast.TypeChecker) => {
    const Visibility = typeChecker.resolveType('Visibility');
    typeChecker.assert(Visibility, 'Type `Visibility` is missing.');

    return ast.VariableDeclaration({
      identifier: 'visible',
      type: ast.TypeReference({ identifier: 'Visibility' }),
      defaultValue: ast.Map({
        entries: [
          ast.DefaultEntry({ value: ast.Element({ identifier: '1' }) }),
        ],
      }),
    });
  },
];

export function addBuiltins(gameDeclaration: ast.GameDeclaration) {
  const typeChecker: ast.TypeChecker = new ast.TypeChecker(gameDeclaration);

  for (const builtinType of builtinTypes) {
    const builtin = builtinType(gameDeclaration, typeChecker);
    const defined = typeChecker.resolveType(builtin.identifier);

    if (defined) {
      typeChecker.assert(
        typeChecker.isAssignable(builtin.type, defined.type) &&
          typeChecker.isAssignable(defined.type, builtin.type),
        `Incorrect "${builtin.identifier}" type definition.`,
      );
    } else {
      gameDeclaration.types.push(builtin);
    }
  }

  for (const builtinVariable of builtinVariables) {
    const builtin = builtinVariable(gameDeclaration, typeChecker);
    const defined = typeChecker.resolveVariable(builtin.identifier);

    if (defined) {
      typeChecker.assert(
        typeChecker.isAssignable(builtin.type, defined.type) &&
          typeChecker.isAssignable(defined.type, builtin.type),
        `Incorrect "${builtin.identifier}" type definition.`,
      );
    } else {
      gameDeclaration.variables.push(builtin);
    }
  }
}
