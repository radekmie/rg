import { CstChildrenDictionary as Context, CstElement } from 'chevrotain';

import parser from './parser';
import * as ast from './types';

class HLVisitor extends parser.getBaseCstVisitorConstructor() {
  constructor() {
    super();
    this.validateVisitor();
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

  GameDeclaration(context: Context): ast.GameDeclaration {
    return ast.GameDeclaration({
      domains: this.visitNodes(context.DomainDeclaration),
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
