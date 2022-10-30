import { creator } from '../../utils';

export type Action =
  | Assignment
  | Check
  | Comparison
  | Off
  | On
  | Shift
  | Switch;

export const Assignment = creator<Assignment>('Assignment');
export type Assignment = {
  kind: 'Assignment';
  variable: string;
  value: Value;
};

export const Atom = creator<Atom>('Atom');
export type Atom = {
  kind: 'Atom';
  content: Action | Rule;
  power: boolean;
};

export const Check = creator<Check>('Check');
export type Check = {
  kind: 'Check';
  negated: boolean;
  rule: Rule;
};

export const Comparison = creator<Comparison>('Comparison');
export type Comparison = {
  kind: 'Comparison';
  lhs: Value;
  rhs: Value;
  operator: '>' | '>=' | '==' | '!=' | '<=' | '<';
};

export const Edge = creator<Edge>('Edge');
export type Edge = {
  kind: 'Edge';
  label: string;
  node: string;
};

export const Expression = creator<Expression>('Expression');
export type Expression = {
  kind: 'Expression';
  lhs: Value;
  rhs: Value;
  operator: '+' | '-' | '*' | '/';
};

export const Game = creator<Game>('Game');
export type Game = {
  kind: 'Game';
  pieces: string[];
  variables: Variable[];
  players: Variable[];
  board: Node[];
  rules: Rule;
};

export const Node = creator<Node>('Node');
export type Node = {
  kind: 'Node';
  node: string;
  piece: string;
  edges: Edge[];
};

export const Off = creator<Off>('Off');
export type Off = {
  kind: 'Off';
  piece: string;
};

export const On = creator<On>('On');
export type On = {
  kind: 'On';
  pieces: string[];
};

export const Shift = creator<Shift>('Shift');
export type Shift = {
  kind: 'Shift';
  label: string;
};

export const Switch = creator<Switch>('Switch');
export type Switch = {
  kind: 'Switch';
  player: string | null;
};

export const Rule = creator<Rule>('Rule');
export type Rule = {
  kind: 'Rule';
  elements: Atom[][];
};

export type Value = Expression | number | string;

export const Variable = creator<Variable>('Variable');
export type Variable = {
  kind: 'Variable';
  name: string;
  bound: number;
};
