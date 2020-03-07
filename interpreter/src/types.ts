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
  | { kind: 'assignment'; lhs: string; rhs: string }
  | { kind: 'condition'; inverted: boolean; lhs: string; rhs: string }
  | { kind: 'empty' }
  | { kind: 'reachability'; inverted: boolean; lhs: number; rhs: number }
  | { kind: 'switch'; player: null | string };

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
