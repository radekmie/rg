import breakthrough from 'bundle-text:../../../../examples/breakthrough.hrg';
import { useDeferredValue, useMemo, useState } from 'react';

import { openGame } from '../../io';
import { Settings } from '../../types';
import { safe } from '../../utils';

export function useGame(settings: Settings) {
  const [source, setSource] = useState(breakthrough);
  const deferredSource = useDeferredValue(source);
  const game = useMemo(
    () => safe(() => openGame(deferredSource, settings)),
    [deferredSource, settings],
  );

  return { game, source, setSource };
}
