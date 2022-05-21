import {
  cloneState,
  cloneValue,
  createInitialState,
  evaluateEquality,
  nextStates,
} from './ist/state';
import * as ist from './ist/types';
import * as utils from './utils';

export type Logger = { log: (message: string) => void };

function avg(counter: Record<number, number>) {
  const [x0, n0] = Object.entries(counter).reduce(
    ([x0, n0], [x, n]) => [x0 + n * +x, n0 + n],
    [0, 0],
  );
  return x0 / n0;
}

function format(number: number) {
  return number.toFixed(3);
}

function frame() {
  return new Promise(resolve => {
    if (typeof requestIdleCallback === 'undefined') {
      setTimeout(resolve);
    } else {
      requestIdleCallback(resolve);
    }
  });
}

function increase(counter: Record<number, number>, x: number) {
  if (x in counter) {
    counter[x]++;
  } else {
    counter[x] = 1;
  }
}

export async function run(game: ist.Game, plays = 1, logger: Logger) {
  // Display stats every ~1% of plays.
  const step = Math.max(1, Math.pow(10, Math.floor(Math.log10(plays / 100))));

  // Initialize counters.
  const moves: Record<number, number> = {};
  const times: Record<number, number> = {};
  const turns: Record<number, number> = {};

  for (let play = 1; play <= plays; ++play) {
    await frame();
    const now = performance.now();
    let state = createInitialState(game);
    let turn = 0;
    for (;;) {
      const states = Array.from(nextStates(game, state, true), cloneState);
      if (states.length === 0) {
        break;
      }

      utils.assert(state.values.player.kind === 'Element', 'Player is element');
      if (state.values.player.value !== 'keeper') {
        increase(moves, states.length);
        ++turn;
      }

      state = states[Math.floor(states.length * Math.random())];
    }

    const end = performance.now();
    increase(times, end - now);
    increase(turns, turn);

    if (play % step === 0) {
      logger.log(`after ${play} plays:`);
      logger.log(`  avg. moves: ${format(avg(moves))}`);
      logger.log(`  avg. turns: ${format(avg(turns))}`);
      logger.log(`  avg. times: ${format(avg(times))}ms`);
    }
  }
}

const keeper = ist.Element({ value: 'keeper' });
function isSameOrKeeper(prev: ist.Value, next: ist.Value) {
  return evaluateEquality(prev, next) || evaluateEquality(prev, keeper);
}

export async function perf(game: ist.Game, maxDepth: number, logger: Logger) {
  for (let depth = 0; depth <= maxDepth; ++depth) {
    await frame();
    let count = 0;
    const initialState = createInitialState(game);
    const now = performance.now();
    // eslint-disable-next-line @typescript-eslint/no-unused-vars -- It's not needed.
    for (const _ of nextStatesN(game, initialState, depth)) {
      ++count;
    }
    const end = performance.now();
    logger.log(`perf(depth: ${depth}) = ${count} in ${format(end - now)}ms`);
  }

  function* nextStatesN(
    game: ist.Game,
    state: ist.State,
    depth: number,
  ): Generator<ist.State, void, undefined> {
    if (depth === 0) {
      yield state;
    } else {
      const player = cloneValue(state.values.player);
      for (const nextState of nextStates(game, state, true)) {
        const step = isSameOrKeeper(player, state.values.player);
        yield* nextStatesN(game, nextState, depth - (step ? 0 : 1));
      }
    }
  }
}
