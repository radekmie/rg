import {
  CstChildrenDictionary as Context,
  CstElement,
  CstNode,
} from 'chevrotain';

import * as ast from './types';
import * as cst from '../cst';

class Visitor extends cst.parser.getBaseCstVisitorConstructor() {
  constructor() {
    super();
    this.validateVisitor();
  }

  ConstantDeclaration(context: Context): ast.ConstantDeclaration {
    return ast.ConstantDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
      value: this.visitNode(context.Value[0]),
    });
  }

  EdgeDeclaration(context: Context): ast.EdgeDeclaration {
    return ast.EdgeDeclaration({
      lhs: this.visitNode(context.EdgeName[0]),
      rhs: this.visitNode(context.EdgeName[1]),
      label: this.visitNode(context.EdgeLabel[0]),
    });
  }

  EdgeLabel(context: Context): ast.EdgeLabel {
    if ('Equal' in context) {
      return ast.Assignment({
        lhs: this.visitNode(context.Expression[0]),
        rhs: this.visitNode(context.Expression[1]),
      });
    }

    if ('BangEqual' in context || 'EqualEqual' in context) {
      return ast.Comparison({
        lhs: this.visitNode(context.Expression[0]),
        rhs: this.visitNode(context.Expression[1]),
        negated: 'BangEqual' in context,
      });
    }

    if ('Bang' in context || 'Question' in context) {
      return ast.Reachability({
        lhs: ast.EdgeName({
          parts: [
            ast.Literal({
              identifier: this.visitToken(context.Identifier[0]),
            }),
          ],
        }),
        rhs: ast.EdgeName({
          parts: [
            ast.Literal({
              identifier: this.visitToken(context.Identifier[1]),
            }),
          ],
        }),
        negated: 'Bang' in context,
      });
    }

    return ast.Skip({});
  }

  EdgeName(context: Context): ast.EdgeName {
    return ast.EdgeName({ parts: this.visitNodes(context.EdgeNamePart) });
  }

  EdgeNamePart(context: Context): ast.EdgeNamePart {
    if ('ParenthesisLeft' in context) {
      return ast.Binding({
        identifier: this.visitToken(context.Identifier[0]),
        type: this.visitNode(context.Type[0]),
      });
    }

    return ast.Literal({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  Expression(context: Context): ast.Expression {
    const isCast = 'ParenthesisLeft' in context;
    const expressions = context.Expression?.slice() ?? [];

    const identifier = this.visitToken(context.Identifier[0]);
    const expression = isCast
      ? ast.Cast({
          lhs: ast.TypeReference({ identifier }),
          rhs: this.visitNode(expressions[0]),
        })
      : ast.Reference({ identifier });

    const cstNodes = isCast ? expressions.slice(1) : expressions;
    return cstNodes.reduce<ast.Expression>(
      (expression, cstNode) =>
        ast.Access({ lhs: expression, rhs: this.visitNode(cstNode) }),
      expression,
    );
  }

  GameDeclaration(context: Context): ast.GameDeclaration {
    return ast.GameDeclaration({
      constants: this.visitNodes(context.ConstantDeclaration),
      edges: this.visitNodes(context.EdgeDeclaration),
      types: this.visitNodes(context.TypeDeclaration),
      variables: this.visitNodes(context.VariableDeclaration),
    });
  }

  Type(context: Context): ast.Type {
    if ('Arrow' in context) {
      return ast.Arrow({
        lhs: this.visitToken(context.Identifier[0]),
        rhs: this.visitNode(context.Type[0]),
      });
    }

    if ('BraceLeft' in context) {
      return ast.Set({ identifiers: this.visitTokens(context.Identifier) });
    }

    return ast.TypeReference({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  TypeDeclaration(context: Context): ast.TypeDeclaration {
    return ast.TypeDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
    });
  }

  Value(context: Context): ast.Value {
    if ('BraceLeft' in context) {
      return ast.Map({ entries: this.visitNodes(context.ValueEntry) });
    }

    return ast.Element({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  ValueEntry(context: Context): ast.ValueEntry {
    if ('Identifier' in context) {
      return ast.NamedEntry({
        identifier: this.visitToken(context.Identifier[0]),
        value: this.visitNode(context.Value[0]),
      });
    }

    return ast.DefaultEntry({ value: this.visitNode(context.Value[0]) });
  }

  VariableDeclaration(context: Context): ast.VariableDeclaration {
    return ast.VariableDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
      defaultValue: this.visitNode(context.Value[0]),
    });
  }

  visitNode(cstElement: CstElement) {
    if (!('name' in cstElement)) {
      throw new Error('CstNode expected');
    }
    return this.visit(cstElement);
  }

  visitNodes(cstElements: CstElement[] = []) {
    // eslint-disable-next-line @typescript-eslint/unbound-method -- We provide `this` explicitly.
    return cstElements.map(this.visitNode, this);
  }

  visitToken(cstElement: CstElement) {
    if (!('tokenType' in cstElement)) {
      throw new Error('Token expected');
    }
    return cstElement.image;
  }

  visitTokens(cstElements: CstElement[] = []) {
    // eslint-disable-next-line @typescript-eslint/unbound-method -- We provide `this` explicitly.
    return cstElements.map(this.visitToken, this);
  }
}

export const visitor = new Visitor();
export function visit(cstNode: CstNode): ast.GameDeclaration {
  return visitor.visitNode(cstNode);
}
