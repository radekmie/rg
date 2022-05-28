import { ast as hrg } from '../hrg';
import { ast as rg } from '../rg';
import * as utils from '../utils';

enum Ord {
  Eq,
  Gt,
  Lt,
}

function numberToOrd(number: number) {
  return number === 0 ? Ord.Eq : number < 0 ? Ord.Lt : Ord.Gt;
}

function compareValues(lhs: hrg.Value, rhs: hrg.Value): Ord {
  switch (lhs.kind) {
    case 'ValueConstructor': {
      utils.assert(rhs.kind === 'ValueConstructor', 'Incomparable values.');
      const ord1 = numberToOrd(lhs.identifier.localeCompare(rhs.identifier));
      if (ord1 !== Ord.Eq) {
        return ord1;
      }

      const ord2 = numberToOrd(lhs.args.length - rhs.args.length);
      if (ord2 !== Ord.Eq) {
        return ord2;
      }

      for (let index = 0; index < lhs.args.length; ++index) {
        const ord3 = compareValues(lhs.args[index], rhs.args[index]);
        if (ord3 !== Ord.Eq) {
          return ord3;
        }
      }

      return Ord.Eq;
    }
    case 'ValueElement':
      utils.assert(rhs.kind === 'ValueElement', 'Incomparable values.');
      return numberToOrd(lhs.identifier.localeCompare(rhs.identifier));
    case 'ValueMap':
      throw new Error('Not implemented.');
  }
}

function constructMap(entries: rg.NamedEntry[]) {
  utils.assert(entries.length > 0, 'At least one entry is required.');

  type Count = { count: number; value: rg.Value };
  const valueCounts = entries.reduce<Record<string, Count>>((counts, entry) => {
    const hash = JSON.stringify(entry.value);
    if (hash in counts) {
      counts[hash].count++;
    } else {
      counts[hash] = { count: 1, value: entry.value };
    }

    return counts;
  }, Object.create(null));

  const defaultValue = Object.values(valueCounts).sort(
    (a, b) => b.count - a.count,
  )[0].value;

  return rg.Map({
    entries: [
      rg.DefaultEntry({ value: defaultValue }),
      ...entries.filter(entry => !evaluateEquality(entry.value, defaultValue)),
    ],
  });
}

function evaluateBinding(
  pattern: hrg.Pattern,
  value: hrg.Value,
): Record<string, hrg.Value> | undefined {
  switch (pattern.kind) {
    case 'PatternConstructor':
      return value.kind === 'ValueConstructor' &&
        value.identifier === pattern.identifier &&
        value.args.length === pattern.args.length
        ? pattern.args.reduce<Record<string, hrg.Value> | undefined>(
            (binding, pattern, index) => {
              if (binding) {
                const subbinding = evaluateBinding(pattern, value.args[index]);
                if (subbinding) {
                  return Object.assign(binding, subbinding);
                }
              }
            },
            {},
          )
        : undefined;
    case 'PatternLiteral':
      return value.kind === 'ValueElement' &&
        value.identifier === pattern.identifier
        ? {}
        : undefined;
    case 'PatternVariable':
      return { [pattern.identifier]: value };
    case 'PatternWildcard':
      return {};
  }
}

// eslint-disable-next-line complexity -- This function could be improved.
function evaluateCondition(
  expression: hrg.Expression,
  binding: Record<string, hrg.Value>,
): boolean {
  if (
    expression.kind !== 'ExpressionGt' &&
    expression.kind !== 'ExpressionGte' &&
    expression.kind !== 'ExpressionEq' &&
    expression.kind !== 'ExpressionLt' &&
    expression.kind !== 'ExpressionLte' &&
    expression.kind !== 'ExpressionNe'
  ) {
    throw new Error(
      `Expression "${expression.kind}" is not a valid condition.`,
    );
  }

  const lhs = evaluateExpression(expression.lhs, binding);
  const rhs = evaluateExpression(expression.rhs, binding);
  const ord = compareValues(lhs, rhs);
  switch (expression.kind) {
    case 'ExpressionGt':
      return ord === Ord.Gt;
    case 'ExpressionGte':
      return ord === Ord.Gt || ord === Ord.Eq;
    case 'ExpressionEq':
      return ord === Ord.Eq;
    case 'ExpressionLt':
      return ord === Ord.Lt;
    case 'ExpressionLte':
      return ord === Ord.Lt || ord === Ord.Eq;
    case 'ExpressionNe':
      return ord !== Ord.Eq;
  }
}

