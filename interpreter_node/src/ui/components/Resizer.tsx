import * as styles from '../index.module.css';

function onSized(event: MouseEvent | TouchEvent) {
  const resizer = document.querySelector('[data-resizer-active]');
  if (resizer instanceof HTMLElement && resizer.parentElement) {
    // Drag interactions often result in text selection which we don't want to
    // trigger. As there's no way to prevent it, we remove it.
    window.getSelection()?.removeAllRanges();

    // Resize.
    const element = resizer.parentElement;
    const position = event instanceof MouseEvent ? event : event.touches[0];
    if (resizer.dataset.resizer === 'horizontal') {
      element.style.width = `${Math.min(
        Math.max(position.clientX, 10) - element.offsetLeft,
        window.innerWidth - 10,
      )}px`;
    } else {
      element.style.height = `${Math.max(
        window.innerHeight - position.clientY,
        10,
      )}px`;
    }
  }
}

function onStart(event: MouseEvent | TouchEvent) {
  if (event.target instanceof HTMLElement && event.target.dataset.resizer) {
    event.target.dataset.resizerActive = '';
    onSized(event);
  }
}

function onStop() {
  const resizer = document.querySelector('[data-resizer-active]');
  if (resizer instanceof HTMLElement) {
    delete resizer.dataset.resizerActive;
  }
}

if (typeof window !== 'undefined') {
  let modifier: false | { passive: true } = false;
  // @ts-expect-error Invalid event name.
  window.addEventListener('', null, {
    get passive() {
      modifier = { passive: true };
      return false;
    },
  });

  document.addEventListener('mousedown', onStart, modifier);
  document.addEventListener('mousemove', onSized, modifier);
  document.addEventListener('mouseup', onStop, modifier);
  document.addEventListener('touchend', onStop, modifier);
  document.addEventListener('touchmove', onSized, modifier);
  document.addEventListener('touchstart', onStart, modifier);
}

export type ResizerProps = { axis: 'horizontal' | 'vertical' };

export function Resizer({ axis }: ResizerProps) {
  return <div className={styles.resizer} data-resizer={axis} />;
}
