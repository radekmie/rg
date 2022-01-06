import { creator } from '../utils';

/* eslint-disable prettier/prettier */
export const ConditionEq = creator<ConditionEq>('ConditionEq');
export type ConditionEq = { kind: 'ConditionEq'; lhs: Expression; rhs: Expression };

export const DomainDeclaration = creator<DomainDeclaration>('DomainDeclaration');
export type DomainDeclaration = { kind: 'DomainDeclaration'; identifier: string; elements: DomainElement[] };

export const DomainGenerator = creator<DomainGenerator>('DomainGenerator');
export type DomainGenerator = { kind: 'DomainGenerator'; identifier: string; args: string[]; values: DomainValues[] };

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

export const ExpressionSub = creator<ExpressionSub>('ExpressionSub');
export type ExpressionSub = { kind: 'ExpressionSub'; lhs: Expression; rhs: Expression };

export const FunctionCase = creator<FunctionCase>('FunctionCase');
export type FunctionCase = { kind: 'FunctionCase'; identifier: string; args: Pattern[]; body: Expression };

export const FunctionDeclaration = creator<FunctionDeclaration>('FunctionDeclaration');
export type FunctionDeclaration = { kind: 'FunctionDeclaration'; identifier: string; type: Type; cases: FunctionCase[] };

export const GameDeclaration = creator<GameDeclaration>('GameDeclaration');
export type GameDeclaration = { kind: 'GameDeclaration'; domains: DomainDeclaration[]; functions: FunctionDeclaration[] };

export const PatternConstructor = creator<PatternConstructor>('PatternConstructor');
export type PatternConstructor = { kind: 'PatternConstructor'; identifier: string; args: Pattern[] };

export const PatternLiteral = creator<PatternLiteral>('PatternLiteral');
export type PatternLiteral = { kind: 'PatternLiteral'; identifier: string };

export const PatternVariable = creator<PatternVariable>('PatternVariable');
export type PatternVariable = { kind: 'PatternVariable'; identifier: string };

export const PatternWildcard = creator<PatternWildcard>('PatternWildcard');
export type PatternWildcard = { kind: 'PatternWildcard' };

export const TypeFunction = creator<TypeFunction>('TypeFunction');
export type TypeFunction = { kind: 'TypeFunction'; lhs: Type; rhs: Type };

export const TypeName = creator<TypeName>('TypeName');
export type TypeName = { kind: 'TypeName'; identifier: string };

export const ValueConstructor = creator<ValueConstructor>('ValueConstructor');
export type ValueConstructor = { kind: 'ValueConstructor'; identifier: string; args: Value[] };

export const ValueElement = creator<ValueElement>('ValueElement');
export type ValueElement = { kind: 'ValueElement'; identifier: string };

export type Condition = ConditionEq;
export type DomainElement = DomainGenerator | DomainLiteral;
export type DomainValues = DomainRange | DomainSet;
export type Expression = ExpressionAdd | ExpressionConstructor | ExpressionIf | ExpressionLiteral | ExpressionSub;
export type Pattern = PatternConstructor | PatternLiteral | PatternVariable | PatternWildcard;
export type Value = ValueConstructor | ValueElement;
export type Type = TypeFunction | TypeName;