function evaluateDefaultValue(
  type: rg.Type,
  typeValues: Record<string, hrg.Value[]>,
): hrg.Value {
  switch (type.kind) {
    case 'Arrow':
      utils.assert(
        type.lhs in typeValues,
        `Unresolved TypeReference "${type.lhs}".`,
      );
      utils.assert(
        typeValues[type.lhs].length,
        'Expected at least one identifier.',
      );
      // NOTE: Is this even correct?
      return hrg.ValueMap({
        entries: typeValues[type.lhs].map(value =>
          hrg.ValueMapEntry({
            key: value,
            value: evaluateDefaultValue(type.rhs, typeValues),
          }),
        ),
      });
    case 'Set':
      throw new Error('Not implemented (Set).');
    case 'TypeReference':
      utils.assert(
        type.identifier in typeValues,
        `Unresolved TypeReference "${type.identifier}".`,
      );
      utils.assert(
        typeValues[type.identifier].length,
        'Expected at least one identifier.',
      );
      return typeValues[type.identifier][0];
  }
}

function evaluateDomainValues(
  domainValues: hrg.DomainValue[],
): Record<string, hrg.Value>[] {
  domainValues.slice(1).forEach(domainValue => {
    utils.assert(
      domainValues[0].identifier !== domainValue.identifier,
      'Duplicated identifier.',
    );
  });

  return domainValues
    .map(domainValue => {
      switch (domainValue.kind) {
        case 'DomainRange': {
          const max = +domainValue.max;
          const min = +domainValue.min;
          return Array.from({ length: max - min + 1 }, (_, index) => ({
            [domainValue.identifier]: hrg.ValueElement({
              identifier: `${index + min}`,
            }),
          }));
        }
        case 'DomainSet':
          return domainValue.elements.map(identifier => ({
            [domainValue.identifier]: hrg.ValueElement({ identifier }),
          }));
      }
    })
    .reduce<Record<string, hrg.Value>[][]>(utils.cartesian, [[]])
    .map(bindings => bindings.reduce((a, b) => Object.assign(a, b), {}));
}

function evaluateEquality(lhs: rg.Value, rhs: rg.Value): boolean {
  switch (lhs.kind) {
    case 'Element':
      utils.assert(rhs.kind === 'Element', 'Equality for different kinds.');
      return lhs.identifier === rhs.identifier;
    case 'Map':
      throw new Error('Not implemented (Map).');
  }
}

// eslint-disable-next-line complexity -- This function could be improved.
function evaluateExpression(
  expression: hrg.Expression,
  binding: Record<string, hrg.Value>,
): hrg.Value {
  switch (expression.kind) {
    case 'ExpressionAccess':
      throw new Error('Not implemented (ExpressionAccess).');
    case 'ExpressionAdd': {
      const lhs = evaluateExpressionIdentifier(expression.lhs, binding);
      const rhs = evaluateExpressionIdentifier(expression.rhs, binding);
      return hrg.ValueElement({ identifier: String(Number(lhs) + Number(rhs)) });
    }
    case 'ExpressionAnd':
      throw new Error('Not implemented (ExpressionAnd).');
    case 'ExpressionCall':
      throw new Error('Not implemented (ExpressionCall).');
    case 'ExpressionConstructor':
      return hrg.ValueConstructor({
        identifier: expression.identifier,
        args: expression.args.map(expression =>
          evaluateExpression(expression, binding),
        ),
      });
    case 'ExpressionEq':
      throw new Error('Not implemented (ExpressionEq).');
    case 'ExpressionGt':
      throw new Error('Not implemented (ExpressionGt).');
    case 'ExpressionGte':
      throw new Error('Not implemented (ExpressionGte).');
    case 'ExpressionIf':
      return evaluateCondition(expression.cond, binding)
        ? evaluateExpression(expression.then, binding)
        : evaluateExpression(expression.else, binding);
    case 'ExpressionLiteral':
      return expression.identifier in binding
        ? binding[expression.identifier]
        : hrg.ValueElement({ identifier: expression.identifier });
    case 'ExpressionLt':
      throw new Error('Not implemented (ExpressionLt).');
    case 'ExpressionLte':
      throw new Error('Not implemented (ExpressionLte).');
    case 'ExpressionOr':
      throw new Error('Not implemented (ExpressionOr).');
    case 'ExpressionMap':
      return hrg.ValueMap({
        entries: evaluateDomainValues(expression.domains)
          .map(subbinding => Object.assign({}, binding, subbinding))
          .map(binding =>
            hrg.ValueMapEntry({
              key: evaluatePattern(expression.pattern, binding),
              value: evaluateExpression(expression.expression, binding),
            }),
          ),
      });
    case 'ExpressionNe':
      throw new Error('Not implemented (ExpressionNe).');
    case 'ExpressionSub': {
      const lhs = evaluateExpressionIdentifier(expression.lhs, binding);
      const rhs = evaluateExpressionIdentifier(expression.rhs, binding);
      return hrg.ValueElement({ identifier: String(Number(lhs) - Number(rhs)) });
    }
  }
}

