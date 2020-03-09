import { EdgeLabel, Expression, Game, State, Type, Value } from './types';
import { assert } from './utils';

export function createInitialState(game: Game): State {
  const variables: Record<string, null | Value> = Object.create(null);
  for (const name in game.variables)
    variables[name] = createInitialValue(game, name);
  return { player: null, position: 0, variables };
}

export function createInitialValue(
  game: Game,
  variableName: string,
): null | Value {
  const { defaultValue, type } = game.variables[variableName];
  switch (type.kind) {
    case 'arrow':
      return { kind: 'map', values: Object.create(null) };
    case 'domain':
      return defaultValue;
    case 'domain-inline':
      assert(false, '"domain-inline" is not implemented yet.');
      break;
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
  if (lhs === rhs) return true;
  switch (lhs.kind) {
    case 'map':
      if (lhs.kind !== rhs.kind) return false;
      for (const key in lhs.values)
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
  expression: Expression,
): Value {
  switch (expression.kind) {
    case 'constant-call': {
      const constant = game.constants[expression.name];
      assert(constant, 'Accessed not existing variable.');
      const argument = evaluateExpression(game, state, expression.argument);
      assert(argument.kind === 'symbol', 'Only "symbol" can access.');
      for (const [mapArgument, mapResult] of constant.mapping)
        if (evaluateEquality(argument, mapArgument)) return mapResult;
      assert(constant.defaultValue, 'No default value provided.');
      return constant.defaultValue;
    }
    case 'value':
      return expression.value;
    case 'variable': {
      const value = state.variables[expression.name];
      assert(value, 'Accessed not existing variable.');
      return value;
    }
    case 'variable-access': {
      const value = state.variables[expression.name];
      assert(value, 'Accessed not existing variable.');
      assert(value.kind === 'map', 'Accessed not map.');
      const key = evaluateExpression(game, state, expression.key);
      assert(key.kind === 'symbol', 'Only "symbol" can access.');
      if (key.value in value.values) return value.values[key.value];
      const variable = game.variables[expression.name];
      assert(variable.defaultValue, 'No default value provided.');
      return variable.defaultValue;
    }
  }
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
): Generator<State, void, undefined> {
  if (!(state.position in game.edges)) return;
  for (const { a, b, label } of game.edges[state.position]) {
    state.position = b;
    switch (label.kind) {
      case 'assignment': {
        for (const value of resolveAssignedValues(game, state, label)) {
          const previousValue = setVariable(game, state, label.lhs, value);
          yield* nextStates(game, state);
          setVariable(game, state, label.lhs, previousValue);
        }
        break;
      }
      case 'condition':
        if (evaluateCondition(game, state, label))
          yield* nextStates(game, state);
        break;
      case 'empty':
        yield* nextStates(game, state);
        break;
      case 'reachability':
        if (evaluateReachability(game, state, label))
          yield* nextStates(game, state);
        break;
      case 'switch': {
        const previousPlayer = state.player;
        state.player = label.player;
        yield state;
        state.player = previousPlayer;
        break;
      }
    }
    state.position = a;
  }
}

export function resolveAssignedValues(
  game: Game,
  state: State,
  label: EdgeLabel,
) {
  assert(label.kind === 'assignment', 'Expected "assignment".');
  const value = evaluateExpression(game, state, label.rhs);
  if (value.kind === 'wildcard') {
    assert(label.lhs.kind === 'variable', 'Only variable can be *.');
    return resolveDomainValues(game, game.variables[label.lhs.name].type);
  }

  return [value];
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
  path: Expression,
  value: null | Value,
): null | Value {
  switch (path.kind) {
    case 'constant-call':
      assert(false, 'Cannot assign to a call.');
      break;
    case 'value':
      assert(false, 'Cannot assign to a value.');
      break;
    case 'variable': {
      const previousValue = state.variables[path.name];
      const { type } = game.variables[path.name];
      switch (type.kind) {
        case 'arrow': {
          assert(value, 'Map required.');
          assert(value.kind === 'map', 'Map required.');
          const initialValue = createInitialValue(game, path.name);
          assert(initialValue, 'Map required.');
          assert(initialValue.kind === 'map', 'Map required.');
          const domainVs = resolveDomainValues(game, type.to);
          for (const valueV of Object.values(value.values)) {
            if (!domainVs.some(domainV => evaluateEquality(domainV, valueV)))
              assert(false, 'Invalid assignment.');
          }
          state.variables[path.name] = { kind: 'map', values: value.values };
          break;
        }
        case 'domain':
        case 'domain-inline':
          if (value) {
            const domainVs = resolveDomainValues(game, type);
            if (!domainVs.some(domainV => evaluateEquality(domainV, value)))
              assert(false, 'Invalid assignment.');
          }
          state.variables[path.name] = value;
          break;
      }

      return previousValue;
    }
    case 'variable-access': {
      const previousValue = state.variables[path.name];
      assert(previousValue, 'Only existing "map" can be accessed.');
      assert(previousValue.kind === 'map', 'Only "map" can be accessed.');
      const key = evaluateExpression(game, state, path.key);
      assert(key.kind === 'symbol', 'Only "symbol" can access.');
      const previousKeyValue =
        key.value in previousValue.values
          ? previousValue.values[key.value]
          : null;
      if (value === null) delete previousValue.values[key.value];
      else {
        const variable = game.variables[path.name];
        assert(variable.defaultValue, 'No default value provided.');
        if (evaluateEquality(value, variable.defaultValue))
          delete previousValue.values[key.value];
        else {
          const { type } = game.variables[path.name];
          assert(type.kind === 'arrow', 'Must be an "arrow" type.');
          const domainVs = resolveDomainValues(game, type.to);
          if (!domainVs.some(domainV => evaluateEquality(domainV, value)))
            assert(false, 'Invalid assignment.');
          previousValue.values[key.value] = value;
        }
      }
      return previousKeyValue;
    }
  }
}
