export type ConstDeclaration = {
  kind: 'ConstDeclaration';
  identifier: Identifier;
  type: Type;
  value: Value;
};

export type EdgeDeclaration = {
  kind: 'EdgeDeclaration';
  lhs: Identifier;
  rhs: Identifier;
  label: EdgeLabel;
};

export type EdgeLabel =
  | { kind: 'Assignment'; lhs: Expression; rhs: Expression }
  | { kind: 'Comparison'; lhs: Expression; rhs: Expression; negated: boolean }
  | {
      kind: 'Reachability';
      lhs: Identifier;
      rhs: Identifier;
      mode: 'not' | 'rev';
    }
  | { kind: 'Skip' };

export type Expression =
  | { kind: 'Access'; lhs: Identifier; rhs: Expression }
  | { kind: 'Cast'; lhs: Identifier; rhs: Expression }
  | { kind: 'VarReference'; identifier: Identifier };

export type Game = {
  kind: 'Game';
  consts: ConstDeclaration[];
  edges: EdgeDeclaration[];
  types: TypeDeclaration[];
  vars: VarDeclaration[];
};

export type Identifier = {
  kind: 'Identifier';
  identifier: string;
};

export type Type =
  | { kind: 'Arrow'; lhs: Identifier; rhs: Type }
  | { kind: 'Set'; identifiers: Identifier[] }
  | { kind: 'TypeReference'; identifier: Identifier };

export type TypeDeclaration = {
  kind: 'TypeDeclaration';
  identifier: Identifier;
  type: Type;
};

export type Value =
  | { kind: 'Map'; entries: ValueEntry[] }
  | { kind: 'Reference'; identifier: Identifier };

export type ValueEntry =
  | { kind: 'DefaultEntry'; value: Value }
  | { kind: 'NamedEntry'; identifier: Identifier; value: Value };

export type VarDeclaration = {
  kind: 'VarDeclaration';
  identifier: Identifier;
  type: Type;
  initialValue: Value;
};