function evaluateExpressionIdentifier(
  expression: hrg.Expression,
  binding: Record<string, hrg.Value>,
): string {
  const value = evaluateExpression(expression, binding);
  utils.assert(value.kind === 'ValueElement', 'Expected ValueElement.');
  return value.identifier;
}

function evaluatePattern(
  pattern: hrg.Pattern,
  binding: Record<string, hrg.Value>,
): hrg.Value {
  switch (pattern.kind) {
    case 'PatternConstructor':
      return hrg.ValueConstructor({
        identifier: pattern.identifier,
        args: pattern.args.map(pattern => evaluatePattern(pattern, binding)),
      });
    case 'PatternLiteral':
      return hrg.ValueElement({ identifier: pattern.identifier });
    case 'PatternVariable':
      utils.assert(
        pattern.identifier in binding,
        `Unknown variable "${pattern.identifier}".`,
      );
      return binding[pattern.identifier];
    case 'PatternWildcard':
      throw new Error('PatternWildcard is not evaluable.');
  }
}

function serializeValue(value: hrg.Value): string {
  switch (value.kind) {
    case 'ValueConstructor': {
      const args = value.args.map(serializeValue);
      return `${value.identifier.toLowerCase()}__${args.join('_')}`;
    }
    case 'ValueElement':
      return value.identifier.toLowerCase();
    case 'ValueMap':
      throw new Error('Not implemented.');
  }
}

function translateAutomatonFunction(
  automatonFunction: hrg.AutomatonFunction,
  automatonFunctions: hrg.AutomatonFunction[],
  exitEdgeName: rg.EdgeName | null,
  returnEdgeName: rg.EdgeName | null,
  edges: rg.EdgeDeclaration[],
  variableDeclarations: rg.VariableDeclaration[],
  prefix: string,
) {
  translateAutomatonStatements(
    automatonFunction.body,
    automatonFunctions,
    rg.EdgeName({
      parts: [
        rg.Literal({ identifier: `${prefix}${automatonFunction.name}_0` }),
      ],
    }),
    exitEdgeName,
    returnEdgeName,
    edges,
    variableDeclarations,
    `${prefix}${automatonFunction.name}`,
  );
}

const globalCounters: Record<string, number> = Object.create(null);
function RANDOM(prefix: string) {
  if (!(prefix in globalCounters)) {
    globalCounters[prefix] = 0;
  }

  return `${prefix}_${++globalCounters[prefix]}`;
}

function RANDOM_RESET() {
  for (const key of Object.keys(globalCounters)) {
    delete globalCounters[key];
  }
}

