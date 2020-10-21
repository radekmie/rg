import util from 'util';

export function assert(condition: unknown, message: string): asserts condition {
  if (!(condition as boolean)) throw new Error(message);
}

export function average(xs: number[]) {
  return xs.reduce((a, b) => a + b, 0) / xs.length;
}

export function creator<Type extends { kind: string }>(kind: Type['kind']) {
  return (data: Omit<Type, 'kind'>) => ({ kind, ...data });
}

export function mapValues<T, U>(
  object: Record<string, T>,
  fn: (value: T) => U,
): Record<string, U> {
  return Object.entries(object).reduce((object, [key, value]) => {
    object[key] = fn(value);
    return object;
  }, Object.create(null));
}

export function pretty(object: unknown) {
  return util
    .inspect(object, {
      colors: true,
      compact: Infinity,
      depth: Infinity,
      sorted: true,
    })
    .replace(/\[Object: null prototype\] /g, '');
}
