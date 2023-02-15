import { Graphviz as GraphvizWasm } from '@hpcc-js/wasm';
import { TOOL_PAN, UncontrolledReactSVGPanZoom } from 'react-svg-pan-zoom';

import { Autosize } from './Autosize';
import { Loader } from './Loader';
import { PrettyPrint } from './PrettyPrint';
import { usePromise } from '../hooks/usePromise';
import * as styles from '../index.module.css';

export type GraphvizProps = {
  source: string;
};

export function Graphviz({ source }: GraphvizProps) {
  const svg = usePromise(async () => {
    const graphviz = await GraphvizWasm.load();
    const div = document.createElement('div');
    div.innerHTML = graphviz.dot(source, 'svg');

    const svg = div.children[0] as SVGSVGElement;
    const height = svg.height.baseVal.valueInSpecifiedUnits;
    const width = svg.width.baseVal.valueInSpecifiedUnits;

    return {
      dangerouslySetInnerHTML: { __html: svg.innerHTML },
      height,
      width,
      viewBox: `0 0 ${width} ${height}`,
    };
  }, [source]);

  return svg.error ? (
    <PrettyPrint value={svg.error} />
  ) : svg.value ? (
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
    <Loader />
  );
}

function Miniature() {
  return null;
}
