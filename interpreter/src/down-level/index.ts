import lexer from './lexer';
import parser from './parser';
import * as hl from './types';
import visitor from './visitor';
import * as ll from '../ast/types';
import * as utils from '../utils';

// FIXME: Properly represent patterns and matches.
type SerializedValue = string & { readonly __tag: symbol };

function __serializeConstructor(identifier: string, args: SerializedValue[]): SerializedValue {
  return `${identifier}___${args.join('_')}` as SerializedValue;
}

function __deserializeConstructor(value: SerializedValue): [string, SerializedValue[]] {
  const parts = value.split('___', 2);
  return [parts[0], parts.length === 1 ? [] : parts[1].split('_') as SerializedValue[]];
}

// FIXME: Here we assume the pattern is already matched.
function __bindingsForConstructor(pattern: hl.Pattern, value: SerializedValue): Record<string, SerializedValue> {
  const [, args] = __deserializeConstructor(value);
  switch (pattern.kind) {
    case 'PatternConstructor':
      return pattern.args.map((pattern, index) => __bindingsForConstructor(pattern, args[index])).reduce((a, b) => Object.assign(a, b));
    case 'PatternLiteral':
      return {};
    case 'PatternVariable':
      return { [pattern.identifier]: value };
    case 'PatternWildcard':
      return {};
  }
}

function __matchConstructor(pattern: hl.Pattern, value: SerializedValue): boolean {
  const [identifier, args] = __deserializeConstructor(value);
  switch (pattern.kind) {
    case 'PatternConstructor':
      return pattern.identifier === identifier && args.length === pattern.args.length && pattern.args.every((pattern, index) => __matchConstructor(pattern, args[index]));
    case 'PatternLiteral':
      return pattern.identifier === identifier && args.length === 0;
    case 'PatternVariable':
      return true;
    case 'PatternWildcard':
      return true;
  }
}

function resolveCondition(condition: hl.Condition, bindings: Record<string, SerializedValue>): boolean {
  switch (condition.kind) {
    case 'ConditionEq': {
      const lhs = resolveExpression(condition.lhs, bindings);
      const rhs = resolveExpression(condition.rhs, bindings);
      console.log(`ConditionEq(${JSON.stringify(condition.lhs)}, ${JSON.stringify(condition.rhs)}) => ConditionEq(${lhs}, ${rhs}) => ${lhs === rhs}`);
      return lhs === rhs;
    }
  }
}

// FIXME: This should return a value of some sort?
function resolveExpression(expression: hl.Expression, bindings: Record<string, SerializedValue>): SerializedValue {
  switch (expression.kind) {
    case 'ExpressionAdd': {
      const lhs = resolveExpression(expression.lhs, bindings);
      const rhs = resolveExpression(expression.rhs, bindings);
      console.log(`ExpressionAdd(${JSON.stringify(expression.lhs)}, ${JSON.stringify(expression.rhs)}) => ExpressionAdd(${lhs}, ${rhs}) => ${Number(lhs) + Number(rhs)}`);
      return String(Number(lhs) + Number(rhs)) as SerializedValue;
    }
    case 'ExpressionConstructor':
      return __serializeConstructor(expression.identifier, expression.args.map(expression => resolveExpression(expression, bindings)));
    case 'ExpressionIf':
      return resolveCondition(expression.cond, bindings)
        ? resolveExpression(expression.then, bindings)
        : resolveExpression(expression.else, bindings);
    case 'ExpressionLiteral':
      return expression.identifier in bindings
        ? bindings[expression.identifier]
        : expression.identifier as SerializedValue;
    case 'ExpressionSub': {
      const lhs = resolveExpression(expression.lhs, bindings);
      const rhs = resolveExpression(expression.rhs, bindings);
      console.log(`ExpressionSub(${JSON.stringify(expression.lhs)}, ${JSON.stringify(expression.rhs)}) => ExpressionSub(${lhs}, ${rhs}) => ${Number(lhs) - Number(rhs)}`);
      return String(Number(lhs) - Number(rhs)) as SerializedValue;
    }
  }
}

