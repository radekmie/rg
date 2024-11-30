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
