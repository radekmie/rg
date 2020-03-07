export type Constant = {
  defaultValue: null | Value;
  type: Type;
  values: Record<string, Value>;
};

export type Domain = {
  values: Value[];
};

export type Edge = {
  a: number;
  b: number;
  label: EdgeLabel;
};

export type EdgeLabel =
  | { kind: 'assignment'; lhs: Expression; rhs: Expression }
  | { kind: 'condition'; inverted: boolean; lhs: Expression; rhs: Expression }
  | { kind: 'empty' }
  | { kind: 'reachability'; inverted: boolean; lhs: number; rhs: number }
  | { kind: 'switch'; player: null | string };

export type Expression =
  | { kind: 'constant-call'; name: string; argument: Expression }
  | { kind: 'value'; value: Value }
  | { kind: 'variable'; name: string }
  | { kind: 'variable-access'; name: string; key: Expression };

export type Game = {
  constants: Record<string, Constant>;
  domains: Record<string, Domain>;
  edges: Record<number, Edge[]>;
  variables: Record<string, Variable>;
};

export type State = {
  player: null | string;
  position: number;
  variables: Record<string, null | Value>;
};

export type Type =
  | { kind: 'arrow'; from: Type; to: Type }
  | { kind: 'domain'; name: string }
  | { kind: 'domain-inline'; values: Value[] };

export type Value =
  | { kind: 'map'; values: Record<string, Value> }
  | { kind: 'symbol'; value: string }
  | { kind: 'wildcard' };

export type Variable = {
  defaultValue: null | Value;
  type: Type;
  visibility: Value[];
};
