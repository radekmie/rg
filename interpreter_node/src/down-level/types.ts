import { creator } from '../utils';

export const AutomatonAssignment = creator<AutomatonAssignment>(
  'AutomatonAssignment',
);
export type AutomatonAssignment = {
  kind: 'AutomatonAssignment';
  identifier: string;
  accessors: Expression[];
  expression: Expression;
};

export const AutomatonBranch = creator<AutomatonBranch>('AutomatonBranch');
export type AutomatonBranch = {
  kind: 'AutomatonBranch';
  arms: AutomatonStatement[][];
};

export const AutomatonCall = creator<AutomatonCall>('AutomatonCall');
export type AutomatonCall = {
  kind: 'AutomatonCall';
  identifier: string;
  args: Expression[];
};

export const AutomatonFunction =
  creator<AutomatonFunction>('AutomatonFunction');
export type AutomatonFunction = {
  kind: 'AutomatonFunction';
  name: string;
  args: AutomatonFunctionArgument[];
  body: AutomatonStatement[];
};

export const AutomatonFunctionArgument = creator<AutomatonFunctionArgument>(
  'AutomatonFunctionArgument',
);
export type AutomatonFunctionArgument = {
  kind: 'AutomatonFunctionArgument';
  identifier: string;
  type: Type;
};

export const AutomatonLoop = creator<AutomatonLoop>('AutomatonLoop');
export type AutomatonLoop = {
  kind: 'AutomatonLoop';
  body: AutomatonStatement[];
};

export const AutomatonWhen = creator<AutomatonWhen>('AutomatonWhen');
export type AutomatonWhen = {
  kind: 'AutomatonWhen';
  expression: Expression;
  body: AutomatonStatement[];
};

export const AutomatonWhile = creator<AutomatonWhile>('AutomatonWhile');
export type AutomatonWhile = {
  kind: 'AutomatonWhile';
  expression: Expression;
  body: AutomatonStatement[];
};

export const DomainDeclaration =
  creator<DomainDeclaration>('DomainDeclaration');
export type DomainDeclaration = {
  kind: 'DomainDeclaration';
  identifier: string;
  elements: DomainElement[];
};

export const DomainGenerator = creator<DomainGenerator>('DomainGenerator');
export type DomainGenerator = {
  kind: 'DomainGenerator';
  identifier: string;
  args: string[];
  values: DomainValue[];
};

export const DomainLiteral = creator<DomainLiteral>('DomainLiteral');
export type DomainLiteral = { kind: 'DomainLiteral'; identifier: string };

export const DomainRange = creator<DomainRange>('DomainRange');
export type DomainRange = {
  kind: 'DomainRange';
  identifier: string;
  min: string;
  max: string;
};

export const DomainSet = creator<DomainSet>('DomainSet');
export type DomainSet = {
  kind: 'DomainSet';
  identifier: string;
  elements: string[];
};