function translateAutomatonStatements(
  automatonStatements: hrg.AutomatonStatement[],
  automatonFunctions: hrg.AutomatonFunction[],
  entryEdgeName: rg.EdgeName,
  exitEdgeName: rg.EdgeName | null,
  returnEdgeName: rg.EdgeName | null,
  edges: rg.EdgeDeclaration[],
  variableDeclarations: rg.VariableDeclaration[],
  prefix: string,
) {
  let currentEdgeName = entryEdgeName;
  for (const automatonStatement of automatonStatements) {
    switch (automatonStatement.kind) {
      case 'AutomatonAssignment': {
        const nextEdgeName = rg.EdgeName({
          parts: [rg.Literal({ identifier: RANDOM(prefix) })],
        });
        edges.push(
          rg.EdgeDeclaration({
            lhs: currentEdgeName,
            rhs: nextEdgeName,
            label: rg.Assignment({
              lhs: automatonStatement.accessors.reduce<rg.Expression>(
                (expression, accessor) =>
                  rg.Access({
                    lhs: expression,
                    rhs: translateExpression(accessor),
                  }),
                rg.Reference({ identifier: automatonStatement.identifier }),
              ),
              rhs: translateExpression(automatonStatement.expression),
            }),
          }),
        );
        currentEdgeName = nextEdgeName;
        continue;
      }
      case 'AutomatonBranch': {
        const nextEdgeName = rg.EdgeName({
          parts: [rg.Literal({ identifier: RANDOM(prefix) })],
        });
        automatonStatement.arms.forEach(arm => {
          translateAutomatonStatements(
            arm,
            automatonFunctions,
            currentEdgeName,
            exitEdgeName,
            nextEdgeName,
            edges,
            variableDeclarations,
            prefix,
          );
        });
        currentEdgeName = nextEdgeName;
        continue;
      }
      case 'AutomatonCall':
        switch (automatonStatement.identifier) {
          case 'assert': {
            utils.assert(
              automatonStatement.args.length === 1,
              'assert expects 1 argument',
            );
            const nextEdgeName = rg.EdgeName({
              parts: [rg.Literal({ identifier: RANDOM(prefix) })],
            });
            translateCondition(
              automatonStatement.args[0],
              currentEdgeName,
              nextEdgeName,
              null,
              edges,
              prefix,
              automatonFunctions,
              variableDeclarations,
            );
            currentEdgeName = nextEdgeName;
            continue;
          }
          case 'forall': {
            utils.assert(
              automatonStatement.args.length === 1,
              'forall expects 1 argument',
            );
            const assignmentVariable = automatonStatement.args[0];
            utils.assert(
              assignmentVariable.kind === 'ExpressionLiteral',
              'forall expects a literal',
            );
            const assignmentVariableDeclaration = variableDeclarations.find(
              variableDeclaration =>
                variableDeclaration.identifier ===
                assignmentVariable.identifier,
            );
            utils.assert(
              assignmentVariableDeclaration,
              `Unknown variable "${assignmentVariable.identifier}" in forall.`,
            );
            const identifier = RANDOM('x');
            const assignmentEdgeName = rg.EdgeName({
              parts: [
                rg.Literal({ identifier: RANDOM(prefix) }),
                rg.Binding({
                  identifier,
                  type: assignmentVariableDeclaration.type,
                }),
              ],
            });
            edges.push(
              rg.EdgeDeclaration({
                lhs: currentEdgeName,
                rhs: assignmentEdgeName,
                label: rg.Assignment({
                  lhs: rg.Reference({
                    identifier: assignmentVariable.identifier,
                  }),
                  rhs: rg.Cast({
                    lhs: assignmentVariableDeclaration.type,
                    rhs: rg.Reference({ identifier }),
                  }),
                }),
              }),
            );
            currentEdgeName = assignmentEdgeName;
            continue;
          }
          case 'return': {
            utils.assert(
              automatonStatements.indexOf(automatonStatement) ===
                automatonStatements.length - 1,
              'Return has to be the last statement.',
            );
            utils.assert(exitEdgeName, 'Return requires exitEdgeName.');
            edges.push(
              rg.EdgeDeclaration({
                lhs: currentEdgeName,
                rhs: exitEdgeName,
                label: rg.Skip({}),
              }),
            );
            return;
          }
          default: {
            const automatonFunction = automatonFunctions.find(
              automatonFunction =>
                automatonFunction.name === automatonStatement.identifier,
            );
            utils.assert(
              automatonFunction,
              `Unknown automaton function "${automatonStatement.identifier}".`,
            );
            edges.push(
              rg.EdgeDeclaration({
                lhs: currentEdgeName,
                rhs: rg.EdgeName({
                  parts: [
                    rg.Literal({ identifier: `${automatonFunction.name}_0` }),
                  ],
                }),
                label: rg.Skip({}),
              }),
            );
            const nextEdgeName = rg.EdgeName({
              parts: [rg.Literal({ identifier: RANDOM(prefix) })],
            });
            translateAutomatonFunction(
              automatonFunction,
              automatonFunctions,
              exitEdgeName,
              nextEdgeName,
              edges,
              variableDeclarations,
              '',
            );
            currentEdgeName = nextEdgeName;
            continue;
          }
        }
      case 'AutomatonLoop':
        utils.assert(
          automatonStatements.indexOf(automatonStatement) ===
            automatonStatements.length - 1,
          'Loop has to be the last statement.',
        );
        translateAutomatonStatements(
          automatonStatement.body,
          automatonFunctions,
          entryEdgeName,
          exitEdgeName,
          currentEdgeName,
          edges,
          variableDeclarations,
          prefix,
        );
        return;
      case 'AutomatonWhen': {
        const thenEdgeName = rg.EdgeName({
          parts: [rg.Literal({ identifier: RANDOM(prefix) })],
        });
        const elseEdgeName = rg.EdgeName({
          parts: [rg.Literal({ identifier: RANDOM(prefix) })],
        });
        translateCondition(
          automatonStatement.expression,
          currentEdgeName,
          thenEdgeName,
          elseEdgeName,
          edges,
          prefix,
          automatonFunctions,
          variableDeclarations,
        );
        translateAutomatonStatements(
          automatonStatement.body,
          automatonFunctions,
          thenEdgeName,
          exitEdgeName,
          elseEdgeName,
          edges,
          variableDeclarations,
          prefix,
        );
        currentEdgeName = elseEdgeName;
        continue;
      }
      case 'AutomatonWhile':
        throw new Error('Not implemented (AutomatonWhile).');
    }
  }

  if (returnEdgeName) {
    edges.push(
      rg.EdgeDeclaration({
        lhs: currentEdgeName,
        rhs: returnEdgeName,
        label: rg.Skip({}),
      }),
    );
  }
}

