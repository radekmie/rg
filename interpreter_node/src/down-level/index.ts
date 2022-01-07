import lexer from './lexer';
import parser from './parser';
import * as hl from './types';
import visitor from './visitor';
import * as ll from '../ast/types';
import * as utils from '../utils';

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

function evaluateCondition(condition: hl.Condition, binding: Record<string, hl.Value>): boolean {
  switch (condition.kind) {
    case 'ConditionEq': {
      const lhs = evaluateExpression(condition.lhs, binding);
      const rhs = evaluateExpression(condition.rhs, binding);
      const equal = evaluateEquality(lhs, rhs);
      return equal;
    }
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

function evaluateEquality(lhs: hl.Value, rhs: hl.Value): boolean {
  switch (lhs.kind) {
    case 'ValueConstructor':
      utils.assert(rhs.kind === 'ValueConstructor', 'Incomparable values.');
      return (
        lhs.identifier === rhs.identifier &&
        lhs.args.length === rhs.args.length &&
        lhs.args.every((arg, index) => evaluateEquality(arg, rhs.args[index]))
      );
    case 'ValueElement':
      utils.assert(rhs.kind === 'ValueElement', 'Incomparable values.');
      return lhs.identifier === rhs.identifier;
  }
}

function evaluateExpression(expression: hl.Expression, binding: Record<string, hl.Value>): hl.Value {
  switch (expression.kind) {
    case 'ExpressionAdd': {
      const lhs = evaluateExpressionIdentifier(expression.lhs, binding);
      const rhs = evaluateExpressionIdentifier(expression.rhs, binding);
      return hl.ValueElement({ identifier: String(Number(lhs) + Number(rhs)) });
    }
    case 'ExpressionConstructor':
      return hl.ValueConstructor({
        identifier: expression.identifier,
        args: expression.args.map(expression => evaluateExpression(expression, binding)),
      });
    case 'ExpressionIf':
      return evaluateCondition(expression.cond, binding)
        ? evaluateExpression(expression.then, binding)
        : evaluateExpression(expression.else, binding);
    case 'ExpressionLiteral':
      return expression.identifier in binding
        ? binding[expression.identifier]
        : hl.ValueElement({ identifier: expression.identifier });
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

function serializeValue(value: hl.Value): string {
  switch (value.kind) {
    case 'ValueConstructor':
      return `${value.identifier}__${value.args.map(serializeValue).join('_')}`;
    case 'ValueElement':
      return value.identifier;
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

function translateFunctionDeclaration(functionDeclaration: hl.FunctionDeclaration, typeValues: Record<string, hl.Value[]>): ll.ConstantDeclaration {
  utils.assert(functionDeclaration.cases[0].args.length === 1, 'Only simple functions are allowed.');
  functionDeclaration.cases.forEach(functionCase => {
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
  return ll.GameDeclaration({
    constants,
    edges: [],
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
