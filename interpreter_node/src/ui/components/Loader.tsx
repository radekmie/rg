import { Spinner } from '@blueprintjs/core';

import * as styles from '../index.module.css';

export function Loader() {
  return (
    <section className={styles.wrapHidden}>
      <Spinner className={styles.spinner} />
    </section>
  );
}
