import * as utils from '../../utils';
import * as ist from './types';

export function cloneState(state: ist.State) {
  return ist.State({
    position: state.position,
    values: utils.mapValues(state.values, cloneValue),
  });
}

export function cloneValue(value: ist.Value): ist.Value {
  switch (value.kind) {
    case 'Element':
      return ist.Element({ value: value.value });
    case 'Map':
      return ist.Map({
        defaultValue: cloneValue(value.defaultValue),
        values: utils.mapValues(value.values, cloneValue),
      });
  }
}

export function createInitialState(game: ist.Game) {
  return ist.State({
    position: 'begin',
    values: utils.mapValues(game.variables, variable => variable.defaultValue),
  });
}

export function evaluateComparison(
  game: ist.Game,
  state: ist.State,
  label: ist.EdgeLabel,
) {
  utils.assert(label.kind === 'Comparison', 'Expected Comparison.');
  const lhs = evaluateExpression(game, state, label.lhs, true);
  const rhs = evaluateExpression(game, state, label.rhs, true);
  const isEqual = evaluateEquality(lhs, rhs);
  return isEqual !== label.negated;
}

export function evaluateEquality(lhs: ist.Value, rhs: ist.Value): boolean {
  switch (lhs.kind) {
    case 'Element':
      utils.assert(rhs.kind === 'Element', 'Equality for different kinds.');
      return lhs.value === rhs.value;
    case 'Map':
      utils.assert(rhs.kind === 'Map', 'Equality for different kinds.');
      for (const [key, lhsValue] of Object.entries(lhs.values)) {
        const rhsValue = key in rhs.values ? rhs.values[key] : rhs.defaultValue;
        if (!evaluateEquality(lhsValue, rhsValue)) {
          return false;
        }
      }

      for (const [key, rhsValue] of Object.entries(rhs.values)) {
        const lhsValue = key in lhs.values ? lhs.values[key] : lhs.defaultValue;
        if (!evaluateEquality(rhsValue, lhsValue)) {
          return false;
        }
      }

      return true;
  }
}

export function evaluateExpression(
  game: ist.Game,
  state: ist.State,
  expression: ist.Expression,
  readOnly: boolean,
): ist.Value {
  switch (expression.kind) {
    case 'Access': {
      const map = evaluateExpression(game, state, expression.lhs, readOnly);
      utils.assert(map.kind === 'Map', 'Only Map can be accessed.');
      const key = evaluateExpression(game, state, expression.rhs, readOnly);
      utils.assert(key.kind === 'Element', 'Only Element can be key.');
      if (readOnly) {
        return key.value in map.values
          ? map.values[key.value]
          : map.defaultValue;
      }

      if (!(key.value in map.values)) {
        map.values[key.value] = cloneValue(map.defaultValue);
      }

      return map.values[key.value];
    }
    case 'ConstantReference':
      return game.constants[expression.identifier];
    case 'Literal':
      return expression.value;
    case 'VariableReference':
      return state.values[expression.identifier];
  }
}

export function evaluateReachability(
  game: ist.Game,
  state: ist.State,
  label: ist.EdgeLabel,
) {
  utils.assert(label.kind === 'Reachability', 'Expected Reachability.');

  const clone = cloneState(state);
  clone.position = label.lhs;

  for (const reachedState of nextStates(game, clone, false)) {
    if (reachedState.position === label.rhs) {
      return true;
    }
  }

  return false;
}

export function nextStates(
  game: ist.Game,
  state: ist.State,
  breakOnPlayer: boolean,
) {
  return nextStatesInner(game, state, breakOnPlayer, Object.create(null));
}

// eslint-disable-next-line complexity -- This function could be improved.
function* nextStatesInner(
  game: ist.Game,
  state: ist.State,
  breakOnPlayer: boolean,
  visitedStates: Record<string, Record<string, ist.Value>[]>,
): Generator<ist.State, void, undefined> {
  // Check whether this state was already visited and if so, skip it. It could
  // happen conditionally, only when a game requires that, but that'd require an
  // additional analysis.
  const visited = (visitedStates[state.position] ??= []);
  if (visited?.some(values => utils.isEqual(values, state.values))) {
    return;
  }

  visited.push(utils.clone(state.values));

  // Yield all states by default.
  if (!breakOnPlayer) {
    yield state;
  }

  for (const { label, next } of game.edges[state.position] ?? []) {
    const prev = state.position;
    state.position = next;
    switch (label.kind) {
      case 'Assignment': {
        const value = evaluateExpression(game, state, label.rhs, true);
        const previousValue = setValue(game, state, label.lhs, value);

        const yieldAndStop =
          breakOnPlayer &&
          label.lhs.kind === 'VariableReference' &&
          label.lhs.identifier === 'player';

        if (yieldAndStop) {
          yield state;
        } else {
          yield* nextStatesInner(game, state, breakOnPlayer, visitedStates);
        }

        setValue(game, state, label.lhs, previousValue);
        break;
      }
      case 'Comparison':
        if (evaluateComparison(game, state, label)) {
          yield* nextStatesInner(game, state, breakOnPlayer, visitedStates);
        }

        break;
      case 'Reachability':
        if (evaluateReachability(game, state, label) !== label.negated) {
          yield* nextStatesInner(game, state, breakOnPlayer, visitedStates);
        }

        break;
      case 'Skip':
        yield* nextStatesInner(game, state, breakOnPlayer, visitedStates);
        break;
    }
    state.position = prev;
  }
}

export function setValue(
  game: ist.Game,
  state: ist.State,
  expression: ist.Expression,
  value: ist.Value,
): ist.Value {
  switch (expression.kind) {
    case 'Access': {
      const map = evaluateExpression(game, state, expression.lhs, false);
      utils.assert(map.kind === 'Map', 'Only Map can be accessed.');
      const key = evaluateExpression(game, state, expression.rhs, true);
      utils.assert(key.kind === 'Element', 'Only Element can be a key.');
      const previousValue =
        key.value in map.values ? map.values[key.value] : map.defaultValue;

      if (evaluateEquality(previousValue, value)) {
        delete map.values[key.value];
      } else {
        map.values[key.value] = value;
      }

      compact(map);
      return previousValue;
    }
    case 'ConstantReference':
      utils.assert(false, 'Cannot assign to a ConstantReference.');
    case 'Literal':
      utils.assert(false, 'Cannot assign to a Literal.');
    case 'VariableReference': {
      const previousValue = state.values[expression.identifier];
      state.values[expression.identifier] = value;
      return previousValue;
    }
  }
}

export function compact(value: ist.Value) {
  switch (value.kind) {
    case 'Map':
      compact(value.defaultValue);
      for (const key of Object.keys(value.values)) {
        compact(value.values[key]);
        if (evaluateEquality(value.defaultValue, value.values[key])) {
          delete value.values[key];
        }
      }

      break;
    case 'Element':
      break;
  }
}
