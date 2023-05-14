import * as ast from './types';

export function serializeAutomatonStatement(
  automatonStatement: ast.AutomatonStatement,
): string {
  switch (automatonStatement.kind) {
    case 'AutomatonAny':
      return `any {\n${serializeAutomatonStatements(
        automatonStatement.body,
      )}\n}`;
    case 'AutomatonAssignment':
      return `${automatonStatement.identifier}${automatonStatement.accessors
        .map(serializeExpression)
        .map(access => `[${access}]`)
        .join('')} = ${serializeExpression(automatonStatement.expression)}`;
    case 'AutomatonBranch':
      return `branch {\n${automatonStatement.arms
        .flatMap(serializeAutomatonStatements)
        .join('\n} or {\n')}\n}`;
    case 'AutomatonCall':
      return `${automatonStatement.identifier}(${automatonStatement.args
        .map(serializeExpression)
        .join(', ')})`;
    case 'AutomatonForall':
      return `forall ${automatonStatement.identifier}:${serializeType(
        automatonStatement.type,
      )} {\n${serializeAutomatonStatements(automatonStatement.body)}\n}`;
    case 'AutomatonLoop':
      return `loop {\n${serializeAutomatonStatements(
        automatonStatement.body,
      )}\n}`;
    case 'AutomatonWhen':
      return `when ${serializeExpression(
        automatonStatement.expression,
      )} {\n${serializeAutomatonStatements(automatonStatement.body)}\n}`;
    case 'AutomatonWhile':
      return `while ${serializeExpression(
        automatonStatement.expression,
      )} {\n${serializeAutomatonStatements(automatonStatement.body)}\n}`;
  }
}

export function serializeAutomatonStatements(
  automatonStatements: ast.AutomatonStatement[],
) {
  return automatonStatements
    .flatMap(serializeAutomatonStatement)
    .flatMap(line => line.split('\n'))
    .map(line => `  ${line}`)
    .join('\n');
}

export function serializeDomainElement(domainElement: ast.DomainElement) {
  switch (domainElement.kind) {
    case 'DomainGenerator':
      return `${domainElement.identifier}(${domainElement.args.join(
        ', ',
      )})${serializeDomainValues(domainElement.values)}`;
    case 'DomainLiteral':
      return domainElement.identifier;
  }
}

export function serializeDomainValue(domainValue: ast.DomainValue) {
  switch (domainValue.kind) {
    case 'DomainRange':
      return `${domainValue.identifier} in ${domainValue.min}..${domainValue.max}`;
    case 'DomainSet':
      return `${domainValue.identifier} in { ${domainValue.elements.join(
        ', ',
      )} }`;
  }
}

export function serializeDomainValues(domainValues: ast.DomainValue[]) {
  return domainValues.length === 0
    ? ''
    : ` where ${domainValues.map(serializeDomainValue).join(', ')}`;
}

