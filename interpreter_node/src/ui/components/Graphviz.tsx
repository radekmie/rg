import { Spinner } from '@blueprintjs/core';
import type { graphviz as GraphvizWASM } from '@hpcc-js/wasm';
import { SVGProps, useEffect, useState } from 'react';

import { Result } from '../../utils';
import { Autosize } from './Autosize';
import * as styles from './Graphviz.module.css';
import { PrettyPrint } from './PrettyPrint';

// @ts-expect-error: Loaded in HTML.
// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access -- Loaded in HTML.
const graphviz = window['@hpcc-js/wasm'].graphviz as typeof GraphvizWASM;

export type GraphvizProps = {
  source: string;
};

export function Graphviz({ source }: GraphvizProps) {
  const [svg, setSVG] = useState<null | Result<SVGProps<SVGSVGElement>>>(null);
  useEffect(() => {
    graphviz.dot(source, 'svg').then(
      source => {
        const div = document.createElement('div');
        div.innerHTML = source;

        const svg = div.children[0] as SVGSVGElement;
        const height = svg.height.baseVal.valueInSpecifiedUnits;
        const width = svg.width.baseVal.valueInSpecifiedUnits;

        setSVG({
          ok: true,
          value: {
            dangerouslySetInnerHTML: { __html: svg.innerHTML },
            viewBox: `0 0 ${width} ${height}`,
          },
        });
      },
      error => {
        setSVG({ ok: false, error });
      },
    );
  }, [source]);

  return svg === null ? (
    <section className={styles.wrap}>
      <Spinner className={styles.spinner} />
    </section>
  ) : svg.ok ? (
    <Autosize>
      {({ height, width }) => (
        <section className={styles.wrap}>
          <svg height={height - 1} width={width - 1} {...svg.value} />
        </section>
      )}
    </Autosize>
  ) : (
    <PrettyPrint value={svg.error} />
  );
}
