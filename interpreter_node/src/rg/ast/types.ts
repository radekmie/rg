import { creator } from '../../utils';

export const Access = creator<Access>('Access');
export type Access = { kind: 'Access'; lhs: Expression; rhs: Expression };

export const Any = creator<Any>('Any');
export type Any = {
  kind: 'Any';
  lhs: EdgeName;
  rhs: EdgeName;
};

export const Arrow = creator<Arrow>('Arrow');
export type Arrow = { kind: 'Arrow'; lhs: Type; rhs: Type };

export const Assignment = creator<Assignment>('Assignment');
export type Assignment = {
  kind: 'Assignment';
  lhs: Expression;
  rhs: Expression;
};

export const Binding = creator<Binding>('Binding');
export type Binding = { kind: 'Binding'; identifier: string; type: Type };

export const Cast = creator<Cast>('Cast');
export type Cast = { kind: 'Cast'; lhs: Type; rhs: Expression };

export const Comparison = creator<Comparison>('Comparison');
export type Comparison = {
  kind: 'Comparison';
  lhs: Expression;
  rhs: Expression;
  negated: boolean;
};

export const ConstantDeclaration = creator<ConstantDeclaration>(
  'ConstantDeclaration',
);
export type ConstantDeclaration = {
  kind: 'ConstantDeclaration';
  identifier: string;
  type: Type;
  value: Value;
};

export const EdgeDeclaration = creator<EdgeDeclaration>('EdgeDeclaration');
export type EdgeDeclaration = {
  kind: 'EdgeDeclaration';
  lhs: EdgeName;
  rhs: EdgeName;
  label: EdgeLabel;
};

export const EdgeName = creator<EdgeName>('EdgeName');
export type EdgeName = { kind: 'EdgeName'; parts: EdgeNamePart[] };

export const Element = creator<Element>('Element');
export type Element = { kind: 'Element'; identifier: string };

export const GameDeclaration = creator<GameDeclaration>('GameDeclaration');
export type GameDeclaration = {
  kind: 'GameDeclaration';
  constants: ConstantDeclaration[];
  edges: EdgeDeclaration[];
  pragmas: Pragma[];
  types: TypeDeclaration[];
  variables: VariableDeclaration[];
};

export const Literal = creator<Literal>('Literal');
export type Literal = { kind: 'Literal'; identifier: string };

export const Map = creator<Map>('Map');
export type Map = { kind: 'Map'; entries: ValueEntry[] };

export const PragmaAny = creator<PragmaAny>('Any');
export type PragmaAny = { kind: 'Any'; edgeName: EdgeName };

export const PragmaDisjoint = creator<PragmaDisjoint>('Disjoint');
export type PragmaDisjoint = { kind: 'Disjoint'; edgeName: EdgeName };

export const PragmaMultiAny = creator<PragmaMultiAny>('MultiAny');
export type PragmaMultiAny = { kind: 'MultiAny'; edgeName: EdgeName };

export const PragmaUnique = creator<PragmaUnique>('Unique');
export type PragmaUnique = { kind: 'Unique'; edgeName: EdgeName };

export const Reachability = creator<Reachability>('Reachability');
export type Reachability = {
  kind: 'Reachability';
  lhs: EdgeName;
  rhs: EdgeName;
  negated: boolean;
};

export const Reference = creator<Reference>('Reference');
export type Reference = { kind: 'Reference'; identifier: string };

export const Set = creator<Set>('Set');
export type Set = { kind: 'Set'; identifiers: string[] };

export const Skip = creator<Skip>('Skip');
export type Skip = { kind: 'Skip' };

export const Tag = creator<Tag>('Tag');
export type Tag = { kind: 'Tag'; symbols: string[] };

export const TypeDeclaration = creator<TypeDeclaration>('TypeDeclaration');
export type TypeDeclaration = {
  kind: 'TypeDeclaration';
  identifier: string;
  type: Type;
};

export const TypeReference = creator<TypeReference>('TypeReference');
export type TypeReference = { kind: 'TypeReference'; identifier: string };

export const ValueEntry = creator<ValueEntry>('ValueEntry');
export type ValueEntry = {
  kind: 'ValueEntry';
  identifier: null | string;
  value: Value;
};

export const VariableDeclaration = creator<VariableDeclaration>(
  'VariableDeclaration',
);
export type VariableDeclaration = {
  kind: 'VariableDeclaration';
  identifier: string;
  type: Type;
  defaultValue: Value;
};

export type EdgeLabel =
  | Any
  | Assignment
  | Comparison
  | Reachability
  | Skip
  | Tag;
export type EdgeNamePart = Binding | Literal;
export type Expression = Access | Cast | Reference;
export type Pragma = PragmaAny | PragmaDisjoint | PragmaMultiAny | PragmaUnique;
export type Type = Arrow | Set | TypeReference;
export type Value = Map | Element;
