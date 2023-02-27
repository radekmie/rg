import util from 'util';

export type Expand<T> = T extends infer U ? { [K in keyof U]: U[K] } : never;

export type Result<T, E = unknown> =
  | { ok: true; value: T }
  | { ok: false; error: E };

export function success<T, E>(value: T): Result<T, E> {
  return { ok: true, value };
}

export function failure<T, E>(error: E): Result<T, E> {
  return { ok: false, error };
}

export class AssertionError extends Error {
  constructor(message: string) {
    super(message);
    Object.defineProperty(this, 'name', { value: 'AssertionError' });
  }
}

export function assert(
  condition: unknown,
  message: string | (() => string),
): asserts condition {
  if (!(condition as boolean)) {
    throw new AssertionError(typeof message === 'string' ? message : message());
  }
}

export function cartesian<T>(xss: T[][], ys: T[]): T[][] {
  return xss.flatMap(xs => ys.map(y => xs.concat(y)));
}

export function clone<T>(x: T): T {
  if (x === null || typeof x !== 'object') {
    return x;
  }

  if (Array.isArray(x)) {
    return x.map(clone) as unknown as T;
  }

  const y = {} as T;
  for (const key of Object.keys(x) as (keyof T)[]) {
    y[key] = clone(x[key]);
  }

  return y;
}

export function creator<T extends { kind: string }>(kind: T['kind']) {
  return function creator(data: Expand<Omit<T, 'kind'>>) {
    return Object.assign({ kind }, data) as unknown as T;
  };
}

export function find<T>(xs: undefined | T[], needle: Partial<T>) {
  const keys = Object.keys(needle) as (keyof T)[];
  return xs?.find(x => keys.every(key => isEqual(x[key], needle[key])));
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

export function generate<T>(length: number, generate: (index: number) => T) {
  return Array.from({ length }, (_, index) => generate(index));
}

const indexPattern = /(\d*)$/;
function nextIndex(index: string) {
  return index ? String(1 + Number(index)) : '2';
}

export function generateIdentifier(
  xs: { identifier: string }[],
  identifier: string,
) {
  const identifiers = new Set(xs.map(x => x.identifier));
  while (identifiers.has(identifier)) {
    identifier = identifier.replace(indexPattern, nextIndex);
  }

  return identifier;
}

export function isDisjoint<T>(xs: T[], ys: T[]) {
  // Specialized version for simple data.
  if (xs.length && (typeof xs[0] === 'number' || typeof xs[0] === 'string')) {
    return xs.every(x => !ys.includes(x));
  }

  return xs.every(x => !ys.some(y => isEqual(x, y)));
}

// eslint-disable-next-line complexity -- It's fine.
export function isEqual<T>(a: T, b: T) {
  if (a === b) {
    return true;
  }

  if (!a || !b || typeof a !== 'object' || typeof b !== 'object') {
    return Number.isNaN(a) && Number.isNaN(b);
  }

  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) {
      return false;
    }

    for (let index = 0; index < a.length; ++index) {
      if (!isEqual(a[index], b[index])) {
        return false;
      }
    }

    return true;
  }

  const keys = Object.keys(a) as (keyof T)[];
  if (keys.length !== Object.keys(b).length) {
    return false;
  }

  for (const key of keys) {
    if (!isEqual(a[key], b[key])) {
      return false;
    }
  }

  return true;
}

export function isSubset<T>(xs: T[], ys: T[]) {
  // Specialized version for simple data.
  if (xs.length && (typeof xs[0] === 'number' || typeof xs[0] === 'string')) {
    return xs.every(x => ys.includes(x));
  }

  return xs.every(x => ys.some(y => isEqual(x, y)));
}

export const isNotNull = Boolean as unknown as <T>(x: T | null) => x is T;

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

// eslint-disable-next-line @typescript-eslint/no-empty-function -- Only one in the project.
export function noop() {}

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

export function unique<T>(array: T[], element: T) {
  // Specialized version that works in O(log(n)) instead of O(n) time.
  if (typeof element === 'number' || typeof element === 'string') {
    let min = 0;
    let max = array.length;
    while (min < max) {
      const mid = Math.floor((min + max) / 2);
      if (array[mid] === element) {
        return array;
      }

      if (array[mid] < element) {
        min = mid + 1;
      } else {
        max = mid;
      }
    }

    array.splice(min, 0, element);
  } else if (!array.some(other => isEqual(other, element))) {
    array.push(element);
  }

  return array;
}
