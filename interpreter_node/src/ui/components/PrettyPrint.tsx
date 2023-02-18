import { useMemo } from 'react';

import { Editor } from './Editor';
import { pretty } from '../../utils';

export type PrettyPrintProps = { value: unknown };

export function PrettyPrint({ value }: PrettyPrintProps) {
  const [mode, text] = useMemo(() => {
    // Forward string as-is. At some point we'll differentiate between HL and LL.
    if (typeof value === 'string') {
      return ['text', value] as const;
    }

    // Errors usually aren't JSON-serializable.
    if (value instanceof Error) {
      return ['javascript', pretty(value, { colors: false })] as const;
    }

    return ['json', JSON.stringify(value, null, 2)] as const;
  }, [value]);

  return <Editor mode={mode} readOnly value={text} />;
}