function resolveTypeSets(type: ll.Type, typeDeclarations: ll.TypeDeclaration[]): ll.Set[] {
  switch (type.kind) {
    case 'Arrow': {
      const referencedType = typeDeclarations.find(typeDeclaration => typeDeclaration.identifier === type.lhs);
      utils.assert(referencedType, `Unresolved TypeReference "${type.lhs}".`);
      utils.assert(referencedType.type.kind === 'Set', `No left-nested Arrow types allowed.`);
      return [referencedType.type, ...resolveTypeSets(type.rhs, typeDeclarations)];
    }
    case 'Set':
      return [type];
    case 'TypeReference': {
      const referencedType = typeDeclarations.find(typeDeclaration => typeDeclaration.identifier === type.identifier);
      utils.assert(referencedType, `Unresolved TypeReference "${type.identifier}".`);
      return resolveTypeSets(referencedType.type, typeDeclarations);
    }
  }
}

function translateDomainDeclaration(domainDeclaration: hl.DomainDeclaration, hl: hl.GameDeclaration): ll.TypeDeclaration {
  return ll.TypeDeclaration({
    identifier: domainDeclaration.identifier,
    type: ll.Set({
      identifiers: translateDomainElements(domainDeclaration.elements, hl),
    }),
  });
}

function translateDomainElement(domainElement: hl.DomainElement, hl: hl.GameDeclaration): SerializedValue[] {
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
                (_, index) => `${index + min}` as SerializedValue,
              );
            }
            case 'DomainSet':
              return values.elements as SerializedValue[];
          }
        })
        .reduce<SerializedValue[][]>(utils.cartesian, [[]])
        .map(args => __serializeConstructor(domainElement.identifier, args));
    case 'DomainLiteral': {
      const referencedDomain = hl.domains.find(domainDeclaration => domainDeclaration.identifier === domainElement.identifier);
      return referencedDomain
        ? translateDomainElements(referencedDomain.elements, hl)
        : [domainElement.identifier as SerializedValue];
    }
  }
}

function translateDomainElements(domainElements: hl.DomainElement[], hl: hl.GameDeclaration): SerializedValue[] {
  return domainElements.flatMap(domainElement => translateDomainElement(domainElement, hl));
}

function translateFunctionDeclaration(functionDeclaration: hl.FunctionDeclaration, typeDeclarations: ll.TypeDeclaration[]): ll.ConstantDeclaration {
  const type = translateType(functionDeclaration.type);
  utils.assert(type.kind === 'Arrow', 'Function is expected to have Arrow type.');
  const typeSets = resolveTypeSets(type, typeDeclarations);
  utils.assert(typeSets.length === 2, 'Only simple functions are allowed');
  const reciveableIdentifiers = typeSets[0].identifiers as SerializedValue[];
  const returnableIdentifiers = typeSets[1].identifiers as SerializedValue[];
  utils.assert(returnableIdentifiers.length, 'Expected at least one identifier.');
  return ll.ConstantDeclaration({
    identifier: functionDeclaration.identifier,
    type,
    value: ll.Map({
      entries: [
        ll.DefaultEntry({ value: ll.Element({ identifier: returnableIdentifiers[0] }) }),
        ...reciveableIdentifiers.map(identifier => {
          const functionCase = functionDeclaration.cases.find(functionCase => {
            utils.assert(functionCase.args.length === 1, 'Not implemented.');
            const pattern = functionCase.args[0];
            return __matchConstructor(pattern, identifier);
          });
          utils.assert(functionCase, `No FunctionCase found for "${identifier}".`);
          const bindings = __bindingsForConstructor(functionCase.args[0], identifier);
          return ll.NamedEntry({
            identifier,
            value: ll.Element({
              identifier: resolveExpression(functionCase.body, bindings),
            }),
          });
        }),
      ],
    }),
  });
}

function translateGameDeclaration(hl: hl.GameDeclaration): ll.GameDeclaration {
  const types = hl.domains.map(domainDeclaration => translateDomainDeclaration(domainDeclaration, hl));
  const constants = hl.functions.map(functionDeclaration => translateFunctionDeclaration(functionDeclaration, types));
  return ll.GameDeclaration({
    constants,
    edges: [],
    types,
    variables: [],
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
