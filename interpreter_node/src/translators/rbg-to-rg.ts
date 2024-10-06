import { ast as rbg } from '../rbg';
import { ast as rg } from '../rg';
import * as utils from '../utils';

type Context = {
  $connect: (lhs: rg.EdgeName, rhs: rg.EdgeName, label: rg.EdgeLabel) => void;
  $createConstantFromMap: (
    pairs: [string, string][],
    defaultValue: string,
  ) => string;
  $createTypeFromSet: (identifiers: string[]) => rg.Type;
  $exposeIndex: number;
  $mathOperator: (
    limit: number,
    lhs: rg.Expression,
    rhs: rg.Expression,
    operator:
      | rbg.Expression['operator']
      | Exclude<rbg.Comparison['operator'], '==' | '!='>,
  ) => rg.Expression;
  $mathType: (limit: number) => rg.Type;
  $randomEdgeName: () => rg.EdgeName;
  $shiftPatternsCache: Record<string, string[]>;
  rbg: rbg.Game;
  rg: rg.GameDeclaration;
  ruleAutomatons: Record<string, [rg.EdgeName, rg.EdgeName]>;
};

// eslint-disable-next-line complexity -- This function could be improved.
function translateAtomContent(
  context: Context,
  content: rbg.Action | rbg.Rule,
  from: rg.EdgeName,
  to: rg.EdgeName,
) {
  switch (content.kind) {
    case 'Assignment': {
      const { limit, lhs, rhs } = translateRValuePair(
        context,
        content.variable,
        content.rvalue,
      );

      // Check for overflow.
      if (hasMathExpression(rhs)) {
        const local = context.$randomEdgeName();
        context.$connect(
          from,
          local,
          rg.Comparison({
            lhs: rhs,
            rhs: rg.Reference({ identifier: 'nan' }),
            negated: true,
          }),
        );

        from = local;
      }

      const local = context.$randomEdgeName();
      context.$connect(from, local, rg.Assignment({ lhs, rhs }));

      // Expose position.
      exposePosition(context, local, to);
      return;
    }
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
    case 'Comparison': {
      const { limit, lhs, rhs } = translateRValuePair(
        context,
        content.lhs,
        content.rhs,
      );

      context.$connect(
        from,
        to,
        content.operator === '==' || content.operator === '!='
          ? rg.Comparison({ lhs, rhs, negated: content.operator === '!=' })
          : rg.Comparison({
              lhs: context.$mathOperator(limit + 1, lhs, rhs, content.operator),
              rhs: rg.Reference({ identifier: '1' }),
              negated: false,
            }),
      );
      return;
    }
    case 'Off': {
      const oneElement = rg.Reference({ identifier: '1' });

      // Decrease.
      const counterDec = context.$randomEdgeName();
      const counterDecValue = rg.Access({
        lhs: rg.Reference({ identifier: 'counters' }),
        rhs: rg.Access({
          lhs: rg.Reference({ identifier: 'board' }),
          rhs: rg.Reference({ identifier: 'coord' }),
        }),
      });

      context.$connect(
        from,
        counterDec,
        rg.Assignment({
          lhs: counterDecValue,
          rhs: context.$mathOperator(
            context.rbg.board.length + 1,
            counterDecValue,
            oneElement,
            '-',
          ),
        }),
      );

      // Increase.
      const counterInc = context.$randomEdgeName();
      const counterIncValue = rg.Access({
        lhs: rg.Reference({ identifier: 'counters' }),
        rhs: rg.Reference({ identifier: content.piece }),
      });

      context.$connect(
        counterDec,
        counterInc,
        rg.Assignment({
          lhs: counterIncValue,
          rhs: context.$mathOperator(
            context.rbg.board.length + 1,
            counterIncValue,
            oneElement,
            '+',
          ),
        }),
      );

      // Set piece.
      const setPiece = context.$randomEdgeName();
      context.$connect(
        counterInc,
        setPiece,
        rg.Assignment({
          lhs: rg.Access({
            lhs: rg.Reference({ identifier: 'board' }),
            rhs: rg.Reference({ identifier: 'coord' }),
          }),
          rhs: rg.Reference({ identifier: content.piece }),
        }),
      );

      // Expose position.
      exposePosition(context, setPiece, to);
      return;
    }
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
          makeShiftPattern(context, node.node, content),
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

          const constant = context.$createConstantFromMap(map, 'null');
          const local = context.$randomEdgeName();
          context.$connect(
            from,
            local,
            rg.Assignment({
              lhs: rg.Reference({ identifier: 'coord' }),
              rhs: rg.Access({
                lhs: rg.Reference({ identifier: constant }),
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
              type: usesAllCoords
                ? rg.TypeReference({ identifier: 'Coord' })
                : context.$createTypeFromSet(reachableCoords),
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
          rhs: rg.Reference({ identifier: content.label }),
        }),
        rhs: rg.Reference({ identifier: 'coord' }),
      });

      context.$connect(
        from,
        testCoord,
        rg.Assignment({
          lhs: rg.Reference({ identifier: 'coord' }),
          rhs: nextCoord,
        }),
      );
      context.$connect(
        testCoord,
        to,
        rg.Comparison({
          lhs: rg.Reference({ identifier: 'coord' }),
          rhs: rg.Reference({ identifier: 'null' }),
          negated: true,
        }),
      );
      return;
    }
    case 'Switch': {
      // Expose position.
      const local = context.$randomEdgeName();
      exposePosition(context, from, local);

      // Switch player.
      context.$connect(
        local,
        to,
        rg.Assignment({
          lhs: rg.Reference({ identifier: 'player' }),
          rhs: rg.Reference({ identifier: content.player ?? 'keeper' }),
        }),
      );
      return;
    }
  }
}