// eslint-disable-next-line complexity -- It's fine.
export function serializeExpression(expression: ast.Expression): string {
  switch (expression.kind) {
    case 'ExpressionAccess':
      return `${serializeExpression(expression.lhs)}[${serializeExpression(
        expression.rhs,
      )}]`;
    case 'ExpressionAdd':
      return `${serializeExpression(expression.lhs)} + ${serializeExpression(
        expression.rhs,
      )}`;
    case 'ExpressionAnd':
      return `${serializeExpressionParentheses(
        expression.lhs,
      )} && ${serializeExpressionParentheses(expression.rhs)}`;
    case 'ExpressionCall':
      return `${serializeExpression(expression.expression)}(${expression.args
        .map(serializeExpression)
        .join(', ')})`;
    case 'ExpressionConstructor':
      return `${expression.identifier}(${expression.args
        .map(serializeExpression)
        .join(', ')})`;
    case 'ExpressionEq':
      return `${serializeExpression(expression.lhs)} == ${serializeExpression(
        expression.rhs,
      )}`;
    case 'ExpressionGt':
      return `${serializeExpression(expression.lhs)} > ${serializeExpression(
        expression.rhs,
      )}`;
    case 'ExpressionGte':
      return `${serializeExpression(expression.lhs)} >= ${serializeExpression(
        expression.rhs,
      )}`;
    case 'ExpressionIf':
      return `if ${serializeExpression(
        expression.cond,
      )} then ${serializeExpression(
        expression.then,
      )} else ${serializeExpression(expression.else)}`;
    case 'ExpressionLiteral':
      return expression.identifier;
    case 'ExpressionLt':
      return `${serializeExpression(expression.lhs)} < ${serializeExpression(
        expression.rhs,
      )}`;
    case 'ExpressionLte':
      return `${serializeExpression(expression.lhs)} <= ${serializeExpression(
        expression.rhs,
      )}`;
    case 'ExpressionMap':
      return `{ ${serializePattern(expression.pattern)} = ${serializeExpression(
        expression.expression,
      )}${serializeDomainValues(expression.domains)} }`;
    case 'ExpressionNe':
      return `${serializeExpression(expression.lhs)} != ${serializeExpression(
        expression.rhs,
      )}`;
    case 'ExpressionOr':
      return `${serializeExpressionParentheses(
        expression.lhs,
      )} || ${serializeExpressionParentheses(expression.rhs)}`;
    case 'ExpressionSub':
      return `${serializeExpression(expression.lhs)} - ${serializeExpression(
        expression.rhs,
      )}`;
  }
}

function serializeExpressionParentheses(expression: ast.Expression): string {
  const serialized = serializeExpression(expression);
  return expression.kind === 'ExpressionOr' ? `(${serialized})` : serialized;
}

export function serializeGameDeclaration(gameDeclaration: ast.GameDeclaration) {
  return [
    ...gameDeclaration.domains.map(
      domainDeclaration =>
        `domain ${domainDeclaration.identifier} = ${domainDeclaration.elements
          .map(serializeDomainElement)
          .join(' | ')}`,
    ),
    ...gameDeclaration.functions.flatMap(functionDeclaration => [
      '',
      `${functionDeclaration.identifier} : ${serializeType(
        functionDeclaration.type,
      )}`,
      ...functionDeclaration.cases.map(
        functionCase =>
          `${functionCase.identifier}(${functionCase.args
            .map(serializePattern)
            .join(', ')}) = ${serializeExpression(functionCase.body)}`,
      ),
    ]),
    ...gameDeclaration.variables.flatMap(variableDeclaration => [
      '',
      `${variableDeclaration.identifier} : ${serializeType(
        variableDeclaration.type,
      )}`,
      ...(variableDeclaration.defaultValue
        ? [
            `${variableDeclaration.identifier} = ${serializeExpression(
              variableDeclaration.defaultValue,
            )}`,
          ]
        : []),
    ]),
    ...gameDeclaration.automaton.flatMap(automatonFunction => [
      '',
      `graph ${automatonFunction.name}(${automatonFunction.args
        .map(
          automatonFunctionArgument =>
            `${automatonFunctionArgument.identifier}: ${serializeType(
              automatonFunctionArgument.type,
            )}`,
        )
        .join(', ')}) {\n${serializeAutomatonStatements(
        automatonFunction.body,
      )}\n}`,
    ]),
  ].join('\n');
}

export function serializePattern(pattern: ast.Pattern): string {
  switch (pattern.kind) {
    case 'PatternConstructor':
      return `${pattern.identifier}(${pattern.args
        .map(serializePattern)
        .join(', ')})`;
    case 'PatternLiteral':
    case 'PatternVariable':
      return pattern.identifier;
    case 'PatternWildcard':
      return '_';
  }
}

export function serializeType(type: ast.Type): string {
  switch (type.kind) {
    case 'TypeFunction':
      return `${serializeType(type.lhs)} -> ${serializeType(type.rhs)}`;
    case 'TypeName':
      return type.identifier;
  }
}
