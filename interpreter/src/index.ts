import openGame from './io';
import {
  cloneState,
  cloneValue,
  createInitialState,
  evaluateEquality,
  nextStates,
} from './ist/state';
import * as ist from './ist/types';
import * as utils from './utils';

function run(game: ist.Game, plays = 1, debug = false) {
  const moves: number[] = [];
  const times: number[] = [];
  const turns: number[] = [];

  for (let play = 1; play <= plays; ++play) {
    const now = process.hrtime();
    let state = createInitialState(game);
    let turn = 0;
    for (;;) {
      if (debug) console.log(utils.pretty(state));
      const states = Array.from(nextStates(game, state, true), cloneState);
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
    console.log(`  avg. moves: ${utils.average(moves).toFixed(2)}`);
    console.log(`  avg. times: ${utils.average(times).toFixed(2)}ms`);
    console.log(`  avg. turns: ${utils.average(turns).toFixed(2)}`);
  }
}

const keeper = ist.Element({ value: 'keeper' });
function isSameOrKeeper(prev: ist.Value, next: ist.Value) {
  return evaluateEquality(prev, next) || evaluateEquality(prev, keeper);
}

function runPerf(game: ist.Game, depth: number) {
  let count = 0;
  const initialState = createInitialState(game);
  console.time(`runPerf(depth: ${depth})`);
  for (const _ of nextStatesN(game, initialState, depth)) ++count;
  console.timeEnd(`runPerf(depth: ${depth})`);
  console.log(`runPerf(depth: ${depth}) = ${count}`);

  function* nextStatesN(
    game: ist.Game,
    state: ist.State,
    depth: number,
  ): Generator<ist.State, void, undefined> {
    if (depth === 0) yield state;
    else {
      const player = cloneValue(state.values.player);
      for (const nextState of nextStates(game, state, true)) {
        const step = isSameOrKeeper(player, state.values.player);
        yield* nextStatesN(game, nextState, depth - (step ? 0 : 1));
      }
    }
  }
}

const game = openGame(process.argv[2]);
switch (process.argv[3]) {
  case 'perf': {
    const maxDepth = +process.argv[4];
    utils.assert(isFinite(maxDepth) && maxDepth > 0, 'depth must be positive');
    for (let depth = 0; depth <= maxDepth; ++depth) runPerf(game.ist, depth);
    break;
  }
  case 'print-ast':
    console.log(JSON.stringify(game.ast));
    break;
  case 'print-cst':
    console.log(JSON.stringify(game.cst));
    break;
  case 'print-ist':
    console.log(JSON.stringify(game.ist));
    break;
  case 'print-source-hl':
    console.log(game.source.hl);
    break;
  case 'print-source-ll':
    console.log(game.source.ll);
    break;
  case 'run': {
    const plays = +process.argv[4];
    utils.assert(isFinite(plays) && plays > 0, 'plays must be positive');
    run(game.ist, plays);
    break;
  }
}