function translateGame(context: Context) {
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
        identifiers: utils.generate(
          Math.max(...context.rbg.players.map(({ bound }) => bound)) + 1,
          String,
        ),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Goals',
      type: rg.Arrow({
        lhs: rg.TypeReference({ identifier: 'Player' }),
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
        identifiers: ['null', ...context.rbg.board.map(node => node.node)],
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Board',
      type: rg.Arrow({
        lhs: rg.TypeReference({ identifier: 'Coord' }),
        rhs: rg.TypeReference({ identifier: 'Piece' }),
      }),
    }),
    rg.TypeDeclaration({
      identifier: 'Counters',
      type: rg.Arrow({
        lhs: rg.TypeReference({ identifier: 'Piece' }),
        rhs: context.$createTypeFromSet(
          utils.generate(context.rbg.board.length + 1, String),
        ),
      }),
    }),
  );

  const defaultDirection = rg.ValueEntry({
    identifier: null,
    value: rg.Element({ identifier: 'null' }),
  });

  context.rg.constants.push(
    rg.ConstantDeclaration({
      identifier: 'direction',
      type: rg.Arrow({
        lhs: rg.TypeReference({ identifier: 'Label' }),
        rhs: rg.Arrow({
          lhs: rg.TypeReference({ identifier: 'Coord' }),
          rhs: rg.TypeReference({ identifier: 'Coord' }),
        }),
      }),
      value: rg.Map({
        entries: [
          rg.ValueEntry({
            identifier: null,
            value: rg.Map({ entries: [defaultDirection] }),
          }),
          ...context.rbg.board
            .flatMap(node =>
              node.edges.map(edge => ({
                identifier: edge.label,
                valueEntry: rg.ValueEntry({
                  identifier: node.node,
                  value: rg.Element({ identifier: edge.node }),
                }),
              })),
            )
            .reduce<rg.ValueEntry[]>((entries, { identifier, valueEntry }) => {
              const existing = utils.find(entries, { identifier });
              if (existing) {
                utils.assert(existing.value.kind === 'Map', 'Expected Map.');
                existing.value.entries.push(valueEntry);
              } else {
                entries.push(
                  rg.ValueEntry({
                    identifier,
                    value: rg.Map({ entries: [defaultDirection, valueEntry] }),
                  }),
                );
              }

              return entries;
            }, []),
        ],
      }),
    }),
  );

  const mostCommonPiece = Object.entries(
    context.rbg.board.reduce<Record<string, number>>((pieces, { piece }) => {
      pieces[piece] ??= 0;
      pieces[piece]++;
      return pieces;
    }, {}),
  ).reduce((x, y) => (x[1] > y[1] ? x : y))[0];

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
        entries: [
          rg.ValueEntry({
            identifier: null,
            value: rg.Element({ identifier: '0' }),
          }),
        ],
      }),
    }),
    rg.VariableDeclaration({
      identifier: 'board',
      type: rg.TypeReference({ identifier: 'Board' }),
      defaultValue: rg.Map({
        entries: [
          rg.ValueEntry({
            identifier: null,
            value: rg.Element({ identifier: mostCommonPiece }),
          }),
          ...context.rbg.board.flatMap(node =>
            node.piece === mostCommonPiece
              ? []
              : [
                  rg.ValueEntry({
                    identifier: node.node,
                    value: rg.Element({ identifier: node.piece }),
                  }),
                ],
          ),
        ],
      }),
    }),
    rg.VariableDeclaration({
      identifier: 'coord',
      type: rg.TypeReference({ identifier: 'Coord' }),
      defaultValue: rg.Element({ identifier: context.rbg.board[0].node }),
    }),
    rg.VariableDeclaration({
      identifier: 'counters',
      type: rg.TypeReference({ identifier: 'Counters' }),
      defaultValue: rg.Map({
        entries: [
          rg.ValueEntry({
            identifier: null,
            value: rg.Element({ identifier: '0' }),
          }),
          ...context.rbg.pieces.map(piece =>
            rg.ValueEntry({
              identifier: piece,
              value: rg.Element({
                identifier: String(
                  context.rbg.board.filter(node => node.piece === piece).length,
                ),
              }),
            }),
          ),
        ],
      }),
    }),
  );

  for (const variable of context.rbg.variables) {
    translateVariable(context, variable);
  }

  translateAtomContent(
    context,
    context.rbg.rules,
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'begin' })] }),
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
  );

  removePowerSkipEdges(context);
  terminateOnZeroMoves(context);

  return context.rg;
}

