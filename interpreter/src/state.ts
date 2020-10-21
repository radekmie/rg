// import { EdgeLabel, Expression, Game, State, Type, Value } from './types';
// import { assert } from './utils';

import * as types from './ist/types';
import * as utils from './utils';

export function cloneState(state: types.State) {
  return types.State({
    position: state.position,
    values: utils.mapValues(state.values, cloneValue),
  });
}

export function cloneValue(value: types.Value): types.Value {
  switch (value.kind) {
    case 'Element':
      return types.Element({ value: value.value });
    case 'Map':
      return types.Map({
        defaultValue: cloneValue(value.defaultValue),
        values: utils.mapValues(value.values, cloneValue),
      });
  }
}

export function createInitialState(game: types.Game) {
  return types.State({
    position: types.EdgeName({
      label: 'begin',
      types: Object.create(null),
      values: Object.create(null),
    }),
    values: utils.mapValues(game.variables, variable => variable.defaultValue),
  });
}

export function evaluateComparison(
  game: types.Game,
  state: types.State,
  label: types.EdgeLabel,
) {
  utils.assert(label.kind === 'Comparison', 'Expected Comparison.');
  const lhs = evaluateExpression(game, state, label.lhs);
  const rhs = evaluateExpression(game, state, label.rhs);
  const equal = evaluateEquality(lhs, rhs);
  return label.negated ? !equal : equal;
}

export function evaluateEquality(lhs: types.Value, rhs: types.Value): boolean {
  switch (lhs.kind) {
    case 'Element':
      return rhs.kind === 'Element' && lhs.value === rhs.value;
    case 'Map':
      if (rhs.kind !== 'Map') return false;
      // TODO: Type check.
      // TODO: It should iterate over type values.
      for (const [key, lhsValue] of Object.entries(lhs.values)) {
        const rhsValue = key in rhs.values ? rhs.values[key] : rhs.defaultValue;
        if (!evaluateEquality(lhsValue, rhsValue)) return false;
      }
      for (const [key, rhsValue] of Object.entries(rhs.values)) {
        const lhsValue = key in lhs.values ? lhs.values[key] : lhs.defaultValue;
        if (!evaluateEquality(rhsValue, lhsValue)) return false;
      }
      return true;
  }
}

export function evaluateExpression(
  game: types.Game,
  state: types.State,
  expression: types.Expression,
): types.Value {
  switch (expression.kind) {
    case 'Access': {
      const map = evaluateExpression(game, state, expression.lhs);
      utils.assert(map.kind === 'Map', 'Only Map can be accessed.');
      const key = evaluateExpression(game, state, expression.rhs);
      utils.assert(key.kind === 'Element', 'Only Element can be key.');
      // TODO: Type check.
      if (!(key.value in map.values))
        map.values[key.value] = cloneValue(map.defaultValue);
      return map.values[key.value];
    }
    case 'Cast':
      // TODO: Type check.
      return evaluateExpression(game, state, expression.rhs);
    case 'Reference': {
      if (expression.identifier in game.constants)
        return game.constants[expression.identifier];
      if (expression.identifier in game.variables)
        return state.values[expression.identifier];
      if (expression.identifier in state.position.types) {
        const value = state.position.values[expression.identifier];
        utils.assert(value !== null, `Unbound bind: ${expression.identifier}.`);
        return value;
      }
      return types.Element({ value: expression.identifier });
    }
  }
}

export function evaluateReachability(
  game: types.Game,
  state: types.State,
  label: types.EdgeLabel,
) {
  utils.assert(label.kind === 'Reachability', 'Expected Reachability.');

  const clone = cloneState(state);
  clone.position = label.lhs;

  // TODO: Check binds.
  for (const reachedState of nextStates(game, clone, false))
    if (reachedState.position.label === label.rhs.label) return true;
  return false;
}

