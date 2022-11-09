import { Spinner } from '@blueprintjs/core';
import { Graphviz as GraphvizWasm } from '@hpcc-js/wasm';
import { SVGProps, useEffect, useState } from 'react';
import { TOOL_PAN, UncontrolledReactSVGPanZoom } from 'react-svg-pan-zoom';

import { Result } from '../../utils';
import * as styles from '../index.module.css';
import { Autosize } from './Autosize';
import { PrettyPrint } from './PrettyPrint';

const graphvizLoader = GraphvizWasm.load();

export type GraphvizProps = {
  source: string;
};

export function Graphviz({ source }: GraphvizProps) {
  const [svg, setSVG] = useState<null | Result<SVGProps<SVGSVGElement>>>(null);
  useEffect(() => {
    graphvizLoader.then(
      graphviz => {
        const div = document.createElement('div');
        div.innerHTML = graphviz.dot(source, 'svg');

        const svg = div.children[0] as SVGSVGElement;
        const height = svg.height.baseVal.valueInSpecifiedUnits;
        const width = svg.width.baseVal.valueInSpecifiedUnits;

        setSVG({
          ok: true,
          value: {
            dangerouslySetInnerHTML: { __html: svg.innerHTML },
            height,
            width,
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
    <section className={styles.wrapHidden}>
      <Spinner className={styles.spinner} />
    </section>
  ) : svg.ok ? (
    <Autosize>
      {({ height, width }) => (
        <section className={styles.wrapHidden}>
          <UncontrolledReactSVGPanZoom
            customMiniature={Miniature}
            defaultTool={TOOL_PAN}
            height={height}
            width={width}
          >
            <svg {...svg.value}>
              <g dangerouslySetInnerHTML={svg.value.dangerouslySetInnerHTML} />
            </svg>
          </UncontrolledReactSVGPanZoom>
        </section>
      )}
    </Autosize>
  ) : (
    <PrettyPrint value={svg.error} />
  );
}

function Miniature() {
  return null;
}
