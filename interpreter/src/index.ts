import fs from 'fs';

import { parse } from './ast';
import { build } from './ist';
import * as types from './ist/types';
import * as utils from './utils';
import { cloneState, createInitialState, nextStates } from './state';

function average(xs: number[]) {
  return xs.reduce((a, b) => a + b, 0) / xs.length;
}

function* nextStatesN(
  game: types.Game,
  state: types.State,
  depth: number,
): Generator<types.State, void, undefined> {
  if (depth === 0) yield state;
  else {
    for (const nextState of nextStatesUnique(game, state))
      yield* nextStatesN(game, nextState, depth - 1);
  }
}

function nextStatesUnique(game: types.Game, state: types.State) {
  const states: Record<string, types.State> = Object.create(null);
  for (const nextState of nextStates(game, state, true)) {
    const key = JSON.stringify(nextState);
    if (!(key in states)) states[key] = cloneState(nextState);
  }
  return Object.values(states);
}

function run(game: types.Game, plays = 1, debug = false) {
  const moves: number[] = [];
  const times: number[] = [];
  const turns: number[] = [];

  for (let play = 1; play <= plays; ++play) {
    const now = process.hrtime();
    let state = createInitialState(game);
    let turn = 0;
    for (;;) {
      if (debug) console.log(utils.pretty(state));
      const states = nextStatesUnique(game, state);
      if (states.length === 0) break;
      if (state.position.label !== 'end') moves.push(states.length);
      state = states[Math.floor(states.length * Math.random())];
      if (state.position.label !== 'end') ++turn;
    }

    const [s, ns] = process.hrtime(now);
    times.push(Math.round(s * 1e3 + ns / 1e6));
    turns.push(turn);

    console.clear();
    console.log(`after ${play} plays:`);
    console.log(`  avg. moves: ${average(moves).toFixed(2)}`);
    console.log(`  avg. times: ${average(times).toFixed(2)}ms`);
    console.log(`  avg. turns: ${average(turns).toFixed(2)}`);
  }
}

function runPerf(game: types.Game, depth: number) {
  let count = 0;
  const initialState = createInitialState(game);
  console.time(`runPerf(depth: ${depth})`);
  for (const _ of nextStatesN(game, initialState, depth)) ++count;
  console.timeEnd(`runPerf(depth: ${depth})`);
  console.log(`runPerf(depth: ${depth}) = ${count}`);
}

const source = fs.readFileSync(process.argv[2], { encoding: 'utf8' });
const gameDefinition = parse(source);
const game = build(gameDefinition);

for (let depth = 1; depth <= 5; ++depth) runPerf(game, depth);
run(game, 100);
