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
    position: ist.EdgeName({
      label: 'begin',
      types: Object.create(null),
      values: Object.create(null),
    }),
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
      // TODO: Type check.
      // TODO: It should iterate over type values.
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
      // TODO: Type check.
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
    case 'BindReference': {
      const value = state.position.values[expression.identifier];
      utils.assert(value, `Unbound bind: ${expression.identifier}.`);
      return value;
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

  // TODO: Check binds.
  for (const reachedState of nextStates(game, clone, false)) {
    if (reachedState.position.label === label.rhs.label) {
      return true;
    }
  }
  return false;
}

// eslint-disable-next-line complexity -- This function could be improved.
export function* nextStates(
  game: ist.Game,
  state: ist.State,
  yieldOnPlayer: boolean,
): Generator<ist.State, void, undefined> {
  if (!yieldOnPlayer) {
    yield state;
  }

  for (const { label, lhs: lhsGenerator, rhs: rhsGenerator } of game.edges) {
    // TODO: Check binds.
    if (state.position.label !== lhsGenerator.label) {
      continue;
    }

    const lhs = state.position;
    for (const rhs of reifyNodes(lhs, rhsGenerator)) {
      state.position = rhs;
      switch (label.kind) {
        case 'Assignment': {
          const value = evaluateExpression(game, state, label.rhs, true);
          const previousValue = setValue(game, state, label.lhs, value);

          const yieldAndStop =
            yieldOnPlayer &&
            label.lhs.kind === 'VariableReference' &&
            label.lhs.identifier === 'player';

          if (yieldAndStop) {
            yield state;
          } else {
            yield* nextStates(game, state, yieldOnPlayer);
          }

          setValue(game, state, label.lhs, previousValue);
          break;
        }
        case 'Comparison':
          if (evaluateComparison(game, state, label)) {
            yield* nextStates(game, state, yieldOnPlayer);
          }
          break;
        case 'Reachability':
          if (evaluateReachability(game, state, label) !== label.negated) {
            yield* nextStates(game, state, yieldOnPlayer);
          }
          break;
        case 'Skip':
          yield* nextStates(game, state, yieldOnPlayer);
          break;
      }
      state.position = lhs;
    }
  }
}

export function* reifyNodes(lhs: ist.EdgeName, rhsGenerator: ist.EdgeName) {
  // Fast path: nothing to substitute.
  const bindTypes = { ...lhs.types, ...rhsGenerator.types };
  const binds = Object.keys(bindTypes);
  if (binds.length === 0) {
    yield rhsGenerator;
    return;
  }

  // Slow path: substitute all binds.
  const rhs = ist.EdgeName({
    label: rhsGenerator.label,
    types: rhsGenerator.types,
    values: { ...rhsGenerator.values },
  });

  // Copy existing lhs binds to rhs.
  for (const bind of binds) {
    if (bind in lhs.types && lhs.values[bind] !== undefined) {
      utils.assert(rhs.values[bind] === undefined, 'Double bind.');
      rhs.values[bind] = lhs.values[bind];
    }
  }

  function* iterateNth(
    index: number,
  ): Generator<ist.EdgeName, void, undefined> {
    // All binds are substituted.
    if (binds.length === index) {
      yield rhs;
      return;
    }

    const bind = binds[index];
    const bindType = bindTypes[bind];
    utils.assert(bindType.kind === 'Set', 'Can iterate only over Set.');

    // This bind was substituted by lhs.
    if (rhs.values[bind] !== undefined) {
      yield* iterateNth(index + 1);
    } else {
      for (const value of bindType.values) {
        rhs.values[bind] = value;
        yield* iterateNth(index + 1);
      }
    }

    delete rhs.values[bind];
  }

  yield* iterateNth(0);
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
    case 'BindReference':
      utils.assert(false, 'Cannot assign to a BindReference.');
      break;
    case 'ConstantReference':
      utils.assert(false, 'Cannot assign to a ConstantReference.');
      break;
    case 'Literal':
      utils.assert(false, 'Cannot assign to a Literal.');
      break;
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
