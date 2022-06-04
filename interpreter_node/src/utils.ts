import util from 'util';

export type Result<T, E = unknown> =
  | { ok: true; value: T }
  | { ok: false; error: E };

export function assert(condition: unknown, message: string): asserts condition {
  if (!(condition as boolean)) {
    throw new Error(message);
  }
}

export function cartesian<T>(xss: T[][], ys: T[]): T[][] {
  return xss.flatMap(xs => ys.map(y => xs.concat(y)));
}

export function creator<Type extends { kind: string }>(kind: Type['kind']) {
  return (data: Omit<Type, 'kind'>) => ({ kind, ...data });
}

export function find<T>(xs: T[], needle: Partial<T>) {
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

export function isEqual<T>(a: T, b: T) {
  return JSON.stringify(a) === JSON.stringify(b);
}

export function mapToObject<T, U>(
  array: T[],
  fn: (element: T, index: number) => [string, U],
) {
  return Object.fromEntries(array.map(fn));
}

export function mapValues<T, U>(
  object: Record<string, T>,
  fn: (value: T) => U,
): Record<string, U> {
  return Object.entries(object).reduce<Record<string, U>>(
    (object, [key, value]) => {
      object[key] = fn(value);
      return object;
    },
    Object.create(null),
  );
}

type Options = Partial<Parameters<typeof util.inspect>[1]>;
export function pretty(object: unknown, options?: Options) {
  return util
    .inspect(object, {
      colors: true,
      compact: Infinity,
      depth: Infinity,
      sorted: true,
      ...options,
    })
    .replace(/\[Object: null prototype\] /g, '');
}

export function remove<T>(array: T[], element: T) {
  const index = array.indexOf(element);
  if (index !== -1) {
    array.splice(index, 1);
  }
}

export function safe<T>(fn: () => T): Result<T> {
  try {
    return { ok: true, value: fn() };
  } catch (error) {
    return { ok: false, error };
  }
}

export function unique<T>(array: T[], element: T) {
  if (!array.some(other => isEqual(other, element))) {
    array.push(element);
  }

  return array;
}
