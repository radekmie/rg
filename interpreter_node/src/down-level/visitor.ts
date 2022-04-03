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
      args: this.visitNodes(context.AutomatonFunctionArgument),
      body: this.visitNodes(context.AutomatonStatement),
    });
  }

  AutomatonFunctionArgument(context: Context): ast.AutomatonFunctionArgument {
    return ast.AutomatonFunctionArgument({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
    });
  }

  AutomatonStatement(context: Context): ast.AutomatonStatement {
    if ('Equal' in context) {
      return ast.AutomatonAssignment({
        identifier: this.visitToken(context.Identifier[0]),
        accessors: this.visitNodes(context.Expression.slice(0, -1)),
        expression: this.visitNode(context.Expression.slice(-1)[0]),
      });
    }

    if ('Identifier' in context) {
      return ast.AutomatonCall({
        identifier: this.visitToken(context.Identifier[0]),
        args: this.visitNodes(context.Expression),
      });
    }

    if ('KeywordBranch' in context) {
      return ast.AutomatonBranch({
        arms: this.visitNodes(context.AutomatonBranch),
      });
    }

    if ('KeywordWhen' in context) {
      return ast.AutomatonWhen({
        expression: this.visitNode(context.Expression[0]),
        body: this.visitNodes(context.AutomatonStatement),
      });
    }

    if ('KeywordLoop' in context) {
      return ast.AutomatonLoop({
        body: this.visitNodes(context.AutomatonStatement),
      });
    }

    if ('KeywordWhile' in context) {
      return ast.AutomatonWhile({
        expression: this.visitNode(context.Expression[0]),
        body: this.visitNodes(context.AutomatonStatement),
      });
    }

    throw new Error(`AutomatonStatement ${JSON.stringify(context, null, 2)}`);
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
    if ('KeywordIf' in context) {
      return ast.ExpressionIf({
        cond: this.visitNode(context.Expression[0]),
        then: this.visitNode(context.Expression[1]),
        else: this.visitNode(context.Expression[2]),
      });
    }

    if ('BraceLeft' in context) {
      return ast.ExpressionMap({
        pattern: this.visitNode(context.Pattern[0]),
        expression: this.visitNode(context.Expression[0]),
        domains: this.visitNodes(context.DomainValues),
      });
    }

    if ('OrOr' in context) {
      return ast.ExpressionOr({
        lhs: this.visitNode(context.Expression2[0]),
        rhs: this.visitNode(context.Expression[0]),
      });
    }

    return this.visitNode(context.Expression2[0]);
  }

  Expression2(context: Context): ast.Expression {
    if ('AndAnd' in context) {
      return ast.ExpressionAnd({
        lhs: this.visitNode(context.Expression3[0]),
        rhs: this.visitNode(context.Expression2[0]),
      });
    }

    return this.visitNode(context.Expression3[0]);
  }

  Expression3(context: Context): ast.Expression {
    if ('EqualEqual' in context) {
      return ast.ExpressionEq({
        lhs: this.visitNode(context.Expression4[0]),
        rhs: this.visitNode(context.Expression3[0]),
      });
    }

    if ('Gt' in context) {
      return ast.ExpressionGt({
        lhs: this.visitNode(context.Expression4[0]),
        rhs: this.visitNode(context.Expression3[0]),
      });
    }

    if ('GtEqual' in context) {
      return ast.ExpressionGte({
        lhs: this.visitNode(context.Expression4[0]),
        rhs: this.visitNode(context.Expression3[0]),
      });
    }

    if ('Lt' in context) {
      return ast.ExpressionLt({
        lhs: this.visitNode(context.Expression4[0]),
        rhs: this.visitNode(context.Expression3[0]),
      });
    }

    if ('LtEqual' in context) {
      return ast.ExpressionLte({
        lhs: this.visitNode(context.Expression4[0]),
        rhs: this.visitNode(context.Expression3[0]),
      });
    }

    if ('NotEqual' in context) {
      return ast.ExpressionNe({
        lhs: this.visitNode(context.Expression4[0]),
        rhs: this.visitNode(context.Expression3[0]),
      });
    }

    return this.visitNode(context.Expression4[0]);
  }

  Expression4(context: Context): ast.Expression {
    if ('Minus' in context) {
      return ast.ExpressionSub({
        lhs: this.visitNode(context.Expression5[0]),
        rhs: this.visitNode(context.Expression4[0]),
      });
    }

    if ('Plus' in context) {
      return ast.ExpressionAdd({
        lhs: this.visitNode(context.Expression5[0]),
        rhs: this.visitNode(context.Expression4[0]),
      });
    }

    return this.visitNode(context.Expression5[0]);
  }

  Expression5(context: Context): ast.Expression {
    if ('Identifier' in context) {
      return (context.ExpressionSuffix ?? []).reduce<ast.Expression>(
        (expression, suffix) => {
          utils.assert('children' in suffix, 'ExpressionSuffix expected.');
          const expressions = this.visitNode(suffix);
          if ('BracketLeft' in suffix.children) {
            utils.assert(expressions.length === 1, 'Access require exactly one expression.');
            return ast.ExpressionAccess({
              lhs: expression,
              rhs: expressions[0],
            });
          }

          if ('ParenthesisLeft' in suffix.children) {
            // That's how we differentiate calls from constructors.
            if (expression.kind === 'ExpressionLiteral' && expression.identifier[0] === expression.identifier[0].toUpperCase()) {
              return ast.ExpressionConstructor({
                identifier: expression.identifier,
                args: expressions,
              });
            }

            return ast.ExpressionCall({ expression, args: expressions });
          }

          throw new Error(`ExpressionSuffix ${JSON.stringify(context, null, 2)}`);
        },
        ast.ExpressionLiteral({
          identifier: this.visitToken(context.Identifier[0]),
        }),
      );
    }

    throw new Error(`Expression5 ${JSON.stringify(context, null, 2)}`);
  }

  ExpressionSuffix(context: Context): ast.Expression[] {
    return this.visitNodes(context.Expression);
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