function translateRValue(
  context: Context,
  rvalue: rbg.RValue,
  limit: number,
): rg.Expression {
  switch (typeof rvalue) {
    case 'number':
      return rg.Reference({ identifier: `${rvalue}` });
    case 'string':
      if (utils.find(context.rbg.players, { name: rvalue })) {
        return rg.Access({
          lhs: rg.Reference({ identifier: 'goals' }),
          rhs: rg.Reference({ identifier: rvalue }),
        });
      }

      if (context.rbg.pieces.includes(rvalue)) {
        return rg.Access({
          lhs: rg.Reference({ identifier: 'counters' }),
          rhs: rg.Reference({ identifier: rvalue }),
        });
      }

      return rg.Reference({ identifier: rvalue });
  }

  const lhs = translateRValue(context, rvalue.lhs, limit);
  const rhs = translateRValue(context, rvalue.rhs, limit);
  return context.$mathOperator(limit + 1, lhs, rhs, rvalue.operator);
}

function translateRValuePair(
  context: Context,
  lhs: rbg.RValue,
  rhs: rbg.RValue,
) {
  const limit = boundRValue(
    context,
    rbg.Expression({ lhs, rhs, operator: '+' }),
  );

  return {
    limit,
    lhs: translateRValue(context, lhs, limit),
    rhs: translateRValue(context, rhs, limit),
  };
}

function translateVariable(context: Context, variable: rbg.Variable) {
  context.rg.variables.push(
    rg.VariableDeclaration({
      identifier: variable.name,
      type: context.$createTypeFromSet(
        utils.generate(variable.bound + 1, String),
      ),
      defaultValue: rg.Element({ identifier: '0' }),
    }),
  );
}

function boundRValue(context: Context, rvalue: rbg.RValue): number {
  switch (typeof rvalue) {
    case 'number':
      return rvalue;
    case 'string': {
      const player = utils.find(context.rbg.players, { name: rvalue });
      if (player) {
        return player.bound;
      }

      const variable = utils.find(context.rbg.variables, { name: rvalue });
      if (variable) {
        return variable.bound;
      }

      if (context.rbg.pieces.includes(rvalue)) {
        return context.rbg.board.length;
      }

      utils.assert(false, `Unbounded rvalue ${rvalue}.`);
    }
  }

  return Math.max(
    boundRValue(context, rvalue.lhs),
    boundRValue(context, rvalue.rhs),
  );
}

function exposePosition(context: Context, from: rg.EdgeName, to: rg.EdgeName) {
  const localCoord = context.$randomEdgeName();
  const bind = 'coordGenerator';
  const type = rg.TypeReference({ identifier: 'Coord' });
  localCoord.parts.push(rg.Binding({ identifier: bind, type }));
  context.$connect(
    from,
    localCoord,
    rg.Comparison({
      lhs: rg.Reference({ identifier: bind }),
      rhs: rg.Reference({ identifier: 'coord' }),
      negated: false,
    }),
  );

  const localIndex = context.$randomEdgeName();
  context.$connect(localCoord, localIndex, rg.Tag({ symbol: bind }));

  const exposeTag = `index_${++context.$exposeIndex}`;
  context.$connect(localIndex, to, rg.Tag({ symbol: exposeTag }));
}

