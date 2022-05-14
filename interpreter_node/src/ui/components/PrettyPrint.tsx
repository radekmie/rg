import { Intent } from '@blueprintjs/core';
import Convert from 'ansi-to-html';
import classNames from 'classnames';
import { useMemo } from 'react';

import { pretty } from '../../utils';
import * as styles from './PrettyPrint.module.css';

const convert = new Convert({ bg: '#fff', fg: '#000' });

export type PrettyPrintProps = { intent?: Intent; value: unknown };

export function PrettyPrint({ intent, value }: PrettyPrintProps) {
  const html = useMemo(
    () => (typeof value === 'string' ? value : convert.toHtml(pretty(value))),
    [value],
  );

  return (
    <pre
      className={classNames(
        `bp4-callout bp4-elevation-0 bp4-intent-${intent ?? ''}`,
        styles.pre,
      )}
    >
      <code dangerouslySetInnerHTML={{ __html: html }} />
    </pre>
  );
}
