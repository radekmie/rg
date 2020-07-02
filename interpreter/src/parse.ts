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
  const mapping: [Value, Value][] = [];
  for (const [, argument, value] of source.matchAll(valuePattern))
    mapping.push([parseValue(argument), parseValue(value)]);
  return mapping;
}

export function parseConstants(game: Game, source: string) {
  const constantPattern = /constant\s+(.+?)\s*:(.+?),\s*default\s*=\s*(.+?)\s*,\s*(.+?);/gs;
  for (const [, name, type, defaultValue, values] of source.matchAll(
    constantPattern,
  )) {
    game.constants[name] = {
      defaultValue: defaultValue === '' ? null : parseValue(defaultValue),
      mapping: parseConstantMapping(values + ','),
      type: parseType(type),
    };
  }
}

const domainsCache: Record<string, Value[]> = Object.create(null);
export function parseDomainValues(source: string): Value[] {
  source = source.replace(/\s+/g, '');

  if (source in domainsCache) return domainsCache[source];

  const setPattern = /^\s*\{(.+?)\}\s*$/s;
  const setMatch = setPattern.exec(source);
  if (setMatch !== null) {
    const [, values] = setMatch;
    return (domainsCache[source] = values.split(/\s*,\s*/).map(parseValue));
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
  if (emptyMatch !== null) return { kind: 'empty' };

  const assignmentPattern = /^\[(.+?)\s*=\s*(.+?)\]$/s;
  const assignmentMatch = assignmentPattern.exec(source);
  if (assignmentMatch !== null) {
    return {
      kind: 'assignment',
      lhs: parseExpression(game, assignmentMatch[1]),
      rhs: parseExpression(game, assignmentMatch[2]),
    };
  }

  const conditionPattern = /^\{([!?])\s*(.+?)\s*==\s*(.+?)\}$/s;
  const conditionMatch = conditionPattern.exec(source);
  if (conditionMatch !== null) {
    return {
      kind: 'condition',
      inverted: conditionMatch[1] === '!',
      lhs: parseExpression(game, conditionMatch[2]),
      rhs: parseExpression(game, conditionMatch[3]),
    };
  }

  const reachabilityPattern = /^\{([!?])\s*(.+?)\s*->\s*(.+?)\}$/s;
  const reachabilityMatch = reachabilityPattern.exec(source);
  if (reachabilityMatch !== null) {
    return {
      kind: 'reachability',
      inverted: reachabilityMatch[1] === '!',
      lhs: +reachabilityMatch[2],
      rhs: +reachabilityMatch[3],
    };
  }

  const switchPattern = /^->(.*?)$/s;
  const switchMatch = switchPattern.exec(source);
  if (switchMatch !== null) {
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
  if (accessMatch !== null) {
    const [, name, key] = accessMatch;
    return {
      kind: 'variable-access',
      name,
      key: parseExpression(game, key),
    };
  }

  const constantCallPattern = /^(.+?)\((.+?)\)$/s;
  const constantCallMatch = constantCallPattern.exec(source);
  if (constantCallMatch !== null) {
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

const typesCache: Record<string, Type> = Object.create(null);
export function parseType(source: string): Type {
  source = source.replace(/\s+/g, '');

  if (source in typesCache) return typesCache[source];

  const arrowPattern = /^(.+?)->(.+?)$/s;
  const arrowMatch = arrowPattern.exec(source);
  if (arrowMatch !== null) {
    return (typesCache[source] = {
      kind: 'arrow',
      from: parseType(arrowMatch[1]),
      to: parseType(arrowMatch[2]),
    });
  }

  const setPattern = /^\{(.+?)\}$/s;
  const setMatch = setPattern.exec(source);
  if (setMatch !== null) {
    return (typesCache[source] = {
      kind: 'domain-inline',
      values: parseDomainValues(source),
    });
  }

  return (typesCache[source] = { kind: 'domain', name: source });
}

const symbolsCache: Record<string, Value> = Object.create(null);
export function parseValue(source: string): Value {
  source = source.replace(/\s+/g, '');

  if (source in symbolsCache) return symbolsCache[source];
  if (source === '*') return (symbolsCache[source] = { kind: 'wildcard' });

  const mapPattern = /^\{(.+?)\}$/s;
  const mapMatch = mapPattern.exec(source);
  if (mapMatch !== null) {
    return (symbolsCache[source] = {
      kind: 'map',
      values: parseValueMapEntries(mapMatch[1] + ','),
    });
  }

  return (symbolsCache[source] = { kind: 'symbol', value: source });
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
  for (const [, name, type, defaultValue = '', visibility] of source.matchAll(
    variablePattern,
  )) {
    game.variables[name] = {
      defaultValue: defaultValue === '' ? null : parseValue(defaultValue),
      type: parseType(type),
      visibility: parseDomainValues(visibility),
    };
  }
}
