import { EdgeLabel, Expression, Game, Type, Value } from './types';
import { readFileSync } from 'fs';

export function parse(path: string) {
  const source = readFileSync(path, { encoding: 'utf-8' });
  const game: Game = {
    constants: Object.create(null),
    domains: Object.create(null),
    edges: Object.create(null),
    variables: Object.create(null),
  };

  parseConstants(game, source);
  parseDomains(game, source);
  parseVariables(game, source);
  parseEdges(game, source);

  return game;
}

export function parseConstantMapping(source: string) {
  const valuePattern = /.+?\((.+?)\)\s*=\s*(.*?)\s*,/gs;
  const values: Record<string, Value> = Object.create(null);
  for (const [, argument, value] of source.matchAll(valuePattern))
    values[argument] = parseValue(value);
  return values;
}

export function parseConstants(game: Game, source: string) {
  const constantPattern = /constant\s+(.+?)\s*:(.+?),\s*default\s*=\s*(.+?)\s*,\s*(.+?);/gs;
  for (const [, name, type, defaultValue, values] of source.matchAll(
    constantPattern,
  )) {
    game.constants[name] = {
      defaultValue: defaultValue ? parseValue(defaultValue) : null,
      type: parseType(type),
      values: parseConstantMapping(values + ','),
    };
  }
}

export function parseDomainValues(source: string): Value[] {
  const setPattern = /^\s*\{(.+?)\}\s*$/s;
  const setMatch = setPattern.exec(source);
  if (setMatch) {
    const [, values] = setMatch;
    return values.split(/\s*,\s*/).map(parseValue);
  }

  throw new Error(`Invalid domain values: "${source}"`);
}

export function parseDomains(game: Game, source: string) {
  const domainPattern = /domain\s+(.+?)\s*=(.+?);/gs;
  for (const [, name, values] of source.matchAll(domainPattern))
    game.domains[name] = { values: parseDomainValues(values) };
}

export function parseEdgeLabel(game: Game, source: string): EdgeLabel {
  const emptyPattern = /^$/;
  const emptyMatch = emptyPattern.exec(source);
  if (emptyMatch) return { kind: 'empty' };

  const assignmentPattern = /^\[(.+?)\s*=\s*(.+?)\]$/s;
  const assignmentMatch = assignmentPattern.exec(source);
  if (assignmentMatch) {
    return {
      kind: 'assignment',
      lhs: parseExpression(game, assignmentMatch[1]),
      rhs: parseExpression(game, assignmentMatch[2]),
    };
  }

  const conditionPattern = /^\{([!?])\s*(.+?)\s*==\s*(.+?)\}$/s;
  const conditionMatch = conditionPattern.exec(source);
  if (conditionMatch) {
    return {
      kind: 'condition',
      inverted: conditionMatch[1] === '!',
      lhs: parseExpression(game, conditionMatch[2]),
      rhs: parseExpression(game, conditionMatch[3]),
    };
  }

  const reachabilityPattern = /^\{([!?])\s*(.+?)\s*->\s*(.+?)\}$/s;
  const reachabilityMatch = reachabilityPattern.exec(source);
  if (reachabilityMatch) {
    return {
      kind: 'reachability',
      inverted: reachabilityMatch[1] === '!',
      lhs: +reachabilityMatch[2],
      rhs: +reachabilityMatch[3],
    };
  }

  const switchPattern = /^->(.*?)$/s;
  const switchMatch = switchPattern.exec(source);
  if (switchMatch) {
    return {
      kind: 'switch',
      player: switchMatch[1] === '>' ? null : switchMatch[1],
    };
  }

  throw new Error(`Invalid edge: "${source}"`);
}

export function parseEdges(game: Game, source: string) {
  const edgePattern = /(\d+)\s*,(\d+)\s*:\s*(.*?);/gs;
  for (const [, a, b, label] of source.matchAll(edgePattern)) {
    if (!(+a in game.edges)) game.edges[+a] = [];
    game.edges[+a].push({ a: +a, b: +b, label: parseEdgeLabel(game, label) });
  }
}

export function parseExpression(game: Game, source: string): Expression {
  const accessPattern = /^(.+?)\[(.+?)\]$/s;
  const accessMatch = accessPattern.exec(source);
  if (accessMatch) {
    const [, name, key] = accessMatch;
    return {
      kind: 'variable-access',
      name,
      key: parseExpression(game, key),
    };
  }

  const constantCallPattern = /^(.+?)\((.+?)\)$/s;
  const constantCallMatch = constantCallPattern.exec(source);
  if (constantCallMatch) {
    const [, name, argument] = constantCallMatch;
    return {
      kind: 'constant-call',
      name,
      argument: parseExpression(game, argument),
    };
  }

  if (source in game.variables) return { kind: 'variable', name: source };

  return { kind: 'value', value: parseValue(source) };
}

export function parseType(source: string): Type {
  const arrowPattern = /^\s*(.+?)\s*->\s*(.+?)\s*$/s;
  const arrowMatch = arrowPattern.exec(source);
  if (arrowMatch) {
    return {
      kind: 'arrow',
      from: parseType(arrowMatch[1]),
      to: parseType(arrowMatch[1]),
    };
  }

  const setPattern = /^\s*\{(.+?)\}\s*$/s;
  const setMatch = setPattern.exec(source);
  if (setMatch)
    return { kind: 'domain-inline', values: parseDomainValues(source) };

  return { kind: 'domain', name: source.trim() };
}

export function parseValue(source: string): Value {
  if (source === '*') return { kind: 'wildcard' };

  const mapPattern = /^\s*\{(.+?)\}\s*$/s;
  const mapMatch = mapPattern.exec(source);
  if (mapMatch)
    return { kind: 'map', values: parseValueMapEntries(mapMatch[1] + ',') };

  return { kind: 'symbol', value: source.trim() };
}

export function parseValueMapEntries(source: string) {
  const entryPattern = /\s*(.+?)\s*=\s*(.+?),/gs;
  const entries: Record<string, Value> = Object.create(null);
  for (const [, name, value] of source.matchAll(entryPattern))
    entries[name] = parseValue(value);
  return entries;
}

export function parseVariables(game: Game, source: string) {
  const variablePattern = /var\s+(.+?)\s*:(.+?),\s*(?:default\s*=\s*(.*?)\s*,\s*)?visible\s*=\s*(.+?);/gs;
  for (const [, name, type, defaultValue, visibility] of source.matchAll(
    variablePattern,
  )) {
    game.variables[name] = {
      defaultValue: defaultValue ? parseValue(defaultValue) : null,
      type: parseType(type),
      visibility: parseDomainValues(visibility),
    };
  }
}
