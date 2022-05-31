import * as hrg from './hrg';
import * as rg from './rg';
import * as translators from './translators';
import { Extension, Settings } from './types';

function analyzeHrg(source: string, settings: Settings) {
  const sourceHrg = source;
  const cstHrg = hrg.cst.parse(sourceHrg).cstNode;
  const astHrg = hrg.ast.visit(cstHrg);
  const astRg = translators.hrg2rg(astHrg);

  return {
    ...analyzeRgAst(astRg, settings),
    astHrg,
    astRg,
    cstHrg,
    sourceHrg,
  };
}

function analyzeRg(source: string, settings: Settings) {
  const sourceRg = source;
  const cstRg = rg.cst.parse(sourceRg).cstNode;
  const astRg = rg.ast.visit(cstRg);

  return analyzeRgAst(astRg, settings);
}

function analyzeRgAst(astRg: rg.ast.GameDeclaration, settings: Settings) {
  if (settings.flags.compactSkipEdges) {
    rg.optimizer.compactSkipEdges(astRg);
  }

  const sourceRg = rg.ast.serializeGameDeclaration(astRg);
  const cstRg = rg.cst.parse(sourceRg).cstNode;
  const istRg = rg.ist.build(astRg);
  const graphvizRg = rg.ast.graphviz(astRg);

  return {
    astHrg: null,
    astRg,
    cstHrg: null,
    cstRg,
    graphvizRg,
    istRg,
    sourceHrg: null,
    sourceRg,
  };
}

export function parse(source: string, settings: Settings) {
  switch (settings.extension) {
    case Extension.hrg:
      return analyzeHrg(source, settings);
    case Extension.rg:
      return analyzeRg(source, settings);
  }
}
