import util from 'util';

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

const collator = new Intl.Collator(undefined, { numeric: true });
export function localeCompare(x: string, y: string) {
  return collator.compare(x, y);
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

export function prettyError(error: unknown) {
  return error instanceof Error && error.name === 'WorkerError'
    ? error.message
    : pretty(error, { colors: false });
}
