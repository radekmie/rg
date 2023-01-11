import * as ast from '../ast';

const builtins: ((
  gameDeclaration: ast.GameDeclaration,
  typeChecker: ast.TypeChecker,
) => void)[] = [
  // |- Bool
  (gameDeclaration, typeChecker) => {
    if (!typeChecker.resolveType('Bool')) {
      gameDeclaration.types.push(
        ast.TypeDeclaration({
          identifier: 'Bool',
          type: ast.Set({ identifiers: ['0', '1'] }),
        }),
      );
    }
  },
  // Player ^ Score |- Goals
  (gameDeclaration, typeChecker) => {
    if (
      !typeChecker.resolveType('Goals') &&
      typeChecker.resolveType('Player') &&
      typeChecker.resolveType('Score')
    ) {
      gameDeclaration.types.push(
        ast.TypeDeclaration({
          identifier: 'Goals',
          type: ast.Arrow({
            lhs: 'Player',
            rhs: ast.TypeReference({ identifier: 'Score' }),
          }),
        }),
      );
    }
  },
  // Player |- Visibility
  (gameDeclaration, typeChecker) => {
    if (
      !typeChecker.resolveType('Visibility') &&
      typeChecker.resolveType('Player')
    ) {
      gameDeclaration.types.push(
        ast.TypeDeclaration({
          identifier: 'Visibility',
          type: ast.Arrow({
            lhs: 'Player',
            rhs: ast.TypeReference({ identifier: 'Bool' }),
          }),
        }),
      );
    }
  },
  // Player ^ isSet(Player) |- PlayerOrKeeper
  (gameDeclaration, typeChecker) => {
    const playerOrKeeper = typeChecker.resolveType('PlayerOrKeeper');
    if (!playerOrKeeper) {
      const player = typeChecker.resolveType('Player');
      if (player?.type.kind === 'Set') {
        gameDeclaration.types.push(
          ast.TypeDeclaration({
            identifier: 'PlayerOrKeeper',
            type: ast.Set({
              identifiers: player.type.identifiers.includes('keeper')
                ? player.type.identifiers
                : player.type.identifiers.concat('keeper'),
            }),
          }),
        );
      }
    }
  },
  // Goals ^ Score ^ isSet(Score) |- goals
  (gameDeclaration, typeChecker) => {
    if (typeChecker.resolveType('Goals')) {
      const goals = typeChecker.resolveVariable('goals');
      if (!goals) {
        const score = typeChecker.resolveType('Score');
        if (score?.type.kind === 'Set') {
          gameDeclaration.variables.push(
            ast.VariableDeclaration({
              identifier: 'goals',
              type: ast.TypeReference({ identifier: 'Goals' }),
              defaultValue: ast.Map({
                entries: [
                  ast.DefaultEntry({
                    value: ast.Element({
                      identifier: score.type.identifiers[0],
                    }),
                  }),
                ],
              }),
            }),
          );
        }
      }
    }
  },
  // PlayerOrKeeper |- player
  (gameDeclaration, typeChecker) => {
    if (
      !typeChecker.resolveVariable('player') &&
      typeChecker.resolveType('PlayerOrKeeper')
    ) {
      gameDeclaration.variables.push(
        ast.VariableDeclaration({
          identifier: 'player',
          type: ast.TypeReference({ identifier: 'PlayerOrKeeper' }),
          defaultValue: ast.Element({ identifier: 'keeper' }),
        }),
      );
    }
  },
  // Visibility |- visibility
  (gameDeclaration, typeChecker) => {
    if (
      !typeChecker.resolveVariable('visible') &&
      typeChecker.resolveType('Visibility')
    ) {
      gameDeclaration.variables.push(
        ast.VariableDeclaration({
          identifier: 'visible',
          type: ast.TypeReference({ identifier: 'Visibility' }),
          defaultValue: ast.Map({
            entries: [
              ast.DefaultEntry({ value: ast.Element({ identifier: '1' }) }),
            ],
          }),
        }),
      );
    }
  },
];

export function addBuiltins(gameDeclaration: ast.GameDeclaration) {
  const typeChecker = new ast.TypeChecker(gameDeclaration);
  for (const builtin of builtins) {
    builtin(gameDeclaration, typeChecker);
  }
}