// eslint-disable-next-line complexity -- This function could be improved.
function translateCondition(
  expression: hrg.Expression,
  entryEdgeName: rg.EdgeName,
  thenEdgeName: rg.EdgeName | null,
  elseEdgeName: rg.EdgeName | null,
  edges: rg.EdgeDeclaration[],
  prefix: string,
  automatonFunctions: hrg.AutomatonFunction[],
  variableDeclarations: rg.VariableDeclaration[],
) {
  switch (expression.kind) {
    case 'ExpressionAccess':
      throw new Error('Not implemented (ExpressionAccess).');
    case 'ExpressionAdd':
      throw new Error('Not implemented (ExpressionAdd).');
    case 'ExpressionAnd':
      throw new Error('Not implemented (ExpressionAnd).');
    case 'ExpressionCall': {
      utils.assert(
        expression.expression.kind === 'ExpressionLiteral',
        'Call expects a literal',
      );
      switch (expression.expression.identifier) {
        case 'not':
          utils.assert(
            expression.args.length === 1,
            'reachable expects 1 argument',
          );
          translateCondition(
            expression.args[0],
            entryEdgeName,
            elseEdgeName,
            thenEdgeName,
            edges,
            prefix,
            automatonFunctions,
            variableDeclarations,
          );
          return;
        case 'reachable': {
          utils.assert(
            expression.args.length === 1,
            'reachable expects 1 argument',
          );

          const call = expression.args[0];
          utils.assert(
            call.kind === 'ExpressionCall',
            'reachable expects an automaton call',
          );
          utils.assert(
            call.expression.kind === 'ExpressionLiteral',
            'reachable expects an automaton call',
          );

          const automatonName = call.expression.identifier;
          const automatonPrefix = RANDOM(prefix);
          const automatonFunction = automatonFunctions.find(
            automatonFunction => automatonFunction.name === automatonName,
          );
          utils.assert(
            automatonFunction,
            `Unknown automaton function "${call.expression.identifier}".`,
          );

          const automatonStartEdgeName = rg.EdgeName({
            parts: [rg.Literal({ identifier: RANDOM(automatonPrefix) })],
          });
          let automatonCurrentEdgeName = automatonStartEdgeName;
          for (const arg of call.args) {
            const argEdgeName = rg.EdgeName({
              parts: [rg.Literal({ identifier: RANDOM(automatonPrefix) })],
            });
            edges.push(
              rg.EdgeDeclaration({
                lhs: automatonCurrentEdgeName,
                rhs: argEdgeName,
                label: rg.Assignment({
                  lhs: rg.Reference({
                    identifier:
                      automatonFunction.args[call.args.indexOf(arg)].identifier,
                  }),
                  rhs: translateExpression(arg),
                }),
              }),
            );
            automatonCurrentEdgeName = argEdgeName;
          }

          edges.push(
            rg.EdgeDeclaration({
              lhs: automatonCurrentEdgeName,
              rhs: rg.EdgeName({
                parts: [
                  rg.Literal({
                    identifier: `${automatonPrefix}_${automatonName}_0`,
                  }),
                ],
              }),
              label: rg.Skip({}),
            }),
          );
          const automatonEndEdgeName = rg.EdgeName({
            parts: [rg.Literal({ identifier: RANDOM(automatonPrefix) })],
          });
          translateAutomatonFunction(
            automatonFunction,
            automatonFunctions,
            automatonEndEdgeName,
            automatonEndEdgeName,
            edges,
            variableDeclarations,
            `${automatonPrefix}_`,
          );

          if (thenEdgeName) {
            edges.push(
              rg.EdgeDeclaration({
                lhs: entryEdgeName,
                rhs: thenEdgeName,
                label: rg.Reachability({
                  lhs: automatonStartEdgeName,
                  rhs: automatonEndEdgeName,
                  negated: false,
                }),
              }),
            );
          }
          if (elseEdgeName) {
            edges.push(
              rg.EdgeDeclaration({
                lhs: entryEdgeName,
                rhs: elseEdgeName,
                label: rg.Reachability({
                  lhs: automatonStartEdgeName,
                  rhs: automatonEndEdgeName,
                  negated: true,
                }),
              }),
            );
          }
          return;
        }
        default:
          utils.assert(
            false,
            `Unknown condition function ${expression.expression.identifier}`,
          );
      }
      break;
    }
    case 'ExpressionConstructor':
      throw new Error('Not implemented (ExpressionConstructor).');
    case 'ExpressionEq':
      if (thenEdgeName) {
        edges.push(
          rg.EdgeDeclaration({
            lhs: entryEdgeName,
            rhs: thenEdgeName,
            label: rg.Comparison({
              lhs: translateExpression(expression.lhs),
              rhs: translateExpression(expression.rhs),
              negated: false,
            }),
          }),
        );
      }
      if (elseEdgeName) {
        edges.push(
          rg.EdgeDeclaration({
            lhs: entryEdgeName,
            rhs: elseEdgeName,
            label: rg.Comparison({
              lhs: translateExpression(expression.lhs),
              rhs: translateExpression(expression.rhs),
              negated: true,
            }),
          }),
        );
      }
      return;
    case 'ExpressionGt':
      throw new Error('Not implemented (ExpressionGt).');
    case 'ExpressionGte':
      throw new Error('Not implemented (ExpressionGte).');
    case 'ExpressionIf':
      throw new Error('Not implemented (ExpressionIf).');
    case 'ExpressionLiteral':
      throw new Error('Not implemented (ExpressionLiteral).');
    case 'ExpressionLt':
      throw new Error('Not implemented (ExpressionLt).');
    case 'ExpressionLte':
      throw new Error('Not implemented (ExpressionLte).');
    case 'ExpressionMap':
      throw new Error('Not implemented (ExpressionMap).');
    case 'ExpressionNe':
      if (thenEdgeName) {
        edges.push(
          rg.EdgeDeclaration({
            lhs: entryEdgeName,
            rhs: thenEdgeName,
            label: rg.Comparison({
              lhs: translateExpression(expression.lhs),
              rhs: translateExpression(expression.rhs),
              negated: true,
            }),
          }),
        );
      }
      if (elseEdgeName) {
        edges.push(
          rg.EdgeDeclaration({
            lhs: entryEdgeName,
            rhs: elseEdgeName,
            label: rg.Comparison({
              lhs: translateExpression(expression.lhs),
              rhs: translateExpression(expression.rhs),
              negated: false,
            }),
          }),
        );
      }
      return;
    case 'ExpressionOr': {
      const falseEdgeName = rg.EdgeName({
        parts: [rg.Literal({ identifier: RANDOM(prefix) })],
      });
      translateCondition(
        expression.lhs,
        entryEdgeName,
        thenEdgeName,
        falseEdgeName,
        edges,
        prefix,
        automatonFunctions,
        variableDeclarations,
      );
      translateCondition(
        expression.rhs,
        falseEdgeName,
        thenEdgeName,
        elseEdgeName,
        edges,
        prefix,
        automatonFunctions,
        variableDeclarations,
      );
      return;
    }
    case 'ExpressionSub':
      throw new Error('Not implemented (ExpressionSub).');
  }
}