export function* nextStates(
  game: types.Game,
  state: types.State,
  yieldOnPlayer: boolean,
): Generator<types.State, void, undefined> {
  if (!yieldOnPlayer) yield state;

  for (const { label, lhs, rhs: rhsGenerator } of game.edges) {
    // TODO: Check binds.
    if (state.position.label !== lhs.label) continue;
    for (const rhs of reifyNodes(game, lhs, rhsGenerator)) {
      state.position = rhs;
      switch (label.kind) {
        case 'Assignment': {
          const value = evaluateExpression(game, state, label.rhs);
          const previousValue = setValue(game, state, label.lhs, value);

          const yieldAndStop =
            yieldOnPlayer &&
            label.lhs.kind === 'Reference' &&
            label.lhs.identifier === 'player';

          if (yieldAndStop) yield state;
          else yield* nextStates(game, state, yieldOnPlayer);

          setValue(game, state, label.lhs, previousValue);
          break;
        }
        case 'Comparison':
          if (evaluateComparison(game, state, label))
            yield* nextStates(game, state, yieldOnPlayer);
          break;
        case 'Reachability':
          switch (label.mode) {
            case 'not':
              if (!evaluateReachability(game, state, label))
                yield* nextStates(game, state, yieldOnPlayer);
              break;
            case 'rev':
              if (evaluateReachability(game, state, label))
                yield* nextStates(game, state, yieldOnPlayer);
              break;
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

export function* reifyNodes(
  game: types.Game,
  lhs: types.EdgeName,
  rhsGenerator: types.EdgeName,
) {
  // Fast path: nothing to substitute.
  const binds = Object.keys(rhsGenerator.types);
  if (binds.length === 0) yield rhsGenerator;

  // Slow path: substitute all binds.
  const rhs = types.EdgeName({
    label: rhsGenerator.label,
    types: rhsGenerator.types,
    values: utils.mapValues(rhsGenerator.values, value => value),
  });

  // Copy existing lhs binds to rhs.
  for (const bind of binds) {
    if (bind in lhs.types && lhs.values[bind] !== null) {
      utils.assert(rhs.values[bind] === null, 'Double bind.');
      rhs.values[bind] = lhs.values[bind];
    }
  }

  function* iterateNth(
    index: number,
  ): Generator<types.EdgeName, void, undefined> {
    // All binds are substituted.
    if (binds.length === index) {
      yield rhs;
      return;
    }

    const bind = binds[index];
    const bindType = rhs.types[bind];
    utils.assert(bindType.kind === 'Set', 'Can iterate only over Set.');

    // This bind was substituted by lhs.
    if (rhs.values[bind] !== null) {
      yield* iterateNth(index + 1);
      return;
    }

    for (const value of bindType.identifiers) {
      rhs.values[bind] = types.Element({ value });
      yield* iterateNth(index + 1);
    }

    rhs.values[bind] = null;
  }

  yield* iterateNth(0);
}

export function setValue(
  game: types.Game,
  state: types.State,
  expression: types.Expression,
  value: types.Value,
): types.Value {
  switch (expression.kind) {
    case 'Access': {
      const map = evaluateExpression(game, state, expression.lhs);
      utils.assert(map.kind === 'Map', 'Only Map can be accessed.');
      const key = evaluateExpression(game, state, expression.rhs);
      utils.assert(key.kind === 'Element', 'Only Element can be key.');
      // TODO: Type check.
      const previousValue = map.values[key.value];
      map.values[key.value] = value;
      for (const value of Object.values(state.values)) compact(value);
      return previousValue;
    }
    case 'Cast':
      utils.assert(false, 'Cannot assign to a Cast.');
      break;
    case 'Reference': {
      utils.assert(
        !(expression.identifier in game.constants),
        'Cannot assign to a Constant.',
      );
      utils.assert(
        expression.identifier in game.variables,
        'Cannot assign to an Element.',
      );
      // TODO: Type check.
      const previousValue = state.values[expression.identifier];
      state.values[expression.identifier] = value;
      return previousValue;
    }
  }
}

export function compact(value: types.Value) {
  switch (value.kind) {
    case 'Map':
      // FIXME: Why is it happening?
      for (const key in value.values)
        if (value.values[key] === undefined) delete value.values[key];
      compact(value.defaultValue);
      for (const key of Object.keys(value.values)) {
        compact(value.values[key]);
        if (evaluateEquality(value.defaultValue, value.values[key]))
          delete value.values[key];
      }
      break;
    case 'Element':
      break;
  }
}
