import { useState } from 'react';

import { Extension, Optimize, Settings } from '../../types';

const initialSettings: Settings = {
  extension: Extension.hrg,
  optimize: Optimize.yes,
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(initialSettings);
  return { settings, setSettings };
}