function translateDomainElement(
  domainElement: hrg.DomainElement,
  gameDeclaration: hrg.GameDeclaration,
): hrg.Value[] {
  switch (domainElement.kind) {
    case 'DomainGenerator':
      return domainElement.args
        .map(identifier => {
          const values = utils.find(domainElement.values, { identifier });
          utils.assert(values, `Missing values for "${identifier}".`);
          switch (values.kind) {
            case 'DomainRange': {
              const max = +values.max;
              const min = +values.min;
              return Array.from({ length: max - min + 1 }, (_, index) =>
                hrg.ValueElement({ identifier: `${index + min}` }),
              );
            }
            case 'DomainSet':
              return values.elements.map(identifier =>
                hrg.ValueElement({ identifier }),
              );
          }
        })
        .reduce<hrg.Value[][]>(utils.cartesian, [[]])
        .map(args =>
          hrg.ValueConstructor({ identifier: domainElement.identifier, args }),
        );
    case 'DomainLiteral': {
      const referencedDomain = gameDeclaration.domains.find(
        domainDeclaration =>
          domainDeclaration.identifier === domainElement.identifier,
      );
      return referencedDomain
        ? translateDomainElements(referencedDomain.elements, gameDeclaration)
        : [hrg.ValueElement({ identifier: domainElement.identifier })];
    }
  }
}

