import { CstChildrenDictionary as Context, CstElement } from 'chevrotain';

import parser from './parser';
import * as ast from './types';
import * as utils from '../utils';

class HLVisitor extends parser.getBaseCstVisitorConstructor() {
  constructor() {
    super();
    this.validateVisitor();
  }

  AutomatonBranch(context: Context): ast.AutomatonStatement[] {
    return this.visitNodes(context.AutomatonStatement);
  }

  AutomatonFunction(context: Context): ast.AutomatonFunction {
    return ast.AutomatonFunction({
      name: this.visitToken(context.Identifier[0]),
      args: this.visitTokens(context.Identifier.slice(1)),
      body: this.visitNodes(context.AutomatonStatement),
    });
  }

  AutomatonStatement(context: Context): ast.AutomatonStatement {
    if ('Equal' in context) {
      return ast.AutomatonAssignment({
        identifier: this.visitToken(context.Identifier[0]),
        accessors: this.visitNodes(context.Expression.slice(0, -1)),
        expression: this.visitNode(context.Expression[0]),
      });
    }

    if ('Identifier' in context) {
      return ast.AutomatonCall({
        args: this.visitNodes(context.Expression),
      });
    }

    if ('KeywordBranch' in context) {
      return ast.AutomatonBranch({
        arms: this.visitNodes(context.AutomatonBranch),
      });
    }

    if ('KeywordIf' in context) {
      return ast.AutomatonIf({
        condition: this.visitNode(context.AutomatonCondition[0]),
        body: this.visitNodes(context.AutomatonStatement),
      });
    }

    if ('KeywordWhile' in context) {
      return ast.AutomatonWhile({
        condition: this.visitNode(context.AutomatonCondition[0]),
        body: this.visitNodes(context.AutomatonStatement),
      });
    }

    throw new Error(`AutomatonStatement ${JSON.stringify(context, null, 2)}`);
  }

  AutomatonCondition(context: Context): ast.AutomatonCondition {
    const lhs = this.visitNode(context.AutomatonConditionAtom[0]);

    if (!('AndAnd' in context) && !('OrOr' in context)) {
      return lhs;
    }

    const rhs = this.visitNode(context.AutomatonConditionAtom[1]);

    if ('AndAnd' in context) {
      return ast.AutomatonConditionAnd({ lhs, rhs });
    }

    if ('OrOr' in context) {
      return ast.AutomatonConditionOr({ lhs, rhs });
    }

    throw new Error(`AutomatonCondition ${JSON.stringify(context, null, 2)}`);
  }

  AutomatonConditionAtom(context: Context): ast.AutomatonCondition {
    if ('KeywordReachable' in context) {
      const target = this.visitNode(context.ExpressionSimple[0]);
      return ast.AutomatonConditionReachable({ target });
    }

    const lhs = this.visitNode(context.ExpressionSimple[0]);
    const rhs = this.visitNode(context.ExpressionSimple[1]);

    if ('EqualEqual' in context) {
      return ast.AutomatonConditionEq({ lhs, rhs });
    }

    if ('NotEqual' in context) {
      return ast.AutomatonConditionNe({ lhs, rhs });
    }

    throw new Error(`Condition ${JSON.stringify(context, null, 2)}`);
  }

  Condition(context: Context): ast.Condition {
    const lhs = this.visitNode(context.ExpressionSimple[0]);
    const rhs = this.visitNode(context.ExpressionSimple[1]);

    if ('EqualEqual' in context) {
      return ast.ConditionEq({ lhs, rhs });
    }

    if ('Gt' in context) {
      return ast.ConditionGt({ lhs, rhs });
    }

    if ('GtEqual' in context) {
      return ast.ConditionGte({ lhs, rhs });
    }

    if ('Lt' in context) {
      return ast.ConditionLt({ lhs, rhs });
    }

    if ('LtEqual' in context) {
      return ast.ConditionLte({ lhs, rhs });
    }

    if ('NotEqual' in context) {
      return ast.ConditionNe({ lhs, rhs });
    }

    throw new Error(`Condition ${JSON.stringify(context, null, 2)}`);
  }

