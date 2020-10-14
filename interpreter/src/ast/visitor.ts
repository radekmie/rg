import { CstChildrenDictionary, CstElement } from 'chevrotain';

import parser from './parser';
import * as types from './types';

class RGVisitor extends parser.getBaseCstVisitorConstructor() {
  constructor() {
    super();
    this.validateVisitor();
  }

  constDeclaration(context: CstChildrenDictionary): types.ConstDeclaration {
    return {
      kind: 'ConstDeclaration',
      identifier: this.visitNode(context.identifier[0]),
      type: this.visitNode(context.type[0]),
      value: this.visitNode(context.value[0]),
    };
  }

  edgeDeclaration(context: CstChildrenDictionary): types.EdgeDeclaration {
    return {
      kind: 'EdgeDeclaration',
      lhs: this.visitNode(context.identifier[0]),
      rhs: this.visitNode(context.identifier[1]),
      label: this.visitNode(context.edgeLabel[0]),
    };
  }

  edgeLabel(context: CstChildrenDictionary): types.EdgeLabel {
    if ('Equal' in context) {
      return {
        kind: 'Assignment',
        lhs: this.visitNode(context.expression[0]),
        rhs: this.visitNode(context.expression[1]),
      };
    }

    if ('BangEqual' in context || 'EqualEqual' in context) {
      return {
        kind: 'Comparison',
        lhs: this.visitNode(context.expression[0]),
        rhs: this.visitNode(context.expression[1]),
        negated: 'BangEqual' in context,
      };
    }

    if ('KeywordMode' in context) {
      return {
        kind: 'Reachability',
        lhs: this.visitNode(context.identifier[0]),
        rhs: this.visitNode(context.identifier[1]),
        mode: this.visitToken(context.KeywordMode[0]) as 'not' | 'rev',
      };
    }

    return { kind: 'Skip' };
  }

  expression(context: CstChildrenDictionary): types.Expression {
    if ('BracketLeft' in context) {
      return {
        kind: 'Access',
        lhs: this.visitNode(context.identifier[0]),
        rhs: this.visitNode(context.expression[0]),
      };
    }

    if ('ParenthesisLeft' in context) {
      return {
        kind: 'Cast',
        lhs: this.visitNode(context.identifier[0]),
        rhs: this.visitNode(context.expression[0]),
      };
    }

    return {
      kind: 'VarReference',
      identifier: this.visitNode(context.identifier[0]),
    };
  }

  game(context: CstChildrenDictionary): types.Game {
    return {
      kind: 'Game',
      consts: this.visitNodes(context.constDeclaration),
      edges: this.visitNodes(context.edgeDeclaration),
      types: this.visitNodes(context.typeDeclaration),
      vars: this.visitNodes(context.varDeclaration),
    };
  }

  identifier(context: CstChildrenDictionary): types.Identifier {
    return {
      kind: 'Identifier',
      identifier: this.visitToken(context.Identifier[0]),
    };
  }

  type(context: CstChildrenDictionary): types.Type {
    if ('Arrow' in context) {
      return {
        kind: 'Arrow',
        lhs: this.visitNode(context.identifier[0]),
        rhs: this.visitNode(context.type[0]),
      };
    }

    if ('BraceLeft' in context)
      return { kind: 'Set', identifiers: this.visitNodes(context.identifier) };

    return {
      kind: 'TypeReference',
      identifier: this.visitNode(context.identifier[0]),
    };
  }

  typeDeclaration(context: CstChildrenDictionary): types.TypeDeclaration {
    return {
      kind: 'TypeDeclaration',
      identifier: this.visitNode(context.identifier[0]),
      type: this.visitNode(context.type[0]),
    };
  }

  value(context: CstChildrenDictionary): types.Value {
    if ('BraceLeft' in context)
      return { kind: 'Map', entries: this.visitNodes(context.valueEntry) };

    return {
      kind: 'Reference',
      identifier: this.visitNode(context.identifier[0]),
    };
  }

  valueEntry(context: CstChildrenDictionary): types.ValueEntry {
    if ('identifier' in context) {
      return {
        kind: 'NamedEntry',
        identifier: this.visitNode(context.identifier[0]),
        value: this.visitNode(context.value[0]),
      };
    }

    return { kind: 'DefaultEntry', value: this.visitNode(context.value[0]) };
  }

  varDeclaration(context: CstChildrenDictionary): types.VarDeclaration {
    return {
      kind: 'VarDeclaration',
      identifier: this.visitNode(context.identifier[0]),
      type: this.visitNode(context.type[0]),
      initialValue: this.visitNode(context.value[0]),
    };
  }

  visitNode(cstElement: CstElement) {
    if (!('name' in cstElement)) throw new Error('CstNode expected');
    return this.visit(cstElement);
  }

  visitNodes(cstElements: CstElement[]) {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    return cstElements.map(this.visitNode, this);
  }

  visitToken(cstElement: CstElement) {
    if (!('tokenType' in cstElement)) throw new Error('Token expected');
    return cstElement.image;
  }
}

export default new RGVisitor();
