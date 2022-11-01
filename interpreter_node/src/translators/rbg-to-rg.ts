import { ast as rbg } from '../rbg';
import { ast as rg } from '../rg';
import * as utils from '../utils';

type Context = {
  $connect: (lhs: rg.EdgeName, rhs: rg.EdgeName, label: rg.EdgeLabel) => void;
  $randomEdgeName: () => rg.EdgeName;
  rbg: rbg.Game;
  rg: rg.GameDeclaration;
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
      const localFrom = context.$randomEdgeName();
      const localTo = context.$randomEdgeName();
      context.$connect(
        from,
        to,
        rg.Reachability({
          lhs: localFrom,
          rhs: localTo,
          negated: content.negated,
        }),
      );
      translateAtomContent(context, content.rule, localFrom, localTo);
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
        context.$connect(
          from,
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
      type: rg.Set({ identifiers: context.rbg.board.map(node => node.node) }),
    }),
    rg.TypeDeclaration({
      identifier: 'CoordOrNull',
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
          rhs: rg.TypeReference({ identifier: 'CoordOrNull' }),
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

export default function translate(rbg: rbg.Game) {
  let counter = 0;
  return translateGame({
    $connect(lhs, rhs, label) {
      this.rg.edges.push(rg.EdgeDeclaration({ lhs, rhs, label }));
    },
    $randomEdgeName() {
      return rg.EdgeName({
        parts: [rg.Literal({ identifier: `${counter++}` })],
      });
    },
    rbg,
    rg: rg.GameDeclaration({
      constants: [],
      edges: [],
      types: [],
      variables: [],
    }),
  });
}