function translateDomainElements(
  domainElements: hrg.DomainElement[],
  gameDeclaration: hrg.GameDeclaration,
): hrg.Value[] {
  return domainElements.flatMap(domainElement =>
    translateDomainElement(domainElement, gameDeclaration),
  );
}

// eslint-disable-next-line complexity -- This function could be improved.
function translateExpression(expression: hrg.Expression): rg.Expression {
  switch (expression.kind) {
    case 'ExpressionAccess':
      return rg.Access({
        lhs: translateExpression(expression.lhs),
        rhs: translateExpression(expression.rhs),
      });
    case 'ExpressionAdd':
      throw new Error('Not implemented (ExpressionAdd).');
    case 'ExpressionAnd':
      throw new Error('Not implemented (ExpressionAnd).');
    case 'ExpressionCall':
      return expression.args.reduce<rg.Expression>(
        (expression, arg) =>
          rg.Access({ lhs: expression, rhs: translateExpression(arg) }),
        translateExpression(expression.expression),
      );
    case 'ExpressionConstructor':
      throw new Error('Not implemented (ExpressionConstructor).');
    case 'ExpressionEq':
      throw new Error('Not implemented (ExpressionEq).');
    case 'ExpressionGt':
      throw new Error('Not implemented (ExpressionGt).');
    case 'ExpressionGte':
      throw new Error('Not implemented (ExpressionGte).');
    case 'ExpressionIf':
      throw new Error('Not implemented (ExpressionIf).');
    case 'ExpressionLiteral':
      return rg.Reference({ identifier: expression.identifier });
    case 'ExpressionLt':
      throw new Error('Not implemented (ExpressionLt).');
    case 'ExpressionLte':
      throw new Error('Not implemented (ExpressionLte).');
    case 'ExpressionMap':
      throw new Error('Not implemented (ExpressionMap).');
    case 'ExpressionNe':
      throw new Error('Not implemented (ExpressionNe).');
    case 'ExpressionOr':
      throw new Error('Not implemented (ExpressionOr).');
    case 'ExpressionSub':
      throw new Error('Not implemented (ExpressionSub).');
  }
}

function translateFunctionDeclaration(
  functionDeclaration: hrg.FunctionDeclaration,
  typeValues: Record<string, hrg.Value[]>,
): rg.ConstantDeclaration {
  utils.assert(
    functionDeclaration.cases[0].args.length === 1,
    'Only simple functions are allowed.',
  );
  functionDeclaration.cases.forEach(functionCase => {
    utils.assert(
      functionDeclaration.identifier === functionCase.identifier,
      'All function cases should have the same identifier as function declaration.',
    );
    utils.assert(
      functionDeclaration.cases[0].args.length === functionCase.args.length,
      'All function cases should have the same number of arguments.',
    );
  });

  const type = translateType(functionDeclaration.type);
  utils.assert(
    type.kind === 'Arrow',
    'Function is expected to have Arrow type.',
  );
  utils.assert(
    type.lhs in typeValues,
    `Unresolved TypeReference "${type.lhs}".`,
  );
  utils.assert(
    typeValues[type.lhs].length,
    'Expected at least one identifier.',
  );
  const entries = typeValues[type.lhs].map(value => {
    utils.assert(value.kind !== 'ValueMap', 'ValueMap is not allowed.');
    const arm = utils.findMap(functionDeclaration.cases, functionCase => {
      utils.assert(functionCase.args.length === 1, 'Not implemented.');
      const pattern = functionCase.args[0];
      const binding = evaluateBinding(pattern, value);
      return binding ? { binding, functionCase } : undefined;
    });
    utils.assert(arm, `No FunctionCase found for "${value.identifier}".`);
    const { binding, functionCase } = arm;
    return rg.NamedEntry({
      identifier: serializeValue(value),
      value: rg.Element({
        identifier: serializeValue(
          evaluateExpression(functionCase.body, binding),
        ),
      }),
    });
  });

  return rg.ConstantDeclaration({
    identifier: functionDeclaration.identifier,
    type,
    value: constructMap(entries),
  });
}

