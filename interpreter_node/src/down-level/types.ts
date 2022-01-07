import { creator } from '../utils';

/* eslint-disable prettier/prettier */
export const ConditionGt = creator<ConditionGt>('ConditionGt');
export type ConditionGt = { kind: 'ConditionGt'; lhs: Expression; rhs: Expression };

export const ConditionEq = creator<ConditionEq>('ConditionEq');
export type ConditionEq = { kind: 'ConditionEq'; lhs: Expression; rhs: Expression };

export const ConditionLt = creator<ConditionLt>('ConditionLt');
export type ConditionLt = { kind: 'ConditionLt'; lhs: Expression; rhs: Expression };

export const DomainDeclaration = creator<DomainDeclaration>('DomainDeclaration');
export type DomainDeclaration = { kind: 'DomainDeclaration'; identifier: string; elements: DomainElement[] };

export const DomainGenerator = creator<DomainGenerator>('DomainGenerator');
export type DomainGenerator = { kind: 'DomainGenerator'; identifier: string; args: string[]; values: DomainValue[] };

export const DomainLiteral = creator<DomainLiteral>('DomainLiteral');
export type DomainLiteral = { kind: 'DomainLiteral'; identifier: string };

export const DomainRange = creator<DomainRange>('DomainRange');
export type DomainRange = { kind: 'DomainRange'; identifier: string; min: string; max: string };

export const DomainSet = creator<DomainSet>('DomainSet');
export type DomainSet = { kind: 'DomainSet'; identifier: string; elements: string[] };

export const ExpressionAdd = creator<ExpressionAdd>('ExpressionAdd');
export type ExpressionAdd = { kind: 'ExpressionAdd'; lhs: Expression; rhs: Expression };

export const ExpressionConstructor = creator<ExpressionConstructor>('ExpressionConstructor');
export type ExpressionConstructor = { kind: 'ExpressionConstructor'; identifier: string; args: Expression[] };

export const ExpressionIf = creator<ExpressionIf>('ExpressionIf');
export type ExpressionIf = { kind: 'ExpressionIf'; cond: Condition; then: Expression; else: Expression };

export const ExpressionLiteral = creator<ExpressionLiteral>('ExpressionLiteral');
export type ExpressionLiteral = { kind: 'ExpressionLiteral'; identifier: string };

export const ExpressionMap = creator<ExpressionMap>('ExpressionMap');
export type ExpressionMap = { kind: 'ExpressionMap'; pattern: Pattern; expression: Expression; domains: DomainValue[] };

export const ExpressionSub = creator<ExpressionSub>('ExpressionSub');
export type ExpressionSub = { kind: 'ExpressionSub'; lhs: Expression; rhs: Expression };

export const FunctionCase = creator<FunctionCase>('FunctionCase');
export type FunctionCase = { kind: 'FunctionCase'; identifier: string; args: Pattern[]; body: Expression };

export const FunctionDeclaration = creator<FunctionDeclaration>('FunctionDeclaration');
export type FunctionDeclaration = { kind: 'FunctionDeclaration'; identifier: string; type: Type; cases: FunctionCase[] };

export const GameDeclaration = creator<GameDeclaration>('GameDeclaration');
export type GameDeclaration = { kind: 'GameDeclaration'; domains: DomainDeclaration[]; functions: FunctionDeclaration[]; variables: VariableDeclaration[] };

export const PatternConstructor = creator<PatternConstructor>('PatternConstructor');
export type PatternConstructor = { kind: 'PatternConstructor'; identifier: string; args: Pattern[] };

export const PatternLiteral = creator<PatternLiteral>('PatternLiteral');
export type PatternLiteral = { kind: 'PatternLiteral'; identifier: string };

export const PatternVariable = creator<PatternVariable>('PatternVariable');
export type PatternVariable = { kind: 'PatternVariable'; identifier: string };

export const PatternWildcard = creator<PatternWildcard>('PatternWildcard');
export type PatternWildcard = { kind: 'PatternWildcard' };

export const TypeDeclaration = creator<TypeDeclaration>('TypeDeclaration');
export type TypeDeclaration = { kind: 'TypeDeclaration'; identifier: string; type: Type };

export const TypeFunction = creator<TypeFunction>('TypeFunction');
export type TypeFunction = { kind: 'TypeFunction'; lhs: Type; rhs: Type };

export const TypeName = creator<TypeName>('TypeName');
export type TypeName = { kind: 'TypeName'; identifier: string };

export const ValueConstructor = creator<ValueConstructor>('ValueConstructor');
export type ValueConstructor = { kind: 'ValueConstructor'; identifier: string; args: Value[] };

export const ValueElement = creator<ValueElement>('ValueElement');
export type ValueElement = { kind: 'ValueElement'; identifier: string };

export const ValueMap = creator<ValueMap>('ValueMap');
export type ValueMap = { kind: 'ValueMap'; entries: ValueMapEntry[] };

export const ValueMapEntry = creator<ValueMapEntry>('ValueMapEntry');
export type ValueMapEntry = { kind: 'ValueMapEntry'; key: Value; value: Value };

export const VariableAssignment = creator<VariableAssignment>('VariableAssignment');
export type VariableAssignment = { kind: 'VariableAssignment'; identifier: string; expression: Expression };

export const VariableDeclaration = creator<VariableDeclaration>('VariableDeclaration');
export type VariableDeclaration = { kind: 'VariableDeclaration'; identifier: string; type: Type; defaultValue: null | Expression };

export type Condition = ConditionGt | ConditionEq | ConditionLt;
export type DomainElement = DomainGenerator | DomainLiteral;
export type DomainValue = DomainRange | DomainSet;
export type Expression = ExpressionAdd | ExpressionConstructor | ExpressionIf | ExpressionLiteral | ExpressionMap | ExpressionSub;
export type Pattern = PatternConstructor | PatternLiteral | PatternVariable | PatternWildcard;
export type Value = ValueConstructor | ValueElement | ValueMap;
export type Type = TypeFunction | TypeName;
