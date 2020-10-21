import { CstChildrenDictionary as Context, CstElement } from 'chevrotain';

import parser from './parser';
import * as types from './types';

class RGVisitor extends parser.getBaseCstVisitorConstructor() {
  constructor() {
    super();
    this.validateVisitor();
  }

  ConstantDeclaration(context: Context): types.ConstantDeclaration {
    return types.ConstantDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
      value: this.visitNode(context.Value[0]),
    });
  }

  EdgeDeclaration(context: Context): types.EdgeDeclaration {
    return types.EdgeDeclaration({
      lhs: this.visitNode(context.EdgeName[0]),
      rhs: this.visitNode(context.EdgeName[1]),
      label: this.visitNode(context.EdgeLabel[0]),
    });
  }

  EdgeLabel(context: Context): types.EdgeLabel {
    if ('Equal' in context) {
      return types.Assignment({
        lhs: this.visitNode(context.Expression[0]),
        rhs: this.visitNode(context.Expression[1]),
      });
    }

    if ('BangEqual' in context || 'EqualEqual' in context) {
      return types.Comparison({
        lhs: this.visitNode(context.Expression[0]),
        rhs: this.visitNode(context.Expression[1]),
        negated: 'BangEqual' in context,
      });
    }

    if ('KeywordMode' in context) {
      return types.Reachability({
        lhs: types.EdgeName({
          parts: [
            types.Literal({
              identifier: this.visitToken(context.Identifier[0]),
            }),
          ],
        }),
        rhs: types.EdgeName({
          parts: [
            types.Literal({
              identifier: this.visitToken(context.Identifier[1]),
            }),
          ],
        }),
        mode: this.visitToken(context.KeywordMode[0]) as 'not' | 'rev',
      });
    }

    return types.Skip({});
  }

  EdgeName(context: Context): types.EdgeName {
    return types.EdgeName({ parts: this.visitNodes(context.EdgeNamePart) });
  }

  EdgeNamePart(context: Context): types.EdgeNamePart {
    if ('ParenthesisLeft' in context) {
      return types.Binding({
        identifier: this.visitToken(context.Identifier[0]),
        type: this.visitNode(context.Type[0]),
      });
    }

    return types.Literal({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  Expression(context: Context): types.Expression {
    if ('BracketLeft' in context) {
      return context.Expression.reduce(
        (expression, cstNode) =>
          types.Access({ lhs: expression, rhs: this.visitNode(cstNode) }),
        types.Reference({
          identifier: this.visitToken(context.Identifier[0]),
        }) as types.Expression,
      );
    }

    if ('ParenthesisLeft' in context) {
      return types.Cast({
        lhs: types.TypeReference({
          identifier: this.visitToken(context.Identifier[0]),
        }),
        rhs: this.visitNode(context.Expression[0]),
      });
    }

    return types.Reference({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  GameDeclaration(context: Context): types.GameDeclaration {
    return types.GameDeclaration({
      constants: this.visitNodes(context.ConstantDeclaration),
      edges: this.visitNodes(context.EdgeDeclaration),
      types: this.visitNodes(context.TypeDeclaration),
      variables: this.visitNodes(context.VariableDeclaration),
    });
  }

  Type(context: Context): types.Type {
    if ('Arrow' in context) {
      return types.Arrow({
        lhs: this.visitToken(context.Identifier[0]),
        rhs: this.visitNode(context.Type[0]),
      });
    }

    if ('BraceLeft' in context)
      return types.Set({ identifiers: this.visitTokens(context.Identifier) });

    return types.TypeReference({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  TypeDeclaration(context: Context): types.TypeDeclaration {
    return types.TypeDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
    });
  }

  Value(context: Context): types.Value {
    if ('BraceLeft' in context)
      return types.Map({ entries: this.visitNodes(context.ValueEntry) });

    return types.Element({
      identifier: this.visitToken(context.Identifier[0]),
    });
  }

  ValueEntry(context: Context): types.ValueEntry {
    if ('Identifier' in context) {
      return types.NamedEntry({
        identifier: this.visitToken(context.Identifier[0]),
        value: this.visitNode(context.Value[0]),
      });
    }

    return types.DefaultEntry({ value: this.visitNode(context.Value[0]) });
  }

  VariableDeclaration(context: Context): types.VariableDeclaration {
    return types.VariableDeclaration({
      identifier: this.visitToken(context.Identifier[0]),
      type: this.visitNode(context.Type[0]),
      defaultValue: this.visitNode(context.Value[0]),
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

export default new RGVisitor();
