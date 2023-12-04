import { useMemo } from 'react';

import { Editor } from './Editor';
import { pretty } from '../../utils';

export type PrettyPrintProps = { value: unknown };

export function PrettyPrint({ value }: PrettyPrintProps) {
  const [path, text] = useMemo(() => {
    // Forward string as-is. At some point we'll differentiate between HL and LL.
    if (typeof value === 'string') {
      return ['plain.text', value] as const;
    }

    // Errors usually aren't JSON-serializable.
    if (value instanceof Error) {
      return [
        'errors.javascript',
        value.name === 'WorkerError'
          ? value.message
          : pretty(value, { colors: false }),
      ] as const;
    }

    return ['result.json', JSON.stringify(value, null, 2)] as const;
  }, [value]);

  return <Editor path={path} readOnly source={text} />;
}
