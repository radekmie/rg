import { DependencyList, useEffect, useState } from 'react';

export type AsyncState<T, E> =
  | { error: null; loading: true; value: null }
  | { error: E; loading: boolean; value: null }
  | { error: null; loading: boolean; value: T };

export function usePromise<T, E>(fn: () => Promise<T>, deps?: DependencyList) {
  const [state, setState] = useState<AsyncState<T, E>>({
    error: null,
    loading: true,
    value: null,
  });

  useEffect(() => {
    let isCanceled = false;

    setState(state => ({ ...state, loading: true }));
    fn().then(
      value => {
        if (!isCanceled) {
          setState({ error: null, loading: false, value });
        }
      },
      error => {
        if (!isCanceled) {
          setState({ error, loading: false, value: null });
        }
      },
    );

    return () => {
      isCanceled = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- `fn` is fine.
  }, deps);

  return state;
}
