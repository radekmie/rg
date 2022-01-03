import { CstChildrenDictionary as Context, CstElement } from 'chevrotain';

import parser from './parser';
import * as ast from './types';

class HLVisitor extends parser.getBaseCstVisitorConstructor() {
  constructor() {
    super();
    this.validateVisitor();
  }

  Condition(context: Context): ast.Condition {
    return ast.ConditionEq({
      lhs: this.visitNode(context.Expression2[0]),
      rhs: this.visitNode(context.Expression2[1]),
    });
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

  DomainValues(context: Context): ast.DomainValues {
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
        cond: this.visitNode(context.Condition[0]),
        then: this.visitNode(context.Expression[0]),
        else: this.visitNode(context.Expression[1]),
      });
    }

    if ('Minus' in context) {
      return ast.ExpressionSub({
        lhs: this.visitNode(context.Expression2[0]),
        rhs: this.visitNode(context.Expression2[1]),
      });
    }

    if ('Plus' in context) {
      return ast.ExpressionAdd({
        lhs: this.visitNode(context.Expression2[0]),
        rhs: this.visitNode(context.Expression2[1]),
      });
    }

    return this.visitNode(context.Expression2[0]);
  }

  Expression2(context: Context): ast.Expression {
    if ('Identifier' in context) {
      if ('ParenthesisLeft' in context) {
        return ast.ExpressionConstructor({
          identifier: this.visitToken(context.Identifier[0]),
          args: this.visitNodes(context.Expression),
        });
      }

      return ast.ExpressionLiteral({
        identifier: this.visitToken(context.Identifier[0]),
      });
    }

    throw new Error('Expression2 ' + JSON.stringify(context));
  }

  FunctionCase(context: Context): ast.FunctionCase {
    return ast.FunctionCase({
      identifier: this.visitToken(context.Identifier[0]),
      args: this.visitNodes(context.Pattern),
      body: this.visitNode(context.Expression[0]),
    });
  }

  FunctionDeclaration(context: Context): ast.FunctionDeclaration {
    return ast.FunctionDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.FunctionType[0]),
      cases: this.visitNodes(context.FunctionCase),
    });
  }

  FunctionType(context: Context): ast.Type {
    if ('MinusGt' in context) {
      return ast.TypeFunction({
        lhs: ast.TypeName({
          identifier: this.visitToken(context.Identifier[0]),
        }),
        rhs: this.visitNode(context.FunctionType[0]),
      });
    }

    return ast.TypeName({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  GameDeclaration(context: Context): ast.GameDeclaration {
    return ast.GameDeclaration({
      domains: this.visitNodes(context.DomainDeclaration),
      functions: this.visitNodes(context.FunctionDeclaration),
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
