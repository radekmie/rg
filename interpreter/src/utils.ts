import util from 'util';

export function assert(condition: unknown, message: string): asserts condition {
  if (!(condition as boolean)) throw new Error(message);
}

export function average(xs: number[]) {
  return xs.reduce((a, b) => a + b, 0) / xs.length;
}

export function cartesian<T>(xss: T[][], ys: T[]): T[][] {
  return xss.flatMap(xs => ys.map(y => xs.concat(y)));
}

export function creator<Type extends { kind: string }>(kind: Type['kind']) {
  return (data: Omit<Type, 'kind'>) => ({ kind, ...data });
}

export function find<T extends {}>(xs: T[], needle: Partial<T>) {
  const keys = Object.keys(needle) as (keyof T)[];
  return xs.find(x => keys.every(key => x[key] === needle[key]));
}

export function findMap<T, U>(xs: T[], fn: (x: T) => U | undefined) {
  for (const x of xs) {
    const y = fn(x);
    if (y !== undefined) {
      return y;
    }
  }

  return undefined;
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
