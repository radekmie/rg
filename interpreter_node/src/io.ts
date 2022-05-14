import {
  buildAST,
  serializeAST,
  serializeEdgeName,
  serializeEdgeLabel,
} from './ast';
import parse from './cst';
import translate from './down-level';
import buildIST from './ist';
import { optimize } from './optimizer';
import { Optimize, Settings } from './types';

function analyze(content: string, settings: Settings) {
  switch (settings.extension) {
    case '.hrg': {
      const hl = content;
      const ll = serializeAST(translate(hl));
      return { hl, ll };
    }
    case '.rg': {
      const hl = null;
      const ll = content;
      return { hl, ll };
    }
    default:
      throw new Error(`Unknown extension "${settings.extension}".`);
  }
}

export function openGame(content: string, settings: Settings) {
  const source = analyze(content, settings);
  if (settings.optimize === Optimize.yes) {
    const cst = parse(source.ll);
    const ast = buildAST(cst);
    optimize(ast);
    source.ll = serializeAST(ast);
  }

  const cst = parse(source.ll);
  const ast = buildAST(cst);
  const ist = buildIST(ast);
  const graphvizEdges = ast.edges.map(edge => {
    const lhs = serializeEdgeName(edge.lhs);
    const rhs = serializeEdgeName(edge.rhs);
    const label = serializeEdgeLabel(edge.label);
    return `  "${lhs}" -> "${rhs}" [label="${label}"];`;
  });

  const graphviz = `digraph {\n${graphvizEdges.join('\n')}\n}`;
  return { ast, cst, ist, graphviz, source };
}
