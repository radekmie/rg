import {
  Constant,
  Domain,
  Edge,
  EdgeLabel,
  Expression,
  Game,
  Type,
  Value,
  Variable,
} from './types';
import { readFileSync } from 'fs';

export function parse(path: string): Game {
  const source = readFileSync(path, { encoding: 'utf-8' });

  const constants = parseConstants(source);
  const domains = parseDomains(source);
  const variables = parseVariables(source);
  const edges = parseEdges(source, variables);
  return { constants, domains, edges, variables };
}

export function parseConstantMapping(source: string) {
  const valuePattern = /.+?\((.+?)\)\s*=\s*(.*?)\s*,/gs;
  const values: Record<string, Value> = Object.create(null);
  for (const [, argument, value] of source.matchAll(valuePattern))
    values[argument] = parseValue(value);
  return values;
}

export function parseConstants(source: string) {
  const constantPattern = /constant\s+(.+?)\s*:(.+?),\s*default\s*=\s*(.+?)\s*,\s*(.+?);/gs;
  const constants: Record<string, Constant> = Object.create(null);
  for (const [, name, type, defaultValue, values] of source.matchAll(
    constantPattern,
  ))
    constants[name] = {
      defaultValue: defaultValue ? parseValue(defaultValue) : null,
      type: parseType(type),
      values: parseConstantMapping(values + ','),
    };
  return constants;
}

export function parseDomainValues(source: string): Value[] {
  const setPattern = /^\s*\{(.+?)\}\s*$/s;
  const setMatch = setPattern.exec(source);
  if (setMatch) return setMatch[1].split(/\s*,\s*/).map(parseValue);

  throw new Error(`Invalid domain values: "${source}"`);
}

export function parseDomains(source: string) {
  const domainPattern = /domain\s+(.+?)\s*=(.+?);/gs;
  const domains: Record<string, Domain> = Object.create(null);
  for (const [, name, values] of source.matchAll(domainPattern))
    domains[name] = { values: parseDomainValues(values) };
  return domains;
}

export function parseEdgeLabel(
  source: string,
  variables: Record<string, Variable>,
): EdgeLabel {
  const emptyPattern = /^$/;
  const emptyMatch = emptyPattern.exec(source);
  if (emptyMatch) return { kind: 'empty' };

  const assignmentPattern = /^\[(.+?)\s*=\s*(.+?)\]$/s;
  const assignmentMatch = assignmentPattern.exec(source);
  if (assignmentMatch)
    return {
      kind: 'assignment',
      lhs: parseExpression(assignmentMatch[1], variables),
      rhs: parseExpression(assignmentMatch[2], variables),
    };

  const conditionPattern = /^\{([!?])\s*(.+?)\s*==\s*(.+?)\}$/s;
  const conditionMatch = conditionPattern.exec(source);
  if (conditionMatch)
    return {
      kind: 'condition',
      inverted: conditionMatch[1] === '!',
      lhs: parseExpression(conditionMatch[2], variables),
      rhs: parseExpression(conditionMatch[3], variables),
    };

  const reachabilityPattern = /^\{([!?])\s*(.+?)\s*->\s*(.+?)\}$/s;
  const reachabilityMatch = reachabilityPattern.exec(source);
  if (reachabilityMatch)
    return {
      kind: 'reachability',
      inverted: reachabilityMatch[1] === '!',
      lhs: +reachabilityMatch[2],
      rhs: +reachabilityMatch[3],
    };

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

export function parseEdges(
  source: string,
  variables: Record<string, Variable>,
) {
  const edgePattern = /(\d+)\s*,(\d+)\s*:\s*(.*?);/gs;
  const edges: Record<number, Edge[]> = Object.create(null);
  for (const [, a, b, label] of source.matchAll(edgePattern)) {
    if (!(a in edges)) edges[+a] = [];
    edges[+a].push({ a: +a, b: +b, label: parseEdgeLabel(label, variables) });
  }
  return edges;
}

export function parseExpression(
  source: string,
  variables: Record<string, Variable>,
): Expression {
  const accessPattern = /^(.+?)\[(.+?)\]$/s;
  const accessMatch = accessPattern.exec(source);
  if (accessMatch) {
    const [, name, key] = accessMatch;
    return {
      kind: 'variable-access',
      name,
      key: parseExpression(key, variables),
    };
  }

  const constantCallPattern = /^(.+?)\((.+?)\)$/s;
  const constantCallMatch = constantCallPattern.exec(source);
  if (constantCallMatch) {
    const [, name, argument] = constantCallMatch;
    return {
      kind: 'constant-call',
      name,
      argument: parseExpression(argument, variables),
    };
  }

  if (source in variables) {
    return { kind: 'variable', name: source };
  }

  return { kind: 'value', value: parseValue(source) };
}

export function parseType(source: string): Type {
  const arrowPattern = /^\s*(.+?)\s*->\s*(.+?)\s*$/s;
  const arrowMatch = arrowPattern.exec(source);
  if (arrowMatch)
    return {
      kind: 'arrow',
      from: parseType(arrowMatch[1]),
      to: parseType(arrowMatch[1]),
    };

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

export function parseVariables(source: string) {
  const variablePattern = /var\s+(.+?)\s*:(.+?),\s*(?:default\s*=\s*(.*?)\s*,\s*)?visible\s*=\s*(.+?);/gs;
  const variables: Record<string, Variable> = Object.create(null);
  for (const [, name, type, defaultValue, visibility] of source.matchAll(
    variablePattern,
  ))
    variables[name] = {
      defaultValue: defaultValue ? parseValue(defaultValue) : null,
      type: parseType(type),
      visibility: parseDomainValues(visibility),
    };
  return variables;
}