function hasMathExpression(expression: rg.Expression): boolean {
  switch (expression.kind) {
    case 'Access':
      return (
        hasMathExpression(expression.lhs) || hasMathExpression(expression.rhs)
      );
    case 'Cast':
      return hasMathExpression(expression.rhs);
    case 'Reference':
      return expression.identifier.startsWith('math_');
  }
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

function makeShiftPattern(context: Context, coord: string, rule: rbg.Rule) {
  const key = `${coord}:${rbg.serializeRule(rule)}`;
  if (!(key in context.$shiftPatternsCache)) {
    context.$shiftPatternsCache[key] = rule.elements
      .flatMap(concatenation =>
        concatenation.reduce(
          (coords, { content, power }) => {
            const reachableCoords = power ? coords.slice() : [];
            for (let coord of coords) {
              switch (content.kind) {
                case 'Shift': {
                  const { label } = content;
                  do {
                    const node = utils.find(context.rbg.board, { node: coord });
                    const edge = utils.find(node?.edges, { label });
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
                  for (const node of makeShiftPattern(
                    context,
                    coord,
                    content,
                  )) {
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

  return context.$shiftPatternsCache[key];
}

function removePowerSkipEdges(context: Context) {
  function isSkipToEnd(edge: rg.EdgeDeclaration) {
    return (
      edge.rhs.parts.length === 1 &&
      edge.rhs.parts[0].kind === 'Literal' &&
      edge.rhs.parts[0].identifier === 'end' &&
      edge.label.kind === 'Skip'
    );
  }

  let edgesCount = Infinity;
  while (edgesCount !== context.rg.edges.length) {
    edgesCount = context.rg.edges.length;
    for (const x of context.rg.edges.slice()) {
      if (isSkipToEnd(x)) {
        for (const y of rg.lib.incoming(context.rg.edges, x.lhs)) {
          y.rhs = x.rhs;
        }
      }
    }

    for (const x of context.rg.edges.slice()) {
      if (isSkipToEnd(x)) {
        utils.remove(context.rg.edges, x);
      }
    }
  }
}

function copyPath(
  context: Context,
  originalFrom: rg.EdgeName,
  originalTo: rg.EdgeName,
) {
  const error = 'Only simple nodes can be copied.';
  utils.assert(originalFrom.parts.length === 1, error);
  utils.assert(originalFrom.parts[0].kind === 'Literal', error);
  utils.assert(originalTo.parts.length === 1, error);
  utils.assert(originalTo.parts[0].kind === 'Literal', error);

  const prefix = `${originalFrom.parts[0].identifier}_${originalTo.parts[0].identifier}`;
  function prefixEdgeName({ parts }: rg.EdgeName) {
    switch (parts[0].kind) {
      case 'Binding':
        return rg.EdgeName({
          parts: [rg.Literal({ identifier: prefix }), ...parts],
        });
      case 'Literal':
        return rg.EdgeName({
          parts: [
            rg.Literal({ identifier: `${prefix}_${parts[0].identifier}` }),
            ...parts.slice(1),
          ],
        });
    }
  }

  function copy(edge: rg.EdgeDeclaration, distance: number) {
    // If the edge cannot reach the end _yet_, we check whether it is on a cycle
    // and if so, then add it anyway. It will copy too many edges, though.
    if (distance === Infinity) {
      const hash = JSON.stringify(edge.rhs);
      if (!(hash in distances) || distances[hash] !== null) {
        return;
      }
    }

    const copiedEdge = rg.EdgeDeclaration({
      lhs: prefixEdgeName(edge.lhs),
      rhs: prefixEdgeName(edge.rhs),
      label: edge.label,
    });

    // Skip switch-generated edges:
    //   - coordGenerator == coord
    //   - $ coordGenerator
    //   - $ index_N
    if (distance <= 3) {
      copiedEdge.lhs.parts.splice(1, Infinity);
      copiedEdge.rhs.parts.splice(1, Infinity);
      copiedEdge.label = rg.Skip({});
    }

    utils.unique(context.rg.edges, copiedEdge);
  }

  /**
   * Represent minimum distance to `originalTo`. A `null` is an intermediate
   * state where we don't know if it's reachable or no. (It is used to copy
   * edges on cycles.) An `Infinity` means the `originalTo` is not reachable. */
  const distances: Record<string, null | number> = Object.create(null);
  function copyIfOnPath(edgeName: rg.EdgeName): number | null {
    const hash = JSON.stringify(edgeName);
    if (!(hash in distances)) {
      distances[hash] = utils.isEqual(edgeName, originalTo) ? 0 : null;

      // If it's not reached, copy and check.
      if (distances[hash] === null) {
        for (const next of rg.lib.outgoing(context.rg.edges, edgeName)) {
          if (
            next.label.kind !== 'Assignment' ||
            next.label.lhs.kind !== 'Reference' ||
            next.label.lhs.identifier !== 'player'
          ) {
            const distance = 1 + (copyIfOnPath(next.rhs) ?? Infinity);
            copy(next, distance);
            distances[hash] = Math.min(distances[hash] ?? Infinity, distance);
          }
        }
      }

      // If it wasn't reached, mark it as not reachable.
      if (distances[hash] === null) {
        distances[hash] = Infinity;
      }
    }

    return distances[hash];
  }

  copyIfOnPath(originalFrom);
  return { lhs: prefixEdgeName(originalFrom), rhs: prefixEdgeName(originalTo) };
}

// eslint-disable-next-line complexity -- Simplify it later.
function terminateOnZeroMoves(context: Context) {
  const moves: {
    edge: rg.EdgeDeclaration;
    lhs: rg.EdgeName;
    rhs: rg.EdgeName;
  }[] = [];

  // 1. For every `A, B: player = P`, where `P != keeper`.
  //   2. Find all paths from `B` to `D` ending in `E, _: player = *`.
  //   3. Add new edges from `B` to `end` with all `! C -> E`, where `C` is a fresh node between `B` and `D`.
  for (const edge of context.rg.edges) {
    const { rhs: B, label } = edge;
    if (
      label.kind !== 'Assignment' ||
      label.lhs.kind !== 'Reference' ||
      label.lhs.identifier !== 'player'
    ) {
      continue;
    }

    const visited = new Set<string>();
    const reachablePlayerAssignments: rg.EdgeName[] = [];

    for (const { rhs: C } of rg.lib.outgoing(context.rg.edges, B)) {
      const queue = [C];
      for (let node: rg.EdgeName | undefined; (node = queue.pop()); ) {
        for (const edge of rg.lib.outgoing(context.rg.edges, node)) {
          if (
            edge.label.kind === 'Assignment' &&
            edge.label.lhs.kind === 'Reference' &&
            edge.label.lhs.identifier === 'player'
          ) {
            utils.unique(reachablePlayerAssignments, edge.lhs);
          } else {
            const hash = JSON.stringify(edge.rhs);
            if (!visited.has(hash)) {
              visited.add(hash);
              queue.push(edge.rhs);
            }
          }
        }
      }
    }

    if (reachablePlayerAssignments.length === 0) {
      continue;
    }

    switch (reachablePlayerAssignments.length) {
      case 0:
        continue;
      case 1: {
        const { lhs, rhs } = copyPath(
          context,
          B,
          reachablePlayerAssignments[0],
        );
        moves.push({ edge, lhs, rhs });
        break;
      }
      default: {
        const checkFrom = context.$randomEdgeName();
        const checkTo = context.$randomEdgeName();
        for (const C of reachablePlayerAssignments) {
          const { lhs, rhs } = copyPath(context, B, C);
          context.$connect(
            checkFrom,
            checkTo,
            rg.Reachability({ lhs, rhs, negated: false }),
          );
        }

        moves.push({ edge, lhs: checkFrom, rhs: checkTo });
        break;
      }
    }
  }

  for (const { edge, lhs, rhs } of moves) {
    const { label, lhs: A, rhs: B } = edge;
    utils.remove(context.rg.edges, edge);

    const check = context.$randomEdgeName();
    context.$connect(A, check, rg.Reachability({ lhs, rhs, negated: false }));
    context.$connect(check, B, label);

    const assign = context.$randomEdgeName();
    context.$connect(A, assign, rg.Reachability({ lhs, rhs, negated: true }));
    context.$connect(
      assign,
      rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
      rg.Assignment({
        lhs: rg.Reference({ identifier: 'player' }),
        rhs: rg.Reference({ identifier: 'keeper' }),
      }),
    );
  }
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
        rhs: this.$createTypeFromSet(
          pairs
            .map(pair => pair[1])
            .reduce<string[]>(utils.unique, [])
            .sort(),
        ),
      });

      const value = rg.Map({
        entries: [
          rg.ValueEntry({
            identifier: null,
            value: rg.Element({ identifier: defaultValue }),
          }),
          ...pairs
            .filter(([, to]) => to !== defaultValue)
            .sort()
            .map(([from, to]) =>
              rg.ValueEntry({
                identifier: from,
                value: rg.Element({ identifier: to }),
              }),
            ),
        ],
      });

      const { constants } = this.rg;
      const existing = utils.find(constants, { type, value });
      if (existing) {
        return existing.identifier;
      }

      const identifier = utils.generateIdentifier(constants, 'RbgCoordMap1');
      constants.push(rg.ConstantDeclaration({ identifier, type, value }));

      return identifier;
    },
    $createTypeFromSet(identifiers) {
      const type = rg.Set({ identifiers });
      const { types } = this.rg;
      const existing = utils.find(types, { type });
      if (existing) {
        return rg.TypeReference({ identifier: existing.identifier });
      }

      const identifier = utils.generateIdentifier(types, 'RbgType1');
      types.push(rg.TypeDeclaration({ identifier, type }));

      return rg.TypeReference({ identifier });
    },
    $exposeIndex: 0,
    $mathOperator(limit, lhs, rhs, operator) {
      const mathOperator = [
        'math',
        {
          '*': 'mul',
          '+': 'add',
          '-': 'sub',
          '/': 'div',
          '<': 'lt',
          '<=': 'lte',
          '>': 'gt',
          '>=': 'gte',
        }[operator],
        limit,
      ].join('_');

      if (!utils.find(this.rg.constants, { identifier: mathOperator })) {
        const nanElement = rg.Element({ identifier: 'nan' });
        const nanEntry = rg.ValueEntry({ identifier: null, value: nanElement });

        const numberType = this.$mathType(limit);

        this.rg.constants.push(
          rg.ConstantDeclaration({
            identifier: mathOperator,
            type: rg.Arrow({
              lhs: numberType,
              rhs: rg.Arrow({ lhs: numberType, rhs: numberType }),
            }),
            value: rg.Map({
              entries: [
                rg.ValueEntry({
                  identifier: null,
                  value: rg.Map({ entries: [nanEntry] }),
                }),
                ...utils.generate(limit, lhs =>
                  rg.ValueEntry({
                    identifier: `${lhs}`,
                    value: rg.Map({
                      entries: [
                        nanEntry,
                        ...utils
                          .generate(limit, rhs => {
                            let result: number | null;
                            switch (operator) {
                              case '+':
                                result = lhs + rhs >= limit ? null : lhs + rhs;
                                break;
                              case '-':
                                result = lhs - rhs < 0 ? null : lhs - rhs;
                                break;
                              case '<':
                                result = Number(lhs < rhs);
                                break;
                              case '<=':
                                result = Number(lhs <= rhs);
                                break;
                              case '>':
                                result = Number(lhs > rhs);
                                break;
                              case '>=':
                                result = Number(lhs >= rhs);
                                break;
                              default:
                                throw new Error(
                                  `Not implemented ($mathOperator(${operator})).`,
                                );
                            }

                            if (result === null) {
                              return null;
                            }

                            return rg.ValueEntry({
                              identifier: `${rhs}`,
                              value: rg.Element({ identifier: String(result) }),
                            });
                          })
                          .filter(utils.isNotNull),
                      ],
                    }),
                  }),
                ),
              ],
            }),
          }),
        );
      }

      return rg.Access({
        lhs: rg.Access({
          lhs: rg.Reference({ identifier: mathOperator }),
          rhs: lhs,
        }),
        rhs,
      });
    },
    $mathType(limit: number) {
      return this.$createTypeFromSet(['nan', ...utils.generate(limit, String)]);
    },
    $randomEdgeName() {
      const identifier = `${++counter}`;
      return rg.EdgeName({ parts: [rg.Literal({ identifier })] });
    },
    $shiftPatternsCache: Object.create(null),
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
      pragmas: [],
      types: [],
      variables: [],
    }),
    ruleAutomatons: Object.create(null),
  });
}
