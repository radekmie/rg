import lexer from './lexer';
import parser from './parser';
import * as hl from './types';
import visitor from './visitor';
import * as ll from '../ast/types';
import * as utils from '../utils';

enum Ord { Eq, Gt, Lt }
function numberToOrd(number: number) {
  return number === 0 ? Ord.Eq : number < 0 ? Ord.Lt : Ord.Gt;
}

function compareValues(lhs: hl.Value, rhs: hl.Value): Ord {
  switch (lhs.kind) {
    case 'ValueConstructor':
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
    case 'ValueElement': {
      utils.assert(rhs.kind === 'ValueElement', 'Incomparable values.');
      return numberToOrd(lhs.identifier.localeCompare(rhs.identifier));
    }
    case 'ValueMap':
      throw new Error('Not implemented.');
  }
}

function evaluateBinding(pattern: hl.Pattern, value: hl.Value): Record<string, hl.Value> | undefined {
  switch (pattern.kind) {
    case 'PatternConstructor':
      return (
        value.kind === 'ValueConstructor' &&
        value.identifier === pattern.identifier &&
        value.args.length === pattern.args.length
        ? pattern.args.reduce<Record<string, hl.Value> | undefined>((binding, pattern, index) => {
            if (binding) {
              const subbinding = evaluateBinding(pattern, value.args[index]);
              if (subbinding) return Object.assign(binding, subbinding);
            }
          }, {})
        : undefined
      );
    case 'PatternLiteral':
      return value.kind === 'ValueElement' && value.identifier === pattern.identifier ? {} : undefined;
    case 'PatternVariable':
      return { [pattern.identifier]: value };
    case 'PatternWildcard':
      return {};
  }
}

