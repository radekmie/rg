import { ast as rbg } from '../rbg';
import { ast as rg } from '../rg';
import * as utils from '../utils';

type Context = {
  $connect: (lhs: rg.EdgeName, rhs: rg.EdgeName, label: rg.EdgeLabel) => void;
  $createConstantFromMap: (
    pairs: [string, string][],
    defaultValue: string,
  ) => string;
  $createTypeFromSet: (identifiers: string[]) => string;
  $randomEdgeName: () => rg.EdgeName;
  rbg: rbg.Game;
  rg: rg.GameDeclaration;
  ruleAutomatons: Record<string, [rg.EdgeName, rg.EdgeName]>;
};

function translateAtomContent(
  context: Context,
  content: rbg.Action | rbg.Rule,
  from: rg.EdgeName,
  to: rg.EdgeName,
) {
  switch (content.kind) {
    case 'Assignment':
      if (
        context.rbg.players.some(player => player.name === content.variable)
      ) {
        context.$connect(
          from,
          to,
          rg.Assignment({
            lhs: rg.Access({
              lhs: rg.Reference({ identifier: 'goals' }),
              rhs: rg.Reference({ identifier: content.variable }),
            }),
            rhs: rg.Reference({
              identifier: translateValue(context, content.value),
            }),
          }),
        );
        return;
      }
      console.log(content);
      throw new Error('Not implemented (Assignment).');
    case 'Check': {
      const rule = rbg.serializeRule(content.rule);
      if (!(rule in context.ruleAutomatons)) {
        const localFrom = context.$randomEdgeName();
        const localTo = context.$randomEdgeName();
        context.ruleAutomatons[rule] = [localFrom, localTo];
        translateAtomContent(context, content.rule, localFrom, localTo);
      }

      const [localFrom, localTo] = context.ruleAutomatons[rule];
      context.$connect(
        from,
        to,
        rg.Reachability({
          lhs: localFrom,
          rhs: localTo,
          negated: content.negated,
        }),
      );
      return;
    }
    case 'Comparison':
      console.log(content);
      throw new Error('Not implemented (Comparison).');
    case 'Off':
      context.$connect(
        from,
        to,
        rg.Assignment({
          lhs: rg.Access({
            lhs: rg.Reference({ identifier: 'board' }),
            rhs: rg.Reference({ identifier: 'coord' }),
          }),
          rhs: rg.Reference({ identifier: content.piece }),
        }),
      );
      return;
    case 'On':
      if (content.pieces.length === 0) {
        context.$connect(
          from,
          rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
          rg.Skip({}),
        );
      }

      for (const piece of content.pieces) {
        // Add an empty edge to make sure we won't create multiedges.
        const local = context.$randomEdgeName();
        context.$connect(from, local, rg.Skip({}));
        context.$connect(
          local,
          to,
          rg.Comparison({
            lhs: rg.Access({
              lhs: rg.Reference({ identifier: 'board' }),
              rhs: rg.Reference({ identifier: 'coord' }),
            }),
            rhs: rg.Reference({ identifier: piece }),
            negated: false,
          }),
        );
      }
      return;
    case 'Rule':
      if (isExpandableShiftPattern(content)) {
        const pairs = context.rbg.board.map<[string, string[]]>(node => [
          node.node,
          makeShiftPattern(context.rbg, node.node, content),
        ]);

        if (pairs.every(pair => pair[1].length === pairs.length)) {
          const local = context.$randomEdgeName();
          local.parts.push(
            rg.Binding({
              identifier: 'coordGenerator',
              type: rg.TypeReference({ identifier: 'Coord' }),
            }),
          );

          context.$connect(
            from,
            local,
            rg.Comparison({
              lhs: rg.Cast({
                lhs: rg.TypeReference({ identifier: 'Coord' }),
                rhs: rg.Reference({ identifier: 'coordGenerator' }),
              }),
              rhs: rg.Cast({
                lhs: rg.TypeReference({ identifier: 'Coord' }),
                rhs: rg.Reference({ identifier: 'null' }),
              }),
              negated: true,
            }),
          );
          context.$connect(
            local,
            to,
            rg.Assignment({
              lhs: rg.Reference({ identifier: 'coord' }),
              rhs: rg.Reference({ identifier: 'coordGenerator' }),
            }),
          );
          return;
        }

        if (pairs.every(pair => pair[1].length <= 1)) {
          const map = pairs
            .map<[string, string]>(([k, [v = 'null']]) => [k, v])
            .concat([['null', 'null']]);

          const contant = context.$createConstantFromMap(map, 'null');
          const local = context.$randomEdgeName();
          context.$connect(
            from,
            local,
            rg.Assignment({
              lhs: rg.Reference({ identifier: 'coord' }),
              rhs: rg.Access({
                lhs: rg.Reference({ identifier: contant }),
                rhs: rg.Reference({ identifier: 'coord' }),
              }),
            }),
          );
          context.$connect(
            local,
            to,
            rg.Comparison({
              lhs: rg.Reference({ identifier: 'coord' }),
              rhs: rg.Reference({ identifier: 'null' }),
              negated: true,
            }),
          );
          return;
        }

        for (const [coord, reachableCoords] of pairs) {
          if (reachableCoords.length === 0) {
            continue;
          }

          const local = context.$randomEdgeName();
          if (reachableCoords.length === 1) {
            context.$connect(
              from,
              local,
              rg.Comparison({
                lhs: rg.Reference({ identifier: 'coord' }),
                rhs: rg.Reference({ identifier: coord }),
                negated: false,
              }),
            );
            context.$connect(
              local,
              to,
              rg.Assignment({
                lhs: rg.Reference({ identifier: 'coord' }),
                rhs: rg.Reference({ identifier: reachableCoords[0] }),
              }),
            );
            continue;
          }

          const usesAllCoords = reachableCoords.length === pairs.length;
          local.parts.push(
            rg.Binding({
              identifier: 'coordGenerator',
              type: rg.TypeReference({
                identifier: usesAllCoords
                  ? 'Coord'
                  : context.$createTypeFromSet(reachableCoords),
              }),
            }),
          );

          if (usesAllCoords) {
            context.$connect(
              from,
              local,
              rg.Comparison({
                lhs: rg.Cast({
                  lhs: rg.TypeReference({ identifier: 'Coord' }),
                  rhs: rg.Reference({ identifier: 'coordGenerator' }),
                }),
                rhs: rg.Cast({
                  lhs: rg.TypeReference({ identifier: 'Coord' }),
                  rhs: rg.Reference({ identifier: 'null' }),
                }),
                negated: true,
              }),
            );
            context.$connect(
              local,
              to,
              rg.Assignment({
                lhs: rg.Reference({ identifier: 'coord' }),
                rhs: rg.Reference({ identifier: 'coordGenerator' }),
              }),
            );
            continue;
          }

          context.$connect(
            from,
            local,
            rg.Comparison({
              lhs: rg.Reference({ identifier: 'coord' }),
              rhs: rg.Cast({
                lhs: rg.TypeReference({ identifier: 'Coord' }),
                rhs: rg.Reference({ identifier: coord }),
              }),
              negated: false,
            }),
          );
          context.$connect(
            local,
            to,
            rg.Assignment({
              lhs: rg.Reference({ identifier: 'coord' }),
              rhs: rg.Reference({ identifier: 'coordGenerator' }),
            }),
          );
        }
        return;
      }

      for (const concatenation of content.elements) {
        let localFrom = from;
        for (const atom of concatenation) {
          const localTo = context.$randomEdgeName();
          if (atom.power) {
            const localPre = context.$randomEdgeName();
            const localAfter = context.$randomEdgeName();
            translateAtomContent(context, atom.content, localPre, localAfter);
            context.$connect(localFrom, localPre, rg.Skip({}));
            context.$connect(localFrom, localTo, rg.Skip({}));
            context.$connect(localAfter, localPre, rg.Skip({}));
            context.$connect(localAfter, localTo, rg.Skip({}));
          } else {
            translateAtomContent(context, atom.content, localFrom, localTo);
          }
          localFrom = localTo;
        }
        context.$connect(localFrom, to, rg.Skip({}));
      }
      return;
    case 'Shift': {
      const testCoord = context.$randomEdgeName();
      const nextCoord = rg.Access({
        lhs: rg.Access({
          lhs: rg.Reference({ identifier: 'direction' }),
          rhs: rg.Reference({ identifier: 'coord' }),
        }),
        rhs: rg.Reference({ identifier: content.label }),
      });

      context.$connect(
        from,
        testCoord,
        rg.Comparison({
          lhs: nextCoord,
          rhs: rg.Reference({ identifier: 'null' }),
          negated: true,
        }),
      );
      context.$connect(
        testCoord,
        to,
        rg.Assignment({
          lhs: rg.Reference({ identifier: 'coord' }),
          rhs: nextCoord,
        }),
      );
      return;
    }
    case 'Switch':
      context.$connect(
        from,
        to,
        rg.Assignment({
          lhs: rg.Reference({ identifier: 'player' }),
          rhs: rg.Reference({ identifier: content.player ?? 'keeper' }),
        }),
      );
      return;
  }
}

