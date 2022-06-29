import { ResizeSensor } from '@blueprintjs/core';
import { useCallback, useState } from 'react';

const emptyBox = {
  toJSON: () => ({}),
  bottom: 0,
  height: 0,
  left: 0,
  right: 0,
  top: 0,
  width: 0,
  x: 0,
  y: 0,
};

export type AutosizeProps = { children: (box: DOMRectReadOnly) => JSX.Element };

export function Autosize({ children }: AutosizeProps) {
  const [box, setBox] = useState<DOMRectReadOnly>(emptyBox);
  const onResize = useCallback(
    (entries: ResizeObserverEntry[]) => {
      setBox(entries[0].contentRect);
    },
    [setBox],
  );

  return <ResizeSensor onResize={onResize}>{children(box)}</ResizeSensor>;
}