  DomainDeclaration(context: Context): ast.DomainDeclaration {
    return ast.DomainDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      elements: this.visitNodes(context.DomainElement),
    });
  }

  DomainElement(context: Context): ast.DomainElement {
    if ('DomainValues' in context) {
      return ast.DomainGenerator({
        identifier: this.visitToken(context.Identifier[0]),
        args: this.visitTokens(context.Identifier.slice(1)),
        values: this.visitNodes(context.DomainValues),
      });
    }

    return ast.DomainLiteral({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  DomainRange(context: Context): ast.DomainRange {
    throw new Error(`DomainRange ${JSON.stringify(context, null, 2)}`);
  }

  DomainValues(context: Context): ast.DomainValue {
    if ('DotDot' in context) {
      return ast.DomainRange({
        identifier: this.visitToken(context.Identifier[0]),
        min: this.visitToken(context.Identifier[1]),
        max: this.visitToken(context.Identifier[2]),
      });
    }

    return ast.DomainSet({
      identifier: this.visitToken(context.Identifier[0]),
      elements: this.visitTokens(context.Identifier.slice(1)),
    });
  }

  Expression(context: Context): ast.Expression {
    if ('BraceLeft' in context) {
      return ast.ExpressionMap({
        pattern: this.visitNode(context.Pattern[0]),
        expression: this.visitNode(context.Expression[0]),
        domains: this.visitNodes(context.DomainValues),
      });
    }

    if ('KeywordIf' in context) {
      return ast.ExpressionIf({
        cond: this.visitNode(context.Condition[0]),
        then: this.visitNode(context.Expression[0]),
        else: this.visitNode(context.Expression[1]),
      });
    }

    if ('Minus' in context) {
      return ast.ExpressionSub({
        lhs: this.visitNode(context.ExpressionSimple[0]),
        rhs: this.visitNode(context.ExpressionSimple[1]),
      });
    }

    if ('Plus' in context) {
      return ast.ExpressionAdd({
        lhs: this.visitNode(context.ExpressionSimple[0]),
        rhs: this.visitNode(context.ExpressionSimple[1]),
      });
    }

    return this.visitNode(context.ExpressionSimple[0]);
  }

  ExpressionInParenthesis(context: Context): ast.Expression {
    return this.visitNode(context.Expression[0]);
  }

  ExpressionSimple(context: Context): ast.Expression {
    if ('Identifier' in context) {
      if ('ExpressionInParenthesis' in context) {
        return ast.ExpressionConstructor({
          identifier: this.visitToken(context.Identifier[0]),
          args: this.visitNodes(context.ExpressionInParenthesis),
        });
      }

      return ast.ExpressionLiteral({
        identifier: this.visitToken(context.Identifier[0]),
      });
    }

    throw new Error(`ExpressionSimple ${JSON.stringify(context, null, 2)}`);
  }

  FunctionCase(context: Context): ast.FunctionCase {
    return ast.FunctionCase({
      identifier: this.visitToken(context.Identifier[0]),
      args: this.visitNodes(context.Pattern),
      body: this.visitNode(context.Expression[0]),
    });
  }

  GameDeclaration(context: Context): ast.GameDeclaration {
    const typeDeclarations = this.visitNodes(context.TypeDeclaration);

    // Reify FunctionDeclaration.
    const functionCases = this.visitNodes(context.FunctionCase);
    const functionDeclarations = functionCases.reduce(
      (functionDeclarations: ast.FunctionDeclaration[], functionCase) => {
        const existingFunctionDeclaration = functionDeclarations.find(functionDeclaration => functionDeclaration.identifier === functionCase.identifier);
        if (existingFunctionDeclaration) {
          existingFunctionDeclaration.cases.push(functionCase);
        } else {
          const typeDeclaration = typeDeclarations.find(typeDeclaration => typeDeclaration.identifier === functionCase.identifier);
          utils.assert(typeDeclaration, `Type declaration for function "${functionCase.identifier}" not found.`);
          typeDeclarations.splice(typeDeclarations.indexOf(typeDeclaration), 1);
          const functionDeclaration = ast.FunctionDeclaration({
            identifier: typeDeclaration.identifier,
            type: typeDeclaration.type,
            cases: [functionCase],
          });
          functionDeclarations.push(functionDeclaration);
        }

        return functionDeclarations;
      },
      [],
    );

    // Reify VariableDeclaration.
    const variableAssignments = this.visitNodes(context.VariableAssignment);
    const variableDeclarations = variableAssignments.reduce(
      (variableDeclarations: ast.VariableDeclaration[], variableAssignment) => {
        const existingVariableDeclaration = variableDeclarations.find(variableDeclaration => variableDeclaration.identifier === variableAssignment.identifier);
        utils.assert(!existingVariableDeclaration, `Duplicate VariableAssignment found for variable "${variableAssignment.identifier}".`);
        const typeDeclaration = typeDeclarations.find(typeDeclaration => typeDeclaration.identifier === variableAssignment.identifier);
        utils.assert(typeDeclaration, `Type declaration for variable "${variableAssignment.identifier}" not found.`);
        typeDeclarations.splice(typeDeclarations.indexOf(typeDeclaration), 1);
        const variableDeclaration = ast.VariableDeclaration({
          identifier: typeDeclaration.identifier,
          type: typeDeclaration.type,
          defaultValue: variableAssignment.expression,
        });
        variableDeclarations.push(variableDeclaration);
        return variableDeclarations;
      },
      [],
    );

    // Reify VariableDeclaration without default values.
    for (const typeDeclaration of typeDeclarations) {
      const variableDeclaration = ast.VariableDeclaration({
        identifier: typeDeclaration.identifier,
        type: typeDeclaration.type,
        defaultValue: null,
      });
      variableDeclarations.push(variableDeclaration);
    }

    return ast.GameDeclaration({
      automaton: this.visitNodes(context.AutomatonFunction),
      domains: this.visitNodes(context.DomainDeclaration),
      functions: functionDeclarations,
      variables: variableDeclarations,
    });
  }

  Pattern(context: Context): ast.Pattern {
    if ('KeywordWildcard' in context) return ast.PatternWildcard({});

    if ('ParenthesisLeft' in context) {
      return ast.PatternConstructor({
        identifier: this.visitToken(context.Identifier[0]),
        args: this.visitNodes(context.Pattern),
      });
    }

    const identifier = this.visitToken(context.Identifier[0]);
    return identifier[0] === identifier[0].toUpperCase()
      ? ast.PatternLiteral({ identifier })
      : ast.PatternVariable({ identifier });
  }

  Type(context: Context): ast.Type {
    if ('MinusGt' in context) {
      return ast.TypeFunction({
        lhs: ast.TypeName({
          identifier: this.visitToken(context.Identifier[0]),
        }),
        rhs: this.visitNode(context.Type[0]),
      });
    }

    return ast.TypeName({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  TypeDeclaration(context: Context): ast.TypeDeclaration {
    return ast.TypeDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
    });
  }

  VariableAssignment(context: Context): ast.VariableAssignment {
    return ast.VariableAssignment({
      identifier: this.visitToken(context.Identifier[0]),
      expression: this.visitNode(context.Expression[0]),
    });
  }

  visitNode(cstElement: CstElement) {
    if (!('name' in cstElement)) throw new Error('CstNode expected');
    return this.visit(cstElement);
  }

  visitNodes(cstElements: CstElement[] = []) {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    return cstElements.map(this.visitNode, this);
  }

  visitToken(cstElement: CstElement) {
    if (!('tokenType' in cstElement)) throw new Error('Token expected');
    return cstElement.image;
  }

  visitTokens(cstElements: CstElement[] = []) {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    return cstElements.map(this.visitToken, this);
  }
}

export default new HLVisitor();