function translateGameDeclaration(
  gameDeclaration: hrg.GameDeclaration,
): rg.GameDeclaration {
  const { types, typeValues } = gameDeclaration.domains.reduce(
    (result, domainDeclaration) => {
      const values = translateDomainElements(
        domainDeclaration.elements,
        gameDeclaration,
      );
      result.types.push(
        rg.TypeDeclaration({
          identifier: domainDeclaration.identifier,
          type: rg.Set({ identifiers: values.map(serializeValue) }),
        }),
      );
      utils.assert(
        !(domainDeclaration.identifier in result.typeValues),
        `Duplicated type "${domainDeclaration.identifier}".`,
      );
      result.typeValues[domainDeclaration.identifier] = values;
      return result;
    },
    {
      types: [] as rg.TypeDeclaration[],
      typeValues: {} as Record<string, hrg.Value[]>,
    },
  );
  const constants = gameDeclaration.functions.map(functionDeclaration =>
    translateFunctionDeclaration(functionDeclaration, typeValues),
  );
  const variables = gameDeclaration.variables.map(variableDeclaration =>
    translateVariableDeclaration(variableDeclaration, typeValues),
  );

  variables.push(
    translateVariableDeclaration(
      hrg.VariableDeclaration({
        identifier: 'player',
        type: hrg.TypeName({ identifier: 'Player' }),
        defaultValue: hrg.ExpressionLiteral({ identifier: 'keeper' }),
      }),
      typeValues,
    ),
  );
  variables.push(
    translateVariableDeclaration(
      hrg.VariableDeclaration({
        identifier: 'score',
        type: hrg.TypeFunction({
          lhs: hrg.TypeName({ identifier: 'Player' }),
          rhs: hrg.TypeName({ identifier: 'Score' }),
        }),
        defaultValue: null,
      }),
      typeValues,
    ),
  );

  const edges = [
    rg.EdgeDeclaration({
      label: rg.Skip({}),
      lhs: rg.EdgeName({ parts: [rg.Literal({ identifier: 'begin' })] }),
      rhs: rg.EdgeName({ parts: [rg.Literal({ identifier: 'rules_0' })] }),
    }),
  ];

  const rules = gameDeclaration.automaton.find(
    automatonFunction => automatonFunction.name === 'rules',
  );
  utils.assert(rules, 'No `rules` automation function found.');

  translateAutomatonFunction(
    rules,
    gameDeclaration.automaton,
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
    edges,
    variables,
    '',
  );

  return rg.GameDeclaration({ constants, edges, types, variables });
}

function translateType(type: hrg.Type): rg.Type {
  switch (type.kind) {
    case 'TypeFunction': {
      utils.assert(
        type.lhs.kind === 'TypeName',
        'Arrow lhs must be TypeReference.',
      );
      return rg.Arrow({
        lhs: type.lhs.identifier,
        rhs: translateType(type.rhs),
      });
    }
    case 'TypeName':
      return rg.TypeReference({ identifier: type.identifier });
  }
}

function translateValue(value: hrg.Value): rg.Value {
  switch (value.kind) {
    case 'ValueConstructor':
      return rg.Element({ identifier: serializeValue(value) });
    case 'ValueElement':
      return rg.Element({ identifier: value.identifier });
    case 'ValueMap':
      utils.assert(value.entries.length, 'At least one entry is required.');
      return constructMap(
        value.entries.map(entry =>
          rg.NamedEntry({
            identifier: serializeValue(entry.key),
            value: translateValue(entry.value),
          }),
        ),
      );
  }
}

function translateVariableDeclaration(
  variableDeclaration: hrg.VariableDeclaration,
  typeValues: Record<string, hrg.Value[]>,
): rg.VariableDeclaration {
  const type = translateType(variableDeclaration.type);
  return rg.VariableDeclaration({
    identifier: variableDeclaration.identifier,
    defaultValue: translateValue(
      variableDeclaration.defaultValue === null
        ? evaluateDefaultValue(type, typeValues)
        : evaluateExpression(variableDeclaration.defaultValue, {}),
    ),
    type,
  });
}

export default function translate(gameDeclaration: hrg.GameDeclaration) {
  RANDOM_RESET();
  return translateGameDeclaration(gameDeclaration);
}
