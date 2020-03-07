export function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

export function average(xs: number[]) {
  return xs.reduce((a, b) => a + b, 0) / xs.length;
}
