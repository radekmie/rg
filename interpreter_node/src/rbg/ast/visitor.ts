import {
  CstChildrenDictionary as Context,
  CstElement,
  CstNode,
} from 'chevrotain';

import * as utils from '../../utils';
import * as cst from '../cst';
import * as ast from './types';

class Visitor extends cst.parser.getBaseCstVisitorConstructor() {
  constructor() {
    super();
    this.validateVisitor();
  }

  Action(context: Context): ast.Action {
    if ('Assignment' in context) {
      return this.visitNode(context.Assignment[0]);
    }

    if ('Comparison' in context) {
      return this.visitNode(context.Comparison[0]);
    }

    if ('MoveCheck' in context) {
      return this.visitNode(context.MoveCheck[0]);
    }

    if ('Off' in context) {
      return this.visitNode(context.Off[0]);
    }

    if ('On' in context) {
      return this.visitNode(context.On[0]);
    }

    if ('Shift' in context) {
      return this.visitNode(context.Shift[0]);
    }

    if ('Switch' in context) {
      return this.visitNode(context.Switch[0]);
    }

    utils.assert(false, 'Unknown context in Action.');
  }

  Assignment(context: Context): ast.Assignment {
    return ast.Assignment({
      variable: this.visitNode(context.VariableName[0]),
      rvalue: this.visitNode(context.RValue[0]),
    });
  }

  BoardSection(context: Context): ast.Node[] {
    return this.visitNodes(context.Node);
  }

  BoundedVariable(context: Context): ast.Variable {
    return ast.Variable({
      name: this.visitNode(context.VariableName[0]),
      bound: this.visitNode(context.VariableBound[0]),
    });
  }

  Comparison(context: Context): ast.Comparison {
    return ast.Comparison({
      lhs: this.visitNode(context.RValue[0]),
      rhs: this.visitNode(context.RValue[1]),
      operator: this.visitNode(context.ComparisonOperator[0]),
    });
  }

  ComparisonOperator(context: Context): '>' | '>=' | '==' | '!=' | '<=' | '<' {
    if ('Gt' in context) {
      return '>';
    }

    if ('GtEqual' in context) {
      return '>=';
    }

    if ('EqualEqual' in context) {
      return '==';
    }

    if ('BangEqual' in context) {
      return '!=';
    }

    if ('LtEqual' in context) {
      return '<=';
    }

    if ('Lt' in context) {
      return '<';
    }

    utils.assert(false, 'Unknown context in ComparisonOperator.');
  }

  Edge(context: Context): ast.Edge {
    return ast.Edge({
      label: this.visitNode(context.Label[0]),
      node: this.visitNode(context.NodeName[0]),
    });
  }

  GameDescription(context: Context): ast.Game {
    return ast.Game({
      pieces: this.visitNode(context.PiecesSection[0]),
      variables: this.visitNode(context.VariablesSection[0]),
      players: this.visitNode(context.PlayersSection[0]),
      board: this.visitNode(context.BoardSection[0]),
      rules: this.visitNode(context.RulesSection[0]),
    });
  }

  Label(context: Context): string {
    return this.visitToken(context.Ident[0]);
  }

  MoveCheck(context: Context): ast.Check {
    return ast.Check({
      negated: 'BraceLeftBang' in context,
      rule: this.visitNode(context.Rule[0]),
    });
  }

  MultiplicationElement(context: Context): ast.RValue {
    if ('VariableName' in context) {
      return this.visitNode(context.VariableName[0]);
    }

    if ('Nat' in context) {
      return parseInt(this.visitToken(context.Nat[0]));
    }

    if ('Sum' in context) {
      return this.visitNode(context.Sum[0]);
    }

    utils.assert(false, 'Unknown context in MultiplicationElement.');
  }

  NextMultiplicationElements(context: Context): ['*' | '/', ast.RValue] {
    return [
      'Star' in context ? '*' : '/',
      this.visitNode(context.MultiplicationElement[0]),
    ];
  }

  NextSumElements(context: Context): ['+' | '-', ast.RValue] {
    return [
      'Plus' in context ? '+' : '-',
      this.visitNode(context.SumElement[0]),
    ];
  }

  Node(context: Context): ast.Node {
    return ast.Node({
      node: this.visitNode(context.NodeName[0]),
      piece: this.visitNode(context.PieceName[0]),
      edges: this.visitNodes(context.Edge),
    });
  }

  NodeName(context: Context): string {
    return this.visitToken(context.Ident[0]);
  }

  Off(context: Context): ast.Off {
    return ast.Off({ piece: this.visitNode(context.PieceName[0]) });
  }

  On(context: Context): ast.On {
    return ast.On({ pieces: this.visitNodes(context.PieceName) });
  }

  PieceName(context: Context): string {
    return this.visitToken(context.Ident[0]);
  }

  PiecesSection(context: Context): string[] {
    return this.visitNodes(context.PieceName);
  }

  PlayerName(context: Context): string {
    return this.visitToken(context.Ident[0]);
  }

  PlayersSection(context: Context): ast.Variable[] {
    return this.visitNodes(context.BoundedVariable);
  }

  PotentialPower(context: Context): boolean {
    return 'Star' in context;
  }

  RValue(context: Context): ast.RValue {
    return this.visitNode(context.Sum[0]);
  }

  Rule(context: Context): ast.Rule {
    return this.visitNode(context.RuleSum[0]);
  }

  RuleConcatenationElement(context: Context): ast.Atom {
    return ast.Atom({
      content:
        'Action' in context
          ? this.visitNode(context.Action[0])
          : this.visitNode(context.RuleSum[0]),
      power: this.visitNode(context.PotentialPower[0]),
    });
  }

  RuleSum(context: Context): ast.Rule {
    return ast.Rule({ elements: this.visitNodes(context.RuleSumElement) });
  }

  RuleSumElement(context: Context): ast.Atom[] {
    return this.visitNodes(context.RuleConcatenationElement);
  }

  RulesSection(context: Context): ast.Rule {
    return this.visitNode(context.Rule[0]);
  }

  Shift(context: Context): ast.Shift {
    return ast.Shift({ label: this.visitNode(context.Label[0]) });
  }

  Sum(context: Context): ast.RValue {
    return this.visitNodes(context.NextSumElements).reduce(
      (lhs, [operator, rhs]) => ast.Expression({ lhs, rhs, operator }),
      this.visitNode(context.SumElement[0]),
    );
  }

  SumElement(context: Context): ast.RValue {
    return this.visitNodes(context.NextMultiplicationElements).reduce(
      (lhs, [operator, rhs]) => ast.Expression({ lhs, rhs, operator }),
      this.visitNode(context.MultiplicationElement[0]),
    );
  }

  Switch(context: Context): ast.Switch {
    return ast.Switch({
      player:
        'PlayerName' in context ? this.visitNode(context.PlayerName[0]) : null,
    });
  }

  VariableBound(context: Context): number {
    return parseInt(this.visitToken(context.Nat[0]));
  }

  VariableName(context: Context): string {
    return this.visitToken(context.Ident[0]);
  }

  VariablesSection(context: Context): ast.Variable[] {
    return this.visitNodes(context.BoundedVariable);
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
export function visit(cstNode: CstNode): ast.Game {
  return visitor.visitNode(cstNode);
}
