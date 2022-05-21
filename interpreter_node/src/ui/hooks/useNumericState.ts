import { useCallback, useState } from 'react';

export type State = {
  onValueChange: (number: number, string: string) => void;
  valueAsNumber: number;
  valueAsString: string;
};

export function useNumericState(initialValue: number): State {
  const [state, setState] = useState({
    valueAsNumber: initialValue,
    valueAsString: String(initialValue),
  });

  const onValueChange = useCallback(
    (valueAsNumber: number, valueAsString: string) => {
      setState({ valueAsNumber, valueAsString });
    },
    [],
  );

  return { onValueChange, ...state };
}