function translateGame(context: Context) {
  translateAtomContent(
    context,
    context.rbg.rules,
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'begin' })] }),
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
  );

  context.rg.types.push(
    rg.TypeDeclaration({
      identifier: 'Player',
      type: rg.Set({
        identifiers: context.rbg.players.map(player => player.name),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'PlayerOrKeeper',
      type: rg.Set({
        identifiers: context.rbg.players
          .map(player => player.name)
          .concat('keeper'),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Score',
      type: rg.Set({
        identifiers: Array.from(
          {
            length:
              Math.max(...context.rbg.players.map(player => player.bound)) + 1,
          },
          (_, index) => `${index}`,
        ),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Goals',
      type: rg.Arrow({
        lhs: 'Player',
        rhs: rg.TypeReference({ identifier: 'Score' }),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Piece',
      type: rg.Set({ identifiers: context.rbg.pieces }),
    }),
    rg.TypeDeclaration({
      identifier: 'Label',
      type: rg.Set({
        identifiers: context.rbg.board
          .flatMap(node => node.edges)
          .map(edge => edge.label)
          .reduce<string[]>(utils.unique, []),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Coord',
      type: rg.Set({
        identifiers: context.rbg.board.map(node => node.node).concat('null'),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Board',
      type: rg.Arrow({
        lhs: 'Coord',
        rhs: rg.TypeReference({ identifier: 'Piece' }),
      }),
    }),
  );

  context.rg.constants.push(
    rg.ConstantDeclaration({
      identifier: 'direction',
      type: rg.Arrow({
        lhs: 'Coord',
        rhs: rg.Arrow({
          lhs: 'Label',
          rhs: rg.TypeReference({ identifier: 'Coord' }),
        }),
      }),
      value: rg.Map({
        entries: [
          rg.DefaultEntry({
            value: rg.Map({
              entries: [
                rg.DefaultEntry({
                  value: rg.Element({ identifier: 'null' }),
                }),
              ],
            }),
          }),
          ...context.rbg.board.map(node =>
            rg.NamedEntry({
              identifier: node.node,
              value: rg.Map({
                entries: [
                  rg.DefaultEntry({
                    value: rg.Element({ identifier: 'null' }),
                  }),
                  ...node.edges.map(edge =>
                    rg.NamedEntry({
                      identifier: edge.label,
                      value: rg.Element({ identifier: edge.node }),
                    }),
                  ),
                ],
              }),
            }),
          ),
        ],
      }),
    }),
  );

  context.rg.variables.push(
    rg.VariableDeclaration({
      identifier: 'player',
      type: rg.TypeReference({ identifier: 'PlayerOrKeeper' }),
      defaultValue: rg.Element({ identifier: 'keeper' }),
    }),
    rg.VariableDeclaration({
      identifier: 'goals',
      type: rg.TypeReference({ identifier: 'Goals' }),
      defaultValue: rg.Map({
        entries: [rg.DefaultEntry({ value: rg.Element({ identifier: '0' }) })],
      }),
    }),
    rg.VariableDeclaration({
      identifier: 'board',
      type: rg.TypeReference({ identifier: 'Board' }),
      defaultValue: rg.Map({
        entries: [
          rg.DefaultEntry({
            value: rg.Element({ identifier: context.rbg.pieces[0] }),
          }),
          ...context.rbg.board.map(node =>
            rg.NamedEntry({
              identifier: node.node,
              value: rg.Element({ identifier: node.piece }),
            }),
          ),
        ],
      }),
    }),
    rg.VariableDeclaration({
      identifier: 'coord',
      type: rg.TypeReference({ identifier: 'Coord' }),
      defaultValue: rg.Element({ identifier: context.rbg.board[0].node }),
    }),
  );

  return context.rg;
}

function translateValue(context: Context, value: rbg.Value) {
  switch (typeof value) {
    case 'number':
      return `${value}`;
    case 'string':
      return value;
  }

  throw new Error('Not implemented (Value).');
}

function isExpandableShiftPattern(content: rbg.Action | rbg.Rule) {
  return (
    content.kind === 'Rule' &&
    content.elements.some(concatenation => concatenation.length > 1) &&
    isShiftPattern(content)
  );
}

function isShiftPattern(content: rbg.Action | rbg.Rule): boolean {
  return (
    content.kind === 'Shift' ||
    (content.kind === 'Rule' &&
      content.elements.every(concatenation =>
        concatenation.every(atom => isShiftPattern(atom.content)),
      ))
  );
}

function groupShiftPatterns(rule: rbg.Rule) {
  if (isShiftPattern(rule)) {
    return rule;
  }

  return rbg.Rule({
    elements: rule.elements.map(concatenation =>
      concatenation.reduce<rbg.Atom[]>((concatenation, atom) => {
        switch (atom.content.kind) {
          case 'Check':
            atom = rbg.Atom({
              content: rbg.Check({
                negated: atom.content.negated,
                rule: groupShiftPatterns(atom.content.rule),
              }),
              power: atom.power,
            });
            break;
          case 'Rule':
            atom = rbg.Atom({
              content: groupShiftPatterns(atom.content),
              power: atom.power,
            });
            break;
        }

        if (isShiftPattern(atom.content)) {
          const previous = concatenation[concatenation.length - 1];
          if (
            previous &&
            !previous.power &&
            previous.content.kind === 'Rule' &&
            previous.content.elements.length === 1 &&
            isShiftPattern(previous.content)
          ) {
            previous.content.elements[0].push(atom);
          } else {
            concatenation.push(
              rbg.Atom({
                content: rbg.Rule({ elements: [[atom]] }),
                power: false,
              }),
            );
          }
        } else {
          concatenation.push(atom);
        }

        return concatenation;
      }, []),
    ),
  });
}

function makeShiftPattern(game: rbg.Game, coord: string, rule: rbg.Rule) {
  return rule.elements
    .flatMap(concatenation =>
      concatenation.reduce(
        (coords, { content, power }) => {
          const reachableCoords = power ? coords.slice() : [];
          for (let coord of coords) {
            switch (content.kind) {
              case 'Shift': {
                const { label } = content;
                do {
                  const node = game.board.find(node => node.node === coord);
                  const edge = node?.edges.find(edge => edge.label === label);
                  if (edge) {
                    utils.unique(reachableCoords, edge.node);
                    coord = edge.node;
                  } else {
                    break;
                  }
                } while (power);
                break;
              }
              case 'Rule': {
                for (const node of makeShiftPattern(game, coord, content)) {
                  utils.unique(reachableCoords, node);
                }
                break;
              }
              default:
                throw new Error(
                  `Incorrect shift pattern: ${content.toString()}`,
                );
            }
          }

          return reachableCoords;
        },
        [coord],
      ),
    )
    .reduce<string[]>(utils.unique, [])
    .sort();
}

export default function translate(game: rbg.Game) {
  let counter = 0;
  return translateGame({
    $connect(lhs, rhs, label) {
      this.rg.edges.push(rg.EdgeDeclaration({ lhs, rhs, label }));
    },
    $createConstantFromMap(pairs, defaultValue) {
      const type = rg.Arrow({
        lhs: this.$createTypeFromSet(
          pairs
            .map(pair => pair[0])
            .reduce<string[]>(utils.unique, [])
            .sort(),
        ),
        rhs: rg.TypeReference({
          identifier: this.$createTypeFromSet(
            pairs
              .map(pair => pair[1])
              .reduce<string[]>(utils.unique, [])
              .sort(),
          ),
        }),
      });

      const value = rg.Map({
        entries: [
          rg.DefaultEntry({ value: rg.Element({ identifier: defaultValue }) }),
          ...pairs
            .filter(([, to]) => to !== defaultValue)
            .sort()
            .map(([from, to]) =>
              rg.NamedEntry({
                identifier: from,
                value: rg.Element({ identifier: to }),
              }),
            ),
        ],
      });

      const existing = utils.find(this.rg.constants, { type, value });
      if (existing) {
        return existing.identifier;
      }

      const identifier = `CoordMap${counter++}`;
      this.rg.constants.push(
        rg.ConstantDeclaration({ identifier, type, value }),
      );

      return identifier;
    },
    $createTypeFromSet(identifiers) {
      const type = rg.Set({ identifiers });
      const existing = utils.find(this.rg.types, { type });
      if (existing) {
        return existing.identifier;
      }

      const identifier = `CoordGen${counter++}`;
      this.rg.types.push(rg.TypeDeclaration({ identifier, type }));

      return identifier;
    },
    $randomEdgeName() {
      return rg.EdgeName({
        parts: [rg.Literal({ identifier: `${counter++}` })],
      });
    },
    rbg: rbg.Game({
      board: game.board,
      pieces: game.pieces,
      variables: game.variables,
      players: game.players,
      rules: groupShiftPatterns(game.rules),
    }),
    rg: rg.GameDeclaration({
      constants: [],
      edges: [],
      types: [],
      variables: [],
    }),
    ruleAutomatons: Object.create(null),
  });
}
