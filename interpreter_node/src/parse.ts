import { CstNode } from 'chevrotain';

import * as hrg from './hrg';
import * as rbg from './rbg';
import * as rg from './rg';
import * as translators from './translators';
import { Extension, Settings } from './types';
import * as utils from './utils';

export type AnalyzedGame = {
  astHrg: hrg.ast.GameDeclaration | null;
  astRbg: rbg.ast.Game | null;
  astRg: rg.ast.GameDeclaration;
  cstHrg: CstNode | null;
  cstRbg: CstNode | null;
  cstRg: CstNode | null;
  graphvizRg: string;
  istRg: rg.ist.Game;
  sourceHrg: string | null;
  sourceHrgFormatted: string | null;
  sourceRbg: string | null;
  sourceRbgFormatted: string | null;
  sourceRg: string;
  sourceRgFormatted: string;
};

function analyzeHrg(source: string, settings: Settings): AnalyzedGame {
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
    cstHrg,
    sourceHrg,
    sourceHrgFormatted,
  };
}

function analyzeRbg(source: string, settings: Settings): AnalyzedGame {
  const sourceRbg = source;
  const cstRbg = rbg.cst.parse(sourceRbg).cstNode;
  const astRbg = rbg.ast.visit(cstRbg);
  const astRg = translators.rbg2rg(astRbg);
  const sourceRg = rg.ast.serializeGameDeclaration(astRg);

  const sourceRbgFormatted = rbg.ast.serializeGame(astRbg);
  const cstRbgFormatted = rbg.cst.parse(sourceRbgFormatted).cstNode;
  const astRbgFormatted = rbg.ast.visit(cstRbgFormatted);
  if (!utils.isEqual(astRbg, astRbgFormatted)) {
    throw new Error('RbgFormattingError (AST mismatch)');
  }

  return {
    ...analyzeRg(sourceRg, settings),
    astRbg,
    cstRbg,
    sourceRbg,
    sourceRbgFormatted,
  };
}

function analyzeRg(source: string, settings: Settings): AnalyzedGame {
  const sourceRg = source;
  const cstRg = rg.cst.parse(sourceRg).cstNode;
  const astRg = rg.ast.visit(cstRg);

  utils.runTransformators(
    astRg,
    ast => {
      rg.validators.reachables(ast);
      rg.validators.typecheck(ast);
    },
    (['compactSkipEdges', 'expandGeneratorNodes', 'mangleSymbols'] as const)
      .filter(option => settings.flags[option])
      .map(option => rg.transformators[option]),
  );

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
    astRbg: null,
    astRg,
    cstHrg: null,
    cstRbg: null,
    cstRg,
    graphvizRg,
    istRg,
    sourceHrg: null,
    sourceHrgFormatted: null,
    sourceRbg: null,
    sourceRbgFormatted: null,
    sourceRg,
    sourceRgFormatted,
  };
}

export function parse(source: string, settings: Settings) {
  switch (settings.extension) {
    case Extension.hrg:
      return analyzeHrg(source, settings);
    case Extension.rbg:
      return analyzeRbg(source, settings);
    case Extension.rg:
      return analyzeRg(source, settings);
  }
}