function evaluateCondition(expression: hl.Expression, binding: Record<string, hl.Value>): boolean {
  if (
    expression.kind !== 'ExpressionGt' &&
    expression.kind !== 'ExpressionGte' &&
    expression.kind !== 'ExpressionEq' &&
    expression.kind !== 'ExpressionLt' &&
    expression.kind !== 'ExpressionLte' &&
    expression.kind !== 'ExpressionNe'
  ) {
    throw new Error(`Expression "${expression.kind}" is not a valid condition.`);
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

function evaluateDefaultValue(type: ll.Type, typeValues: Record<string, hl.Value[]>): hl.Value {
  switch (type.kind) {
    case 'Arrow':
      throw new Error('Not implemented.');
    case 'Set':
      throw new Error('Not implemented.');
    case 'TypeReference':
      utils.assert(type.identifier in typeValues, `Unresolved TypeReference "${type.identifier}".`);
      utils.assert(typeValues[type.identifier].length, 'Expected at least one identifier.');
      return typeValues[type.identifier][0];
  }
}

function evaluateDomainValues(domainValues: hl.DomainValue[]): Record<string, hl.Value>[] {
  domainValues.slice(1).forEach(domainValue => {
    utils.assert(domainValues[0].identifier !== domainValue.identifier, 'Duplicated identifier.');
  });

  return domainValues
    .map(domainValue => {
      switch (domainValue.kind) {
        case 'DomainRange': {
          const max = +domainValue.max;
          const min = +domainValue.min;
          return Array.from(
            { length: max - min + 1 },
            (_, index) => ({ [domainValue.identifier]: hl.ValueElement({ identifier: `${index + min}` }) }),
          );
        }
        case 'DomainSet':
          return domainValue.elements.map(identifier => ({ [domainValue.identifier]: hl.ValueElement({ identifier }) }));
      }
    })
    .reduce<Record<string, hl.Value>[][]>(utils.cartesian, [[]])
    .map(bindings => bindings.reduce((a, b) => Object.assign(a, b), {}));
}

function evaluateExpression(expression: hl.Expression, binding: Record<string, hl.Value>): hl.Value {
  switch (expression.kind) {
    case 'ExpressionAccess':
      throw new Error('Not implemented (ExpressionAccess).');
    case 'ExpressionAdd': {
      const lhs = evaluateExpressionIdentifier(expression.lhs, binding);
      const rhs = evaluateExpressionIdentifier(expression.rhs, binding);
      return hl.ValueElement({ identifier: String(Number(lhs) + Number(rhs)) });
    }
    case 'ExpressionAnd':
      throw new Error('Not implemented (ExpressionAnd).');
    case 'ExpressionCall':
      throw new Error('Not implemented (ExpressionCall).');
    case 'ExpressionConstructor':
      return hl.ValueConstructor({
        identifier: expression.identifier,
        args: expression.args.map(expression => evaluateExpression(expression, binding)),
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
        : hl.ValueElement({ identifier: expression.identifier });
    case 'ExpressionLt':
      throw new Error('Not implemented (ExpressionLt).');
    case 'ExpressionLte':
      throw new Error('Not implemented (ExpressionLte).');
    case 'ExpressionOr':
      throw new Error('Not implemented (ExpressionOr).');
    case 'ExpressionMap':
      return hl.ValueMap({
        entries: evaluateDomainValues(expression.domains)
          .map(subbinding => Object.assign({}, binding, subbinding))
          .map(binding => hl.ValueMapEntry({
            key: evaluatePattern(expression.pattern, binding),
            value: evaluateExpression(expression.expression, binding),
          })),
      });
    case 'ExpressionNe':
      throw new Error('Not implemented (ExpressionNe).');
    case 'ExpressionSub': {
      const lhs = evaluateExpressionIdentifier(expression.lhs, binding);
      const rhs = evaluateExpressionIdentifier(expression.rhs, binding);
      return hl.ValueElement({ identifier: String(Number(lhs) - Number(rhs)) });
    }
  }
}

function evaluateExpressionIdentifier(expression: hl.Expression, binding: Record<string, hl.Value>): string {
  const value = evaluateExpression(expression, binding);
  utils.assert(value.kind === 'ValueElement', 'Expected ValueElement.');
  return value.identifier;
}

function evaluatePattern(pattern: hl.Pattern, binding: Record<string, hl.Value>): hl.Value {
  switch (pattern.kind) {
    case 'PatternConstructor':
      return hl.ValueConstructor({ identifier: pattern.identifier, args: pattern.args.map(pattern => evaluatePattern(pattern, binding)) });
    case 'PatternLiteral':
      return hl.ValueElement({ identifier: pattern.identifier });
    case 'PatternVariable':
      utils.assert(pattern.identifier in binding, `Unknown variable "${pattern.identifier}".`);
      return binding[pattern.identifier];
    case 'PatternWildcard':
      throw new Error('PatternWildcard is not evaluable.');
  }
}

function serializeValue(value: hl.Value): string {
  switch (value.kind) {
    case 'ValueConstructor':
      return `${value.identifier}__${value.args.map(serializeValue).join('_')}`;
    case 'ValueElement':
      return value.identifier;
    case 'ValueMap':
      throw new Error('Not implemented.');
  }
}

function translateAutomatonFunction(
  automatonFunction: hl.AutomatonFunction,
  automatonFunctions: hl.AutomatonFunction[],
  returnEdgeName: ll.EdgeName | null,
  edges: ll.EdgeDeclaration[],
  variableDeclarations: ll.VariableDeclaration[],
) {
  // NOTES:
  //   - The entrypoint of a function is at `automatonFunction.name`.
  //   - A function should receive an `EdgeLabel` to where it should go once
  //     it's finished. It's required for a situation when a function is NOT the
  //     last statement of a block, i.e., a `while` statement.
  //
  // TODO:
  //   - How to work around non-trivial arguments, e.g., `opponnent(me)`? We
  //     don't have a concept of registers so it'd have to be calculated before
  //     the actual call happens.
  //   - How to work around multiple return points? We cannot store an edge
  //     label in a variable, so we'd need multiple subgraphs of function for
  //     each of its callsites, but that's potentially expensive. (And we have
  //     to calculate them beforehand.)
  translateAutomatonStatements(
    automatonFunction.body,
    automatonFunctions,
    // TODO: Arguments!
    ll.EdgeName({ parts: [ll.Literal({ identifier: `${automatonFunction.name}_0` })] }),
    returnEdgeName,
    edges,
    variableDeclarations,
    automatonFunction.name,
  );
}

let globalCounter = 0;
function RANDOM(prefix: string) {
  return `${prefix}_${++globalCounter}`;
}

function translateAutomatonStatements(
  automatonStatements: hl.AutomatonStatement[],
  automatonFunctions: hl.AutomatonFunction[],
  entryEdgeName: ll.EdgeName,
  returnEdgeName: ll.EdgeName | null,
  edges: ll.EdgeDeclaration[],
  variableDeclarations: ll.VariableDeclaration[],
  prefix: string,
) {
  let currentEdgeName = entryEdgeName;
  for (const automatonStatement of automatonStatements) {
    switch (automatonStatement.kind) {
      case 'AutomatonAssignment': {
        const nextEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
        edges.push(ll.EdgeDeclaration({
          lhs: currentEdgeName,
          rhs: nextEdgeName,
          label: ll.Assignment({
            lhs: automatonStatement.accessors.reduce<ll.Expression>(
              (expression, accessor) => ll.Access({
                lhs: expression,
                rhs: translateExpression(accessor),
              }),
              ll.Reference({ identifier: automatonStatement.identifier }),
            ),
            rhs: translateExpression(automatonStatement.expression),
          }),
        }));
        currentEdgeName = nextEdgeName;
        continue;
      }
      case 'AutomatonBranch': {
        const nextEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
        automatonStatement.arms.forEach(arm => {
          translateAutomatonStatements(
            arm,
            automatonFunctions,
            currentEdgeName,
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
            utils.assert(automatonStatement.args.length === 1, 'assert expects 1 argument');
            const nextEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
            translateCondition(
              automatonStatement.args[0],
              currentEdgeName,
              nextEdgeName,
              null,
              edges,
              prefix,
            );
            currentEdgeName = nextEdgeName;
            continue;
          }
          case 'forall': {
            utils.assert(automatonStatement.args.length === 1, 'forall expects 1 argument');
            const assignmentVariable = automatonStatement.args[0];
            utils.assert(assignmentVariable.kind === 'ExpressionLiteral', 'forall expects a literal');
            const assignmentVariableDeclaration = variableDeclarations.find(variableDeclaration => variableDeclaration.identifier === assignmentVariable.identifier);
            utils.assert(assignmentVariableDeclaration, `Unknown variable "${assignmentVariable.identifier}" in forall.`);
            const assignmentIterator = RANDOM(prefix);
            const assignmentEdgeName = ll.EdgeName({
              parts: [
                ll.Literal({ identifier: assignmentIterator }),
                ll.Binding({ identifier: 'x', type: assignmentVariableDeclaration.type }),
              ],
            });
            edges.push(ll.EdgeDeclaration({
              lhs: currentEdgeName,
              rhs: assignmentEdgeName,
              label: ll.Assignment({
                lhs: ll.Reference({ identifier: assignmentVariable.identifier }),
                rhs: ll.Cast({ lhs: assignmentVariableDeclaration.type, rhs: ll.Reference({ identifier: 'x' }) }),
              }),
            }));
            const nextEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
            edges.push(ll.EdgeDeclaration({
              lhs: assignmentEdgeName,
              rhs: nextEdgeName,
              label: ll.Skip({}),
            }));
            currentEdgeName = nextEdgeName;
            continue;
          }
          case 'return': {
            utils.assert(automatonStatements.indexOf(automatonStatement) === automatonStatements.length - 1, 'Return has to be the last statement.');
            utils.assert(returnEdgeName, 'Return requires returnEdgeName.');
            edges.push(ll.EdgeDeclaration({
              lhs: currentEdgeName,
              rhs: returnEdgeName,
              label: ll.Skip({}),
            }));
            return;
          }
          default: {
            const automatonFunction = automatonFunctions.find(automatonFunction => automatonFunction.name === automatonStatement.identifier);
            utils.assert(automatonFunction, `Unknown automaton function "${automatonStatement.identifier}".`);
            edges.push(ll.EdgeDeclaration({
              lhs: currentEdgeName,
              rhs: ll.EdgeName({ parts: [ll.Literal({ identifier: `${automatonFunction.name}_0` })] }),
              label: ll.Skip({}),
            }));
            const nextEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
            translateAutomatonFunction(
              automatonFunction,
              automatonFunctions,
              nextEdgeName,
              edges,
              variableDeclarations,
            );
            currentEdgeName = nextEdgeName;
            continue;
          }
        }
      case 'AutomatonLoop':
        utils.assert(automatonStatements.indexOf(automatonStatement) === automatonStatements.length - 1, 'Loop has to be the last statement.');
        translateAutomatonStatements(
          automatonStatement.body,
          automatonFunctions,
          entryEdgeName,
          currentEdgeName,
          edges,
          variableDeclarations,
          prefix,
        );
        return;
      case 'AutomatonWhen':
        const thenEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
        const elseEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
        translateCondition(
          automatonStatement.expression,
          currentEdgeName,
          thenEdgeName,
          elseEdgeName,
          edges,
          prefix,
        );
        translateAutomatonStatements(
          automatonStatement.body,
          automatonFunctions,
          thenEdgeName,
          elseEdgeName,
          edges,
          variableDeclarations,
          prefix,
        );
        edges.push(ll.EdgeDeclaration({
          lhs: thenEdgeName,
          rhs: elseEdgeName,
          label: ll.Skip({}),
        }));
        currentEdgeName = elseEdgeName;
        continue;
      case 'AutomatonWhile':
        throw new Error('Not implemented (AutomatonWhile).');
    }
  }

  if (returnEdgeName) {
    edges.push(ll.EdgeDeclaration({
      lhs: currentEdgeName,
      rhs: returnEdgeName,
      label: ll.Skip({}),
    }));
  }
}

function translateCondition(
  expression: hl.Expression,
  entryEdgeName: ll.EdgeName,
  thenEdgeName: ll.EdgeName | null,
  elseEdgeName: ll.EdgeName | null,
  edges: ll.EdgeDeclaration[],
  prefix: string,
) {
  switch (expression.kind) {
    case 'ExpressionAccess':
      throw new Error('Not implemented (ExpressionAccess).');
    case 'ExpressionAdd':
      throw new Error('Not implemented (ExpressionAdd).');
    case 'ExpressionAnd':
      throw new Error('Not implemented (ExpressionAnd).');
    case 'ExpressionCall':
      utils.assert(expression.expression.kind === 'ExpressionLiteral', 'Call expects a literal');
      utils.assert(expression.expression.identifier === 'reachable', `Unknown condition ${expression.expression.identifier}`);
      utils.assert(expression.args.length === 1, 'reachable expects 1 argument');
      // if (thenEdgeName) {
      //   edges.push(ll.EdgeDeclaration({
      //     lhs: entryEdgeName,
      //     rhs: thenEdgeName,
      //     label: ll.Reachability({
      //       lhs:
      //     }),
      //   }));
      // }
      throw new Error('Not implemented (ExpressionCall: reachable).');
    case 'ExpressionConstructor':
      throw new Error('Not implemented (ExpressionConstructor).');
    case 'ExpressionEq':
      if (thenEdgeName) {
        edges.push(ll.EdgeDeclaration({
          lhs: entryEdgeName,
          rhs: thenEdgeName,
          label: ll.Comparison({
            lhs: translateExpression(expression.lhs),
            rhs: translateExpression(expression.rhs),
            negated: false,
          }),
        }));
      }
      if (elseEdgeName) {
        edges.push(ll.EdgeDeclaration({
          lhs: entryEdgeName,
          rhs: elseEdgeName,
          label: ll.Comparison({
            lhs: translateExpression(expression.lhs),
            rhs: translateExpression(expression.rhs),
            negated: true,
          }),
        }));
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
        edges.push(ll.EdgeDeclaration({
          lhs: entryEdgeName,
          rhs: thenEdgeName,
          label: ll.Comparison({
            lhs: translateExpression(expression.lhs),
            rhs: translateExpression(expression.rhs),
            negated: true,
          }),
        }));
      }
      if (elseEdgeName) {
        edges.push(ll.EdgeDeclaration({
          lhs: entryEdgeName,
          rhs: elseEdgeName,
          label: ll.Comparison({
            lhs: translateExpression(expression.lhs),
            rhs: translateExpression(expression.rhs),
            negated: false,
          }),
        }));
      }
      return;
    case 'ExpressionOr': {
      const falseEdgeName = ll.EdgeName({ parts: [ll.Literal({ identifier: RANDOM(prefix) })] });
      translateCondition(
        expression.lhs,
        entryEdgeName,
        thenEdgeName,
        falseEdgeName,
        edges,
        prefix,
      );
      translateCondition(
        expression.rhs,
        falseEdgeName,
        thenEdgeName,
        elseEdgeName,
        edges,
        prefix,
      );
      return;
    }
    case 'ExpressionSub':
      throw new Error('Not implemented (ExpressionSub).');
  }
}

function translateDomainElement(domainElement: hl.DomainElement, gameDeclaration: hl.GameDeclaration): hl.Value[] {
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
              return Array.from(
                { length: max - min + 1 },
                (_, index) => hl.ValueElement({ identifier: `${index + min}` }),
              );
            }
            case 'DomainSet':
              return values.elements.map(identifier => hl.ValueElement({ identifier }));
          }
        })
        .reduce<hl.Value[][]>(utils.cartesian, [[]])
        .map(args => hl.ValueConstructor({ identifier: domainElement.identifier, args }));
    case 'DomainLiteral': {
      const referencedDomain = gameDeclaration.domains.find(domainDeclaration => domainDeclaration.identifier === domainElement.identifier);
      return referencedDomain
        ? translateDomainElements(referencedDomain.elements, gameDeclaration)
        : [hl.ValueElement({ identifier: domainElement.identifier })];
    }
  }
}

function translateDomainElements(domainElements: hl.DomainElement[], gameDeclaration: hl.GameDeclaration): hl.Value[] {
  return domainElements.flatMap(domainElement => translateDomainElement(domainElement, gameDeclaration));
}

function translateExpression(expression: hl.Expression): ll.Expression {
  switch (expression.kind) {
    case 'ExpressionAccess':
      return ll.Access({
        lhs: translateExpression(expression.lhs),
        rhs: translateExpression(expression.rhs),
      });
    case 'ExpressionAdd':
      throw new Error('Not implemented (ExpressionAdd).');
    case 'ExpressionAnd':
      throw new Error('Not implemented (ExpressionAnd).');
    case 'ExpressionCall':
      return expression.args.reduce<ll.Expression>(
        (expression, arg) => ll.Access({ lhs: expression, rhs: translateExpression(arg) }),
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
      return ll.Reference({ identifier: expression.identifier });
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

function translateFunctionDeclaration(functionDeclaration: hl.FunctionDeclaration, typeValues: Record<string, hl.Value[]>): ll.ConstantDeclaration {
  utils.assert(functionDeclaration.cases[0].args.length === 1, 'Only simple functions are allowed.');
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
  utils.assert(type.kind === 'Arrow', 'Function is expected to have Arrow type.');
  utils.assert(type.lhs in typeValues, `Unresolved TypeReference "${type.lhs}".`);
  utils.assert(typeValues[type.lhs].length, 'Expected at least one identifier.');
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
    return ll.NamedEntry({
      identifier: serializeValue(value),
      value: ll.Element({
        identifier: serializeValue(evaluateExpression(functionCase.body, binding)),
      }),
    });
  });

  return ll.ConstantDeclaration({
    identifier: functionDeclaration.identifier,
    type,
    value: ll.Map({
      entries: [...entries, ll.DefaultEntry({ value: entries[0].value })],
    }),
  });
}

function translateGameDeclaration(gameDeclaration: hl.GameDeclaration): ll.GameDeclaration {
  const { types, typeValues } = gameDeclaration.domains.reduce((result, domainDeclaration) => {
    const values = translateDomainElements(domainDeclaration.elements, gameDeclaration);
    result.types.push(ll.TypeDeclaration({ identifier: domainDeclaration.identifier, type: ll.Set({ identifiers: values.map(serializeValue) }) }));
    utils.assert(!(domainDeclaration.identifier in result.typeValues), `Duplicated type "${domainDeclaration.identifier}".`);
    result.typeValues[domainDeclaration.identifier] = values;
    return result;
  }, { types: [] as ll.TypeDeclaration[], typeValues: {} as Record<string, hl.Value[]> });
  const constants = gameDeclaration.functions.map(functionDeclaration => translateFunctionDeclaration(functionDeclaration, typeValues));
  const variables = gameDeclaration.variables.map(variableDeclaration => translateVariableDeclaration(variableDeclaration, typeValues));

  const edges = [
    ll.EdgeDeclaration({
      label: ll.Skip({}),
      lhs: ll.EdgeName({ parts: [ll.Literal({ identifier: 'begin' })] }),
      rhs: ll.EdgeName({ parts: [ll.Literal({ identifier: 'rules_0' })] }),
    }),
  ];

  translateAutomatonFunction(
    // TODO: Assert that it exists.
    gameDeclaration.automaton.find(automatonFunction => automatonFunction.name === 'rules')!,
    gameDeclaration.automaton,
    ll.EdgeName({ parts: [ll.Literal({ identifier: 'end' })] }),
    edges,
    variables,
  );

  return ll.GameDeclaration({
    constants,
    edges,
    types,
    variables,
  });
}

function translateType(type: hl.Type): ll.Type {
  switch (type.kind) {
    case 'TypeFunction': {
      utils.assert(type.lhs.kind === 'TypeName', 'Arrow lhs must be TypeReference.')
      return ll.Arrow({ lhs: type.lhs.identifier, rhs: translateType(type.rhs) });
    }
    case 'TypeName':
      return ll.TypeReference({ identifier: type.identifier });
  }
}

function translateValue(value: hl.Value): ll.Value {
  switch (value.kind) {
    case 'ValueConstructor':
      return ll.Element({ identifier: serializeValue(value) });
    case 'ValueElement':
      return ll.Element({ identifier: value.identifier });
    case 'ValueMap':
      utils.assert(value.entries.length, 'At least one entry is required.');
      return ll.Map({
        entries: [
          ...value.entries.map(entry =>
            ll.NamedEntry({
              identifier: serializeValue(entry.key),
              value: translateValue(entry.value),
            }),
          ),
          ll.DefaultEntry({ value: translateValue(value.entries[0].value) }),
        ],
      });
  }
}

function translateVariableDeclaration(variableDeclaration: hl.VariableDeclaration, typeValues: Record<string, hl.Value[]>): ll.VariableDeclaration {
  const type = translateType(variableDeclaration.type);
  return ll.VariableDeclaration({
    identifier: variableDeclaration.identifier,
    defaultValue: translateValue(
      variableDeclaration.defaultValue === null
        ? evaluateDefaultValue(type, typeValues)
        : evaluateExpression(variableDeclaration.defaultValue, {}),
    ),
    type,
  });
}

export default function translate(source: string) {
  const result = lexer.tokenize(source);
  if (result.errors.length > 0)
    throw Object.assign(new Error('Lexer error'), { errors: result.errors });

  parser.input = result.tokens;
  const cstNode = parser.GameDeclaration();

  if (parser.errors.length > 0)
    throw Object.assign(new Error('Parser error'), { errors: parser.errors });

  const hl: hl.GameDeclaration = visitor.visitNode(cstNode);
  const ll = translateGameDeclaration(hl);
  return ll;
}