export const ExpressionAccess = creator<ExpressionAccess>('ExpressionAccess');
export type ExpressionAccess = {
  kind: 'ExpressionAccess';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionAdd = creator<ExpressionAdd>('ExpressionAdd');
export type ExpressionAdd = {
  kind: 'ExpressionAdd';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionAnd = creator<ExpressionAnd>('ExpressionAnd');
export type ExpressionAnd = {
  kind: 'ExpressionAnd';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionCall = creator<ExpressionCall>('ExpressionCall');
export type ExpressionCall = {
  kind: 'ExpressionCall';
  expression: Expression;
  args: Expression[];
};

export const ExpressionConstructor = creator<ExpressionConstructor>(
  'ExpressionConstructor',
);
export type ExpressionConstructor = {
  kind: 'ExpressionConstructor';
  identifier: string;
  args: Expression[];
};

export const ExpressionEq = creator<ExpressionEq>('ExpressionEq');
export type ExpressionEq = {
  kind: 'ExpressionEq';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionGt = creator<ExpressionGt>('ExpressionGt');
export type ExpressionGt = {
  kind: 'ExpressionGt';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionGte = creator<ExpressionGte>('ExpressionGte');
export type ExpressionGte = {
  kind: 'ExpressionGte';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionIf = creator<ExpressionIf>('ExpressionIf');
export type ExpressionIf = {
  kind: 'ExpressionIf';
  cond: Expression;
  then: Expression;
  else: Expression;
};

export const ExpressionLiteral =
  creator<ExpressionLiteral>('ExpressionLiteral');
export type ExpressionLiteral = {
  kind: 'ExpressionLiteral';
  identifier: string;
};

export const ExpressionLt = creator<ExpressionLt>('ExpressionLt');
export type ExpressionLt = {
  kind: 'ExpressionLt';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionLte = creator<ExpressionLte>('ExpressionLte');
export type ExpressionLte = {
  kind: 'ExpressionLte';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionMap = creator<ExpressionMap>('ExpressionMap');
export type ExpressionMap = {
  kind: 'ExpressionMap';
  pattern: Pattern;
  expression: Expression;
  domains: DomainValue[];
};

export const ExpressionNe = creator<ExpressionNe>('ExpressionNe');
export type ExpressionNe = {
  kind: 'ExpressionNe';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionOr = creator<ExpressionOr>('ExpressionOr');
export type ExpressionOr = {
  kind: 'ExpressionOr';
  lhs: Expression;
  rhs: Expression;
};

export const ExpressionSub = creator<ExpressionSub>('ExpressionSub');
export type ExpressionSub = {
  kind: 'ExpressionSub';
  lhs: Expression;
  rhs: Expression;
};

export const FunctionCase = creator<FunctionCase>('FunctionCase');
export type FunctionCase = {
  kind: 'FunctionCase';
  identifier: string;
  args: Pattern[];
  body: Expression;
};

export const FunctionDeclaration = creator<FunctionDeclaration>(
  'FunctionDeclaration',
);
export type FunctionDeclaration = {
  kind: 'FunctionDeclaration';
  identifier: string;
  type: Type;
  cases: FunctionCase[];
};

export const GameDeclaration = creator<GameDeclaration>('GameDeclaration');
export type GameDeclaration = {
  kind: 'GameDeclaration';
  automaton: AutomatonFunction[];
  domains: DomainDeclaration[];
  functions: FunctionDeclaration[];
  variables: VariableDeclaration[];
};

export const PatternConstructor =
  creator<PatternConstructor>('PatternConstructor');
export type PatternConstructor = {
  kind: 'PatternConstructor';
  identifier: string;
  args: Pattern[];
};

export const PatternLiteral = creator<PatternLiteral>('PatternLiteral');
export type PatternLiteral = { kind: 'PatternLiteral'; identifier: string };

export const PatternVariable = creator<PatternVariable>('PatternVariable');
export type PatternVariable = { kind: 'PatternVariable'; identifier: string };

export const PatternWildcard = creator<PatternWildcard>('PatternWildcard');
export type PatternWildcard = { kind: 'PatternWildcard' };

export const TypeDeclaration = creator<TypeDeclaration>('TypeDeclaration');
export type TypeDeclaration = {
  kind: 'TypeDeclaration';
  identifier: string;
  type: Type;
};

export const TypeFunction = creator<TypeFunction>('TypeFunction');
export type TypeFunction = { kind: 'TypeFunction'; lhs: Type; rhs: Type };

export const TypeName = creator<TypeName>('TypeName');
export type TypeName = { kind: 'TypeName'; identifier: string };

export const ValueConstructor = creator<ValueConstructor>('ValueConstructor');
export type ValueConstructor = {
  kind: 'ValueConstructor';
  identifier: string;
  args: Value[];
};

export const ValueElement = creator<ValueElement>('ValueElement');
export type ValueElement = { kind: 'ValueElement'; identifier: string };

export const ValueMap = creator<ValueMap>('ValueMap');
export type ValueMap = { kind: 'ValueMap'; entries: ValueMapEntry[] };

export const ValueMapEntry = creator<ValueMapEntry>('ValueMapEntry');
export type ValueMapEntry = { kind: 'ValueMapEntry'; key: Value; value: Value };

export const VariableAssignment =
  creator<VariableAssignment>('VariableAssignment');
export type VariableAssignment = {
  kind: 'VariableAssignment';
  identifier: string;
  expression: Expression;
};

export const VariableDeclaration = creator<VariableDeclaration>(
  'VariableDeclaration',
);
export type VariableDeclaration = {
  kind: 'VariableDeclaration';
  identifier: string;
  type: Type;
  defaultValue: null | Expression;
};

export type AutomatonStatement =
  | AutomatonAssignment
  | AutomatonBranch
  | AutomatonCall
  | AutomatonWhen
  | AutomatonLoop
  | AutomatonWhile;
export type DomainElement = DomainGenerator | DomainLiteral;
export type DomainValue = DomainRange | DomainSet;
export type Expression =
  | ExpressionAccess
  | ExpressionAdd
  | ExpressionAnd
  | ExpressionCall
  | ExpressionConstructor
  | ExpressionEq
  | ExpressionGt
  | ExpressionGte
  | ExpressionIf
  | ExpressionLiteral
  | ExpressionLt
  | ExpressionLte
  | ExpressionMap
  | ExpressionNe
  | ExpressionOr
  | ExpressionSub;
export type Pattern =
  | PatternConstructor
  | PatternLiteral
  | PatternVariable
  | PatternWildcard;
export type Type = TypeFunction | TypeName;
export type Value = ValueConstructor | ValueElement | ValueMap;
