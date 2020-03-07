import { EdgeLabel, Game, State, Type, Value } from './types';
import { assert } from './utils';
import { parseValue } from './parse';

export function applyLabelBackward(
  game: Game,
  state: State,
  label: EdgeLabel,
  echo: unknown,
) {
  switch (label.kind) {
    case 'assignment':
      setVariable(game, state, label.lhs, echo as Value);
      return;
    case 'condition':
      return;
    case 'empty':
      return;
    case 'reachability':
      return;
    case 'switch':
      state.player = echo as null | string;
      return;
  }
}

export function* applyLabelForward(game: Game, state: State, label: EdgeLabel) {
  switch (label.kind) {
    case 'assignment': {
      const value = evaluateExpression(game, state, label.rhs);
      const values =
        value.kind === 'wildcard'
          ? resolveDomainValues(game, game.variables[label.lhs].type)
          : [value];
      for (const value of values) {
        const previousValue = setVariable(game, state, label.lhs, value);
        yield previousValue;
      }
      return;
    }
    case 'condition':
      if (evaluateCondition(game, state, label)) yield;
      return;
    case 'empty':
      yield;
      return;
    case 'reachability':
      if (evaluateReachability(game, state, label)) yield;
      return;
    case 'switch': {
      const previousPlayer = state.player;
      state.player = label.player;
      yield previousPlayer;
      return;
    }
  }
}

export function createInitialState(game: Game): State {
  const variables: Record<string, null | Value> = {};
  for (const name of Object.keys(game.variables))
    variables[name] = createInitialValue(game, name);
  return { player: null, position: 0, variables };
}

export function createInitialValue(
  game: Game,
  variableName: string,
): null | Value {
  const { defaultValue, type } = game.variables[variableName];
  switch (type.kind) {
    case 'arrow': {
      assert(defaultValue, 'Maps require default value.');
      const values: Record<string, Value> = {};
      for (const value of resolveDomainValues(game, type.from)) {
        assert(
          value.kind === 'symbol',
          'Maps of non-symbols is not implemented yet.',
        );
        values[value.value] = defaultValue;
      }
      return { kind: 'map', values };
    }
    case 'domain':
      return defaultValue;
    case 'domain-inline':
      assert(false, '"domain-inline" is not implemented yet.');
  }
}

export function evaluateCondition(game: Game, state: State, label: EdgeLabel) {
  assert(label.kind === 'condition', 'Expected "condition".');
  const lhs = evaluateExpression(game, state, label.lhs);
  const rhs = evaluateExpression(game, state, label.rhs);
  const equal = evaluateEquality(lhs, rhs);
  return label.inverted ? !equal : equal;
}

export function evaluateEquality(lhs: Value, rhs: Value) {
  switch (lhs.kind) {
    case 'map':
      if (lhs.kind !== rhs.kind) return false;
      for (const key of Object.keys(lhs.values))
        if (!evaluateEquality(lhs.values[key], rhs.values[key])) return false;
      return true;
    case 'symbol':
      if (lhs.kind !== rhs.kind) return false;
      return lhs.value === rhs.value;
    case 'wildcard':
      if (lhs.kind !== rhs.kind) return false;
      return true;
  }
}

export function evaluateExpression(
  game: Game,
  state: State,
  expression: string,
): Value {
  const accessPattern = /^(.+?)\[(.+?)\]$/s;
  const accessMatch = accessPattern.exec(expression);
  if (accessMatch) {
    const [, variable, keyExpression] = accessMatch;
    const value = state.variables[variable];
    assert(value, 'Only existing "map" can be accessed.');
    assert(value.kind === 'map', 'Only "map" can be accessed.');
    const key = evaluateExpression(game, state, keyExpression);
    assert(key.kind === 'symbol', 'Only "symbol" can be used for an access.');
    return value.values[key.value];
  }

  const constantCallPattern = /^(.+?)\((.+?)\)$/s;
  const constantCallMatch = constantCallPattern.exec(expression);
  if (constantCallMatch) {
    const [, constantName, argumentExpression] = constantCallMatch;
    const constant = game.constants[constantName];
    const argument = evaluateExpression(game, state, argumentExpression);
    assert(
      argument.kind === 'symbol',
      'Only "symbol" can be used as an argument.',
    );
    return constant.values[argument.value] ?? constant.defaultValue;
  }

  if (expression in state.variables) {
    const value = state.variables[expression];
    assert(value, `"${expression}" accessed while not being set.`);
    return value;
  }

  return parseValue(expression);
}

export function evaluateReachability(
  game: Game,
  state: State,
  label: EdgeLabel,
) {
  assert(label.kind === 'reachability', 'Expected "reachability".');
  const origin = state.position;
  state.position = label.lhs;
  let reachable = false;
  for (const reachedState of nextStates(game, state))
    if (reachedState.position === label.rhs) reachable = true;
  state.position = origin;
  return label.inverted ? !reachable : reachable;
}

export function* nextStates(
  game: Game,
  state: State,
): Generator<State, void, unknown> {
  if (!(state.position in game.edges)) return;
  for (const edge of game.edges[state.position]) {
    for (const echo of applyLabelForward(game, state, edge.label)) {
      state.position = edge.b;

      if (edge.label.kind === 'switch') yield state;
      else yield* nextStates(game, state);

      state.position = edge.a;
      applyLabelBackward(game, state, edge.label, echo);
    }
  }
}

export function resolveDomainValues(game: Game, type: Type): Value[] {
  switch (type.kind) {
    case 'arrow':
      assert(false, '"arrow" is not implemented yet.');
      break;
    case 'domain':
      return game.domains[type.name].values;
    case 'domain-inline':
      return type.values;
  }
}

export function setVariable(
  game: Game,
  state: State,
  path: string,
  value: Value,
) {
  const accessPattern = /^(.+?)(?:\[(.+?)\])?$/s;
  const accessMatch = accessPattern.exec(path);
  assert(accessMatch, 'Invalid path.');
  const [, variable, keyExpression] = accessMatch;

  const previousValue = state.variables[variable];
  if (keyExpression === undefined) {
    if (game.variables[variable].type.kind === 'arrow') {
      assert(value.kind === 'map', 'Map required.');
      const initialValue = createInitialValue(game, variable);
      assert(initialValue?.kind === 'map', 'Map required.');
      state.variables[variable] = {
        kind: 'map',
        values: { ...initialValue.values, ...value.values },
      };
    } else {
      state.variables[variable] = value;
    }

    return previousValue;
  }

  assert(previousValue, 'Only existing "map" can be accessed.');
  assert(previousValue.kind === 'map', 'Only "map" can be accessed.');
  const key = evaluateExpression(game, state, keyExpression);
  assert(key.kind === 'symbol', 'Only "symbol" can be used for an access.');
  const previousKeyValue = previousValue.values[key.value];
  previousValue.values[key.value] = value;
  return previousKeyValue;
}
