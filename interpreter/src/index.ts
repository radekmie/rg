import { State, Value } from './types';
import { average } from './utils';
import { createInitialState, nextStates } from './state';
import { join } from 'path';
import { parse } from './parse';

function displayState(state: State) {
  const variables = displayVariables(state.variables, '    ');
  return `${state.player ?? '(keeper)'} at ${state.position} vars ${variables}`;
}

function displayValue(value: null | Value, indent = '') {
  if (value === null) return '(none)';
  switch (value.kind) {
    case 'map':
      return displayVariables(value.values, indent);
    case 'symbol':
      return value.value;
    case 'wildcard':
      return '*';
  }
}

function displayVariables(
  variables: Record<string, null | Value>,
  indent = '',
): string {
  const entries = Object.entries(variables)
    .map(([name, value]) => `${name} = ${displayValue(value, indent + '  ')}`)
    .sort();

  type StringsPair = [string[], string[]];
  const lines =
    entries.length % 8 === 0
      ? entries.reduce<StringsPair>(
          ([lines, line], entry) =>
            line.length === 7
              ? ([[...lines, [...line, entry].join(' ')], []] as StringsPair)
              : ([lines, [...line, entry]] as StringsPair),
          [[], []],
        )[0]
      : entries;

  return `{\n${indent}${lines.join(`\n${indent}`)} }`;
}

function run(path: string, debug = false) {
  const game = parse(path);

  const moves: number[] = [];
  const stats: number[] = [];
  const times: number[] = [];
  const turns: number[] = [];

  for (let play = 1; play <= 100; ++play) {
    const now = process.hrtime();
    let state = createInitialState(game);
    let turn = 0;
    for (;;) {
      if (debug) console.log(displayState(state));
      const states: string[] = [];
      for (const nextState of nextStates(game, state))
        states.push(JSON.stringify(nextState));
      if (states.length === 0) break;
      moves.push(states.length);
      state = JSON.parse(states[Math.floor(states.length * Math.random())]);
      ++turn;
    }

    const [s, ns] = process.hrtime(now);
    times.push(Math.round(s * 1e3 + ns / 1e6));
    turns.push(turn);

    const score = state.variables.white;
    if (score?.kind === 'symbol') stats.push(score.value === '100' ? 1 : 0);
    else stats.push(Infinity);

    console.clear();
    console.log(`after ${play} plays:`);
    console.log(`  avg. moves: ${average(moves).toFixed(2)}`);
    console.log(`  avg. stats: ${average(stats).toFixed(2)}`);
    console.log(`  avg. times: ${average(times).toFixed(2)}`);
    console.log(`  avg. turns: ${average(turns).toFixed(2)}`);
  }

  console.log('Done.');
}

run(join(__dirname, '../../examples/breakthrough.rg'));
