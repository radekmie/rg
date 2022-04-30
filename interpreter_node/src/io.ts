import fs from 'fs';
import path from 'path';

import { buildAST, serializeAST, serializeEdgeName, serializeEdgeLabel } from './ast';
import parse from './cst';
import translate from './down-level';
import buildIST from './ist';

function read(file: string) {
  return fs.readFileSync(file, { encoding: 'utf8' });
}

function analyze(file: string) {
  const extension = path.extname(file);
  switch (extension) {
    case '.hrg': {
      const hl = read(file);
      const ll = serializeAST(translate(hl));
      return { hl, ll };
    }
    case '.rg': {
      const hl = null;
      const ll = read(file);
      return { hl, ll };
    }
    default:
      throw new Error(`Unknown extension "${extension}".`);
  }
}

export default function openGame(file: string) {
  const source = analyze(file);
  const cst = parse(source.ll);
  const ast = buildAST(cst);
  const ist = buildIST(ast);
  const graphvizEdges = ast.edges.map(edge => {
    const lhs = serializeEdgeName(edge.lhs);
    const rhs = serializeEdgeName(edge.rhs);
    const label = serializeEdgeLabel(edge.label);
    return `  "${lhs}" -> "${rhs}" [label="${label}"];`
  });
  const graphviz = `digraph "${file}" {\n${graphvizEdges.join('\n')}\n}`;
  return { ast, cst, ist, graphviz, source };
}
