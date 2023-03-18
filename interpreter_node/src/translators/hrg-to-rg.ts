import { ast as hrg } from '../hrg';
import { ast as rg } from '../rg';
import { Settings } from '../types';
import * as utils from '../utils';

type Context = {
  $random: (prefix: string) => string;
  $randomEdgeName: (prefix: string) => rg.EdgeName;
  $randomLiteral: (prefix: string) => rg.Literal;
  $settings: Settings;
  hrg: hrg.GameDeclaration;
  rg: rg.GameDeclaration;
  translatedFunctions: Set<hrg.AutomatonFunction>;
  typeValues: Record<string, hrg.Value[]>;
};

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

function constructMap(entries: rg.ValueEntry[]) {
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
      rg.ValueEntry({ identifier: null, value: defaultValue }),
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
    expression.kind !== 'ExpressionAnd' &&
    expression.kind !== 'ExpressionEq' &&
    expression.kind !== 'ExpressionGt' &&
    expression.kind !== 'ExpressionGte' &&
    expression.kind !== 'ExpressionLt' &&
    expression.kind !== 'ExpressionLte' &&
    expression.kind !== 'ExpressionNe' &&
    expression.kind !== 'ExpressionOr'
  ) {
    throw new Error(
      `Expression "${expression.kind}" is not a valid condition.`,
    );
  }

  switch (expression.kind) {
    case 'ExpressionAnd':
      return (
        evaluateCondition(expression.lhs, binding) &&
        evaluateCondition(expression.rhs, binding)
      );
    case 'ExpressionOr':
      return (
        evaluateCondition(expression.lhs, binding) ||
        evaluateCondition(expression.rhs, binding)
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

function evaluateDefaultValue(context: Context, type: rg.Type): hrg.Value {
  switch (type.kind) {
    case 'Arrow':
      // NOTE: Is this even correct?
      return hrg.ValueMap({
        entries: evaluateTypeValues(context, type.lhs).map(value =>
          hrg.ValueMapEntry({
            key: value,
            value: evaluateDefaultValue(context, type.rhs),
          }),
        ),
      });
    case 'Set':
      throw new Error('Not implemented (Set).');
    case 'TypeReference':
      return evaluateTypeValues(context, type)[0];
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
      return hrg.ValueElement({
        identifier: String(Number(lhs) + Number(rhs)),
      });
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
      return hrg.ValueElement({
        identifier: String(Number(lhs) - Number(rhs)),
      });
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

function evaluateTypeValues(context: Context, type: rg.Type): hrg.Value[] {
  utils.assert(
    type.kind === 'TypeReference',
    `Expected TypeReference, got "${type.kind}".`,
  );
  utils.assert(
    type.identifier in context.typeValues,
    `Unresolved TypeReference "${type.identifier}".`,
  );
  utils.assert(
    context.typeValues[type.identifier].length,
    'Expected at least one identifier.',
  );
  return context.typeValues[type.identifier];
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
  context: Context,
  automatonFunction: hrg.AutomatonFunction,
  endEdgeName: rg.EdgeName | null,
  returnEdgeName: rg.EdgeName | null,
  prefix: string,
) {
  for (const arg of automatonFunction.args) {
    // Function arguments are hoisted into global variables, shadowing them if
    // needed. For the sake of easier implementation, the type of a global
    // variable has to match the type of the argument.
    const type = translateType(arg.type);
    const variable = context.rg.variables.find(
      ({ identifier }) => identifier === arg.identifier,
    );

    if (variable) {
      utils.assert(
        utils.isEqual(type, variable.type),
        `Argument "${arg.identifier}" of function "${
          automatonFunction.name
        }" has a different type than an already existing variable (${rg.serializeType(
          type,
        )} != ${rg.serializeType(variable.type)})`,
      );
    } else {
      context.rg.variables.push(
        rg.VariableDeclaration({
          identifier: arg.identifier,
          type,
          defaultValue: translateValue(evaluateDefaultValue(context, type)),
        }),
      );
    }
  }

  const nextEdgeName = rg.EdgeName({
    parts: [
      rg.Literal({
        identifier: context.$settings.flags.reuseFunctions
          ? `${automatonFunction.name}_end`
          : `${prefix}${automatonFunction.name}_end`,
      }),
    ],
  });

  const returns = translateAutomatonStatements(context, {
    automatonStatements: automatonFunction.body,
    breakEdgeName: null,
    continueEdgeName: null,
    endEdgeName,
    entryEdgeName: rg.EdgeName({
      parts: [
        rg.Literal({
          identifier: context.$settings.flags.reuseFunctions
            ? `${automatonFunction.name}_begin`
            : `${prefix}${automatonFunction.name}_begin`,
        }),
      ],
    }),
    nextEdgeName,
    prefix: `${prefix}${automatonFunction.name}`,
    returnEdgeName: nextEdgeName,
  });

  if (returns && returnEdgeName) {
    context.rg.edges.push(
      rg.EdgeDeclaration({
        lhs: nextEdgeName,
        rhs: returnEdgeName,
        label: rg.Skip({}),
      }),
    );
  }
}

// eslint-disable-next-line complexity -- It could use some refactor.
function translateAutomatonStatements(
  context: Context,
  {
    automatonStatements,
    breakEdgeName,
    continueEdgeName,
    endEdgeName,
    entryEdgeName,
    nextEdgeName,
    prefix,
    returnEdgeName,
  }: {
    automatonStatements: hrg.AutomatonStatement[];
    breakEdgeName: rg.EdgeName | null;
    continueEdgeName: rg.EdgeName | null;
    endEdgeName: rg.EdgeName | null;
    entryEdgeName: rg.EdgeName;
    nextEdgeName: rg.EdgeName | null;
    prefix: string;
    returnEdgeName: rg.EdgeName | null;
  },
) {
  let currentEdgeName = entryEdgeName;
  for (const automatonStatement of automatonStatements) {
    switch (automatonStatement.kind) {
      case 'AutomatonAssignment': {
        const localEdgeName = context.$randomEdgeName(prefix);
        context.rg.edges.push(
          rg.EdgeDeclaration({
            lhs: currentEdgeName,
            rhs: localEdgeName,
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
        currentEdgeName = localEdgeName;
        continue;
      }
      case 'AutomatonBranch': {
        const localEdgeName = context.$randomEdgeName(prefix);
        automatonStatement.arms.forEach(arm => {
          translateAutomatonStatements(context, {
            automatonStatements: arm,
            breakEdgeName,
            continueEdgeName,
            endEdgeName,
            entryEdgeName: currentEdgeName,
            nextEdgeName: localEdgeName,
            prefix,
            returnEdgeName,
          });
        });
        currentEdgeName = localEdgeName;
        continue;
      }
      case 'AutomatonCall':
        switch (automatonStatement.identifier) {
          case 'assert': {
            utils.assert(
              automatonStatement.args.length === 1,
              'assert() expects 1 argument',
            );
            const localEdgeName = context.$randomEdgeName(prefix);
            translateCondition(
              context,
              automatonStatement.args[0],
              currentEdgeName,
              localEdgeName,
              null,
              prefix,
            );
            currentEdgeName = localEdgeName;
            continue;
          }
          case 'break': {
            utils.assert(
              automatonStatements.indexOf(automatonStatement) ===
                automatonStatements.length - 1,
              'break() has to be the last statement.',
            );
            utils.assert(breakEdgeName, 'break() requires breakEdgeName.');
            context.rg.edges.push(
              rg.EdgeDeclaration({
                lhs: currentEdgeName,
                rhs: breakEdgeName,
                label: rg.Skip({}),
              }),
            );
            return true;
          }
          case 'continue': {
            utils.assert(
              automatonStatements.indexOf(automatonStatement) ===
                automatonStatements.length - 1,
              'continue() has to be the last statement.',
            );
            utils.assert(
              continueEdgeName,
              'continue() requires continueEdgeName.',
            );
            context.rg.edges.push(
              rg.EdgeDeclaration({
                lhs: currentEdgeName,
                rhs: continueEdgeName,
                label: rg.Skip({}),
              }),
            );
            return true;
          }
          case 'end': {
            utils.assert(
              automatonStatements.indexOf(automatonStatement) ===
                automatonStatements.length - 1,
              'end() has to be the last statement.',
            );
            utils.assert(endEdgeName, 'end() requires endEdgeName.');
            context.rg.edges.push(
              rg.EdgeDeclaration({
                lhs: currentEdgeName,
                rhs: endEdgeName,
                label: rg.Assignment({
                  lhs: rg.Reference({ identifier: 'player' }),
                  rhs: rg.Reference({ identifier: 'keeper' }),
                }),
              }),
            );
            return true;
          }
          case 'forall': {
            utils.assert(
              automatonStatement.args.length === 1,
              'forall() expects 1 argument',
            );
            const assignmentVariable = automatonStatement.args[0];
            utils.assert(
              assignmentVariable.kind === 'ExpressionLiteral',
              'forall() expects a literal',
            );
            const assignmentVariableDeclaration = context.rg.variables.find(
              variableDeclaration =>
                variableDeclaration.identifier ===
                assignmentVariable.identifier,
            );
            utils.assert(
              assignmentVariableDeclaration,
              `Unknown variable "${assignmentVariable.identifier}" in forall().`,
            );
            const identifier = context.$random('bind');
            const assignmentEdgeName = rg.EdgeName({
              parts: [
                context.$randomLiteral(prefix),
                rg.Binding({
                  identifier,
                  type: assignmentVariableDeclaration.type,
                }),
              ],
            });
            context.rg.edges.push(
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
              'return() has to be the last statement.',
            );
            utils.assert(returnEdgeName, 'return() requires returnEdgeName.');
            context.rg.edges.push(
              rg.EdgeDeclaration({
                lhs: currentEdgeName,
                rhs: returnEdgeName,
                label: rg.Skip({}),
              }),
            );
            return true;
          }
          default: {
            const automatonFunction = context.hrg.automaton.find(
              automatonFunction =>
                automatonFunction.name === automatonStatement.identifier,
            );
            utils.assert(
              automatonFunction,
              `Unknown automaton function "${automatonStatement.identifier}".`,
            );
            if (context.$settings.flags.reuseFunctions) {
              const callId = context.$random(`${automatonFunction.name}_call`);
              const variable = `${automatonFunction.name}_return`;
              if (
                context.rg.variables.some(
                  ({ identifier }) => identifier === variable,
                )
              ) {
                const typeDeclaration = context.rg.types.find(
                  ({ identifier }) => identifier === variable,
                );
                utils.assert(typeDeclaration, `Type "${variable}" not found.`);
                utils.assert(
                  typeDeclaration.type.kind === 'Set',
                  `Type "${variable}" has invalid type.`,
                );
                typeDeclaration.type.identifiers.push(callId);
              } else {
                context.rg.types.push(
                  rg.TypeDeclaration({
                    identifier: variable,
                    type: rg.Set({ identifiers: [callId] }),
                  }),
                );
                context.rg.variables.push(
                  rg.VariableDeclaration({
                    identifier: variable,
                    type: rg.TypeReference({ identifier: variable }),
                    defaultValue: rg.Element({ identifier: callId }),
                  }),
                );
              }

              const callEdgeName = rg.EdgeName({
                parts: [rg.Literal({ identifier: callId })],
              });
              context.rg.edges.push(
                rg.EdgeDeclaration({
                  lhs: currentEdgeName,
                  rhs: callEdgeName,
                  label: rg.Skip({}),
                }),
              );
              const setEdgeName = context.$randomEdgeName(prefix);
              context.rg.edges.push(
                rg.EdgeDeclaration({
                  lhs: callEdgeName,
                  rhs: setEdgeName,
                  label: rg.Assignment({
                    lhs: rg.Reference({ identifier: variable }),
                    rhs: rg.Reference({ identifier: callId }),
                  }),
                }),
              );
              currentEdgeName = setEdgeName;
              for (const arg of automatonStatement.args) {
                const argEdgeName = context.$randomEdgeName(prefix);
                context.rg.edges.push(
                  rg.EdgeDeclaration({
                    lhs: currentEdgeName,
                    rhs: argEdgeName,
                    label: rg.Assignment({
                      lhs: rg.Reference({
                        identifier:
                          automatonFunction.args[
                            automatonStatement.args.indexOf(arg)
                          ].identifier,
                      }),
                      rhs: translateExpression(arg),
                    }),
                  }),
                );
                currentEdgeName = argEdgeName;
              }
              context.rg.edges.push(
                rg.EdgeDeclaration({
                  lhs: currentEdgeName,
                  rhs: rg.EdgeName({
                    parts: [
                      rg.Literal({
                        identifier: `${automatonFunction.name}_begin`,
                      }),
                    ],
                  }),
                  label: rg.Skip({}),
                }),
              );
              const localEdgeName = rg.EdgeName({
                parts: [
                  rg.Literal({
                    identifier: context.$random(
                      `${automatonFunction.name}_return`,
                    ),
                  }),
                ],
              });
              if (context.translatedFunctions.has(automatonFunction)) {
                context.rg.edges.push(
                  rg.EdgeDeclaration({
                    lhs: rg.EdgeName({
                      parts: [
                        rg.Literal({
                          identifier: `${automatonFunction.name}_end`,
                        }),
                      ],
                    }),
                    rhs: localEdgeName,
                    label: rg.Skip({}),
                  }),
                );
              } else {
                context.translatedFunctions.add(automatonFunction);
                translateAutomatonFunction(
                  context,
                  automatonFunction,
                  endEdgeName,
                  localEdgeName,
                  '',
                );
              }
              currentEdgeName = localEdgeName;
              const x = context.$randomEdgeName(prefix);
              context.rg.edges.push(
                rg.EdgeDeclaration({
                  lhs: currentEdgeName,
                  rhs: x,
                  label: rg.Comparison({
                    lhs: rg.Reference({ identifier: variable }),
                    rhs: rg.Reference({ identifier: callId }),
                    negated: false,
                  }),
                }),
              );
              currentEdgeName = x;
            } else {
              const callId = `${context.$random(prefix)}_`;
              for (const arg of automatonStatement.args) {
                const argEdgeName = context.$randomEdgeName(callId);
                context.rg.edges.push(
                  rg.EdgeDeclaration({
                    lhs: currentEdgeName,
                    rhs: argEdgeName,
                    label: rg.Assignment({
                      lhs: rg.Reference({
                        identifier:
                          automatonFunction.args[
                            automatonStatement.args.indexOf(arg)
                          ].identifier,
                      }),
                      rhs: translateExpression(arg),
                    }),
                  }),
                );
                currentEdgeName = argEdgeName;
              }
              context.rg.edges.push(
                rg.EdgeDeclaration({
                  lhs: currentEdgeName,
                  rhs: rg.EdgeName({
                    parts: [
                      rg.Literal({
                        identifier: `${callId}${automatonFunction.name}_begin`,
                      }),
                    ],
                  }),
                  label: rg.Skip({}),
                }),
              );
              const localEdgeName = context.$randomEdgeName(prefix);
              translateAutomatonFunction(
                context,
                automatonFunction,
                endEdgeName,
                localEdgeName,
                callId,
              );
              currentEdgeName = localEdgeName;
            }
            continue;
          }
        }
      case 'AutomatonLoop': {
        const localEdgeName = context.$randomEdgeName(prefix);
        translateAutomatonStatements(context, {
          automatonStatements: automatonStatement.body,
          breakEdgeName: localEdgeName,
          continueEdgeName: currentEdgeName,
          endEdgeName,
          entryEdgeName: currentEdgeName,
          nextEdgeName: currentEdgeName,
          prefix,
          returnEdgeName,
        });
        currentEdgeName = localEdgeName;
        continue;
      }
      case 'AutomatonWhen': {
        const thenEdgeName = context.$randomEdgeName(prefix);
        const elseEdgeName = context.$randomEdgeName(prefix);
        translateCondition(
          context,
          automatonStatement.expression,
          currentEdgeName,
          thenEdgeName,
          elseEdgeName,
          prefix,
        );
        translateAutomatonStatements(context, {
          automatonStatements: automatonStatement.body,
          breakEdgeName,
          continueEdgeName,
          endEdgeName,
          entryEdgeName: thenEdgeName,
          nextEdgeName,
          prefix,
          returnEdgeName: elseEdgeName,
        });
        currentEdgeName = elseEdgeName;
        continue;
      }
      case 'AutomatonWhile':
        throw new Error('Not implemented (AutomatonWhile).');
    }
  }

  if (nextEdgeName) {
    context.rg.edges.push(
      rg.EdgeDeclaration({
        lhs: currentEdgeName,
        rhs: nextEdgeName,
        label: rg.Skip({}),
      }),
    );
  }

  return true;
}

// eslint-disable-next-line complexity -- This function could be improved.
function translateCondition(
  context: Context,
  expression: hrg.Expression,
  entryEdgeName: rg.EdgeName,
  thenEdgeName: rg.EdgeName | null,
  elseEdgeName: rg.EdgeName | null,
  prefix: string,
) {
  switch (expression.kind) {
    case 'ExpressionAccess':
      throw new Error('Not implemented (ExpressionAccess).');
    case 'ExpressionAdd':
      throw new Error('Not implemented (ExpressionAdd).');
    case 'ExpressionAnd': {
      const trueEdgeName = context.$randomEdgeName(prefix);
      translateCondition(
        context,
        expression.lhs,
        entryEdgeName,
        trueEdgeName,
        elseEdgeName,
        prefix,
      );
      translateCondition(
        context,
        expression.rhs,
        trueEdgeName,
        thenEdgeName,
        elseEdgeName,
        prefix,
      );
      return;
    }
    case 'ExpressionCall': {
      utils.assert(
        expression.expression.kind === 'ExpressionLiteral',
        'Call expects a literal',
      );
      switch (expression.expression.identifier) {
        case 'not':
          utils.assert(expression.args.length === 1, 'not expects 1 argument');
          translateCondition(
            context,
            expression.args[0],
            entryEdgeName,
            elseEdgeName,
            thenEdgeName,
            prefix,
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
          const automatonPrefix = context.$random(prefix);
          const automatonFunction = context.hrg.automaton.find(
            automatonFunction => automatonFunction.name === automatonName,
          );
          utils.assert(
            automatonFunction,
            `Unknown automaton function "${call.expression.identifier}".`,
          );

          const callId = context.$random(`${automatonFunction.name}_call`);
          const automatonStartEdgeName = context.$settings.flags.reuseFunctions
            ? rg.EdgeName({ parts: [rg.Literal({ identifier: callId })] })
            : context.$randomEdgeName(automatonPrefix);
          let automatonCurrentEdgeName = automatonStartEdgeName;

          const variable = `${automatonFunction.name}_return`;
          if (context.$settings.flags.reuseFunctions) {
            if (
              context.rg.variables.some(
                ({ identifier }) => identifier === variable,
              )
            ) {
              const typeDeclaration = context.rg.types.find(
                ({ identifier }) => identifier === variable,
              );
              utils.assert(typeDeclaration, `Type "${variable}" not found.`);
              utils.assert(
                typeDeclaration.type.kind === 'Set',
                `Type "${variable}" has invalid type.`,
              );
              typeDeclaration.type.identifiers.push(callId);
            } else {
              context.rg.types.push(
                rg.TypeDeclaration({
                  identifier: variable,
                  type: rg.Set({ identifiers: [callId] }),
                }),
              );
              context.rg.variables.push(
                rg.VariableDeclaration({
                  identifier: variable,
                  type: rg.TypeReference({ identifier: variable }),
                  defaultValue: rg.Element({ identifier: callId }),
                }),
              );
            }
          }

          if (context.$settings.flags.reuseFunctions) {
            const callEdgeName = context.$randomEdgeName(automatonPrefix);
            context.rg.edges.push(
              rg.EdgeDeclaration({
                lhs: automatonCurrentEdgeName,
                rhs: callEdgeName,
                label: rg.Assignment({
                  lhs: rg.Reference({ identifier: variable }),
                  rhs: rg.Reference({ identifier: callId }),
                }),
              }),
            );
            automatonCurrentEdgeName = callEdgeName;
          }

          for (const arg of call.args) {
            const argEdgeName = context.$randomEdgeName(automatonPrefix);
            context.rg.edges.push(
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

          context.rg.edges.push(
            rg.EdgeDeclaration({
              lhs: automatonCurrentEdgeName,
              rhs: rg.EdgeName({
                parts: [
                  rg.Literal({
                    identifier: context.$settings.flags.reuseFunctions
                      ? `${automatonName}_begin`
                      : `${automatonPrefix}_${automatonName}_begin`,
                  }),
                ],
              }),
              label: rg.Skip({}),
            }),
          );

          let automatonEndEdgeName = rg.EdgeName({
            parts: [
              rg.Literal({
                identifier: context.$settings.flags.reuseFunctions
                  ? `${automatonName}_end`
                  : `${automatonPrefix}_${automatonName}_end`,
              }),
            ],
          });

          if (context.$settings.flags.reuseFunctions) {
            if (!context.translatedFunctions.has(automatonFunction)) {
              context.translatedFunctions.add(automatonFunction);
              translateAutomatonFunction(
                context,
                automatonFunction,
                automatonEndEdgeName,
                null,
                '',
              );
            }
          } else {
            translateAutomatonFunction(
              context,
              automatonFunction,
              automatonEndEdgeName,
              null,
              `${automatonPrefix}_`,
            );
          }

          if (context.$settings.flags.reuseFunctions) {
            const callEdgeName = rg.EdgeName({
              parts: [
                rg.Literal({
                  identifier: context.$random(
                    `${automatonFunction.name}_return`,
                  ),
                }),
              ],
            });
            context.rg.edges.push(
              rg.EdgeDeclaration({
                lhs: automatonEndEdgeName,
                rhs: callEdgeName,
                label: rg.Comparison({
                  lhs: rg.Reference({ identifier: variable }),
                  rhs: rg.Reference({ identifier: callId }),
                  negated: false,
                }),
              }),
            );
            automatonEndEdgeName = callEdgeName;
          }

          if (thenEdgeName) {
            context.rg.edges.push(
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
            context.rg.edges.push(
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
        context.rg.edges.push(
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
        context.rg.edges.push(
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
        context.rg.edges.push(
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
        context.rg.edges.push(
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
      const falseEdgeName = context.$randomEdgeName(prefix);
      translateCondition(
        context,
        expression.lhs,
        entryEdgeName,
        thenEdgeName,
        falseEdgeName,
        prefix,
      );
      translateCondition(
        context,
        expression.rhs,
        falseEdgeName,
        thenEdgeName,
        elseEdgeName,
        prefix,
      );
      return;
    }
    case 'ExpressionSub':
      throw new Error('Not implemented (ExpressionSub).');
  }
}

function translateDomainElement(
  context: Context,
  domainElement: hrg.DomainElement,
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
      const referencedDomain = context.hrg.domains.find(
        domainDeclaration =>
          domainDeclaration.identifier === domainElement.identifier,
      );
      return referencedDomain
        ? translateDomainElements(context, referencedDomain.elements)
        : [hrg.ValueElement({ identifier: domainElement.identifier })];
    }
  }
}

function translateDomainElements(
  context: Context,
  domainElements: hrg.DomainElement[],
): hrg.Value[] {
  return domainElements.flatMap(domainElement =>
    translateDomainElement(context, domainElement),
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
  context: Context,
  functionDeclaration: hrg.FunctionDeclaration,
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
  const entries = evaluateTypeValues(context, type.lhs).map(value => {
    utils.assert(value.kind !== 'ValueMap', 'ValueMap is not allowed.');
    const arm = utils.findMap(functionDeclaration.cases, functionCase => {
      utils.assert(functionCase.args.length === 1, 'Not implemented.');
      const pattern = functionCase.args[0];
      const binding = evaluateBinding(pattern, value);
      return binding ? { binding, functionCase } : undefined;
    });
    utils.assert(arm, `No FunctionCase found for "${value.identifier}".`);
    const { binding, functionCase } = arm;
    return rg.ValueEntry({
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

function translateGameDeclaration(context: Context): rg.GameDeclaration {
  for (const { elements, identifier } of context.hrg.domains) {
    utils.assert(
      !(identifier in context.typeValues),
      `Duplicated type "${identifier}".`,
    );

    context.typeValues[identifier] = translateDomainElements(context, elements);
    context.rg.types.push(
      rg.TypeDeclaration({
        identifier,
        type: rg.Set({
          identifiers: context.typeValues[identifier].map(serializeValue),
        }),
      }),
    );
  }

  context.rg.constants = context.hrg.functions.map(functionDeclaration =>
    translateFunctionDeclaration(context, functionDeclaration),
  );

  context.rg.variables = context.hrg.variables.map(variableDeclaration =>
    translateVariableDeclaration(context, variableDeclaration),
  );

  context.rg.edges = [
    rg.EdgeDeclaration({
      lhs: rg.EdgeName({ parts: [rg.Literal({ identifier: 'begin' })] }),
      rhs: rg.EdgeName({ parts: [rg.Literal({ identifier: 'rules_begin' })] }),
      label: rg.Skip({}),
    }),
  ];

  const rules = context.hrg.automaton.find(
    automatonFunction => automatonFunction.name === 'rules',
  );
  utils.assert(rules, 'No `rules` automation function found.');

  translateAutomatonFunction(
    context,
    rules,
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
    rg.EdgeName({ parts: [rg.Literal({ identifier: 'end' })] }),
    '',
  );

  return context.rg;
}

function translateType(type: hrg.Type): rg.Type {
  switch (type.kind) {
    case 'TypeFunction':
      return rg.Arrow({
        lhs: translateType(type.lhs),
        rhs: translateType(type.rhs),
      });
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
          rg.ValueEntry({
            identifier: serializeValue(entry.key),
            value: translateValue(entry.value),
          }),
        ),
      );
  }
}

function translateVariableDeclaration(
  context: Context,
  variableDeclaration: hrg.VariableDeclaration,
): rg.VariableDeclaration {
  const type = translateType(variableDeclaration.type);
  return rg.VariableDeclaration({
    identifier: variableDeclaration.identifier,
    defaultValue: translateValue(
      variableDeclaration.defaultValue === null
        ? evaluateDefaultValue(context, type)
        : evaluateExpression(variableDeclaration.defaultValue, {}),
    ),
    type,
  });
}

export default function translate(
  hrg: hrg.GameDeclaration,
  settings: Settings,
) {
  const counters: Record<string, number> = Object.create(null);
  return translateGameDeclaration({
    $random(prefix: string) {
      if (!(prefix in counters)) {
        counters[prefix] = 0;
      }

      return `${prefix}_${++counters[prefix]}`;
    },
    $randomEdgeName(prefix: string) {
      return rg.EdgeName({ parts: [this.$randomLiteral(prefix)] });
    },
    $randomLiteral(prefix: string) {
      return rg.Literal({ identifier: this.$random(prefix) });
    },
    $settings: settings,
    hrg,
    rg: rg.GameDeclaration({
      constants: [],
      edges: [],
      pragmas: [],
      types: [],
      variables: [],
    }),
    translatedFunctions: new Set(),
    typeValues: Object.create(null),
  });
}
