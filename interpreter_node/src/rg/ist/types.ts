import { creator } from '../../utils';

export const Access = creator<Access>('Access');
export type Access = { kind: 'Access'; lhs: Expression; rhs: Expression };

export const Arrow = creator<Arrow>('Arrow');
export type Arrow = { kind: 'Arrow'; lhs: Type; rhs: Type };

export const Assignment = creator<Assignment>('Assignment');
export type Assignment = {
  kind: 'Assignment';
  lhs: Expression;
  rhs: Expression;
};

export const Comparison = creator<Comparison>('Comparison');
export type Comparison = {
  kind: 'Comparison';
  lhs: Expression;
  rhs: Expression;
  negated: boolean;
};

export const ConstantReference =
  creator<ConstantReference>('ConstantReference');
export type ConstantReference = {
  kind: 'ConstantReference';
  identifier: string;
};

export const Distinct = creator<Distinct>('Distinct');
export type Distinct = { kind: 'Distinct'; edgeName: string };

export const Edge = creator<Edge>('Edge');
export type Edge = {
  kind: 'Edge';
  label: EdgeLabel;
  next: string;
};

export const Element = creator<Element>('Element');
export type Element = { kind: 'Element'; value: string };

export const Game = creator<Game>('Game');
export type Game = {
  kind: 'Game';
  constants: Record<string, Value>;
  edges: Record<string, Edge[]>;
  pragmas: Pragma[];
  types: Record<string, Type>;
  variables: Record<string, Variable>;
};

export const Literal = creator<Literal>('Literal');
export type Literal = { kind: 'Literal'; value: Value };

export const Map = creator<Map>('Map');
export type Map = {
  kind: 'Map';
  defaultValue: Value;
  values: Record<string, Value>;
};

export const Reachability = creator<Reachability>('Reachability');
export type Reachability = {
  kind: 'Reachability';
  lhs: string;
  rhs: string;
  negated: boolean;
};

export const Set = creator<Set>('Set');
export type Set = { kind: 'Set'; values: Value[] };

export const Skip = creator<Skip>('Skip');
export type Skip = { kind: 'Skip' };

export const State = creator<State>('State');
export type State = {
  kind: 'State';
  position: string;
  values: Record<string, Value>;
};

export const Variable = creator<Variable>('Variable');
export type Variable = { kind: 'Variable'; type: Type; defaultValue: Value };

export const VariableReference =
  creator<VariableReference>('VariableReference');
export type VariableReference = {
  kind: 'VariableReference';
  identifier: string;
};

export type EdgeLabel = Assignment | Comparison | Reachability | Skip;
export type Expression =
  | Access
  | ConstantReference
  | Literal
  | VariableReference;
export type Pragma = Distinct;
export type Type = Arrow | Set;
export type Value = Map | Element;
