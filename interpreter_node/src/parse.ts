import * as hrg from './hrg';
import * as rg from './rg';
import * as translators from './translators';
import { Extension, Settings } from './types';
import * as utils from './utils';

function analyzeHrg(source: string, settings: Settings) {
  const sourceHrg = source;
  const cstHrg = hrg.cst.parse(sourceHrg).cstNode;
  const astHrg = hrg.ast.visit(cstHrg);
  const astRg = translators.hrg2rg(astHrg, settings);
  const sourceRg = rg.ast.serializeGameDeclaration(astRg);

  const sourceHrgFormatted = hrg.ast.serializeGameDeclaration(astHrg);
  const cstHrgFormatted = hrg.cst.parse(sourceHrgFormatted).cstNode;
  const astHrgFormatted = hrg.ast.visit(cstHrgFormatted);
  if (!utils.isEqual(astHrg, astHrgFormatted)) {
    throw new Error('HrgFormattingError (AST mismatch)');
  }

  return {
    ...analyzeRg(sourceRg, settings),
    astHrg,
    astRg,
    cstHrg,
    sourceHrg,
    sourceHrgFormatted,
  };
}

function analyzeRg(source: string, settings: Settings) {
  const sourceRg = source;
  const cstRg = rg.cst.parse(sourceRg).cstNode;
  const astRg = rg.ast.visit(cstRg);

  if (settings.flags.compactSkipEdges) {
    rg.transformations.compactSkipEdges(astRg);
  }

  if (settings.flags.expandGeneratorNodes) {
    rg.transformations.expandGeneratorNodes(astRg);
  }

  if (settings.flags.mangleSymbols) {
    rg.transformations.mangleSymbols(astRg);
  }

  const istRg = rg.ist.build(astRg);
  const graphvizRg = rg.ast.graphviz(astRg);

  const sourceRgFormatted = rg.ast.serializeGameDeclaration(astRg);
  const cstRgFormatted = rg.cst.parse(sourceRgFormatted).cstNode;
  const astRgFormatted = rg.ast.visit(cstRgFormatted);
  if (!utils.isEqual(astRg, astRgFormatted)) {
    throw new Error('RgFormattingError (AST mismatch)');
  }

  return {
    astHrg: null,
    astRg,
    cstHrg: null,
    cstRg,
    graphvizRg,
    istRg,
    sourceHrg: null,
    sourceHrgFormatted: null,
    sourceRg,
    sourceRgFormatted,
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
