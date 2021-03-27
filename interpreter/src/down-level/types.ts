import { creator } from '../utils';

/* eslint-disable prettier/prettier */
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

export const GameDeclaration = creator<GameDeclaration>('GameDeclaration');
export type GameDeclaration = { kind: 'GameDeclaration'; domains: DomainDeclaration[] };

export type DomainElement = DomainGenerator | DomainLiteral;
export type DomainValues = DomainRange | DomainSet;
