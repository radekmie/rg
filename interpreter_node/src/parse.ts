import { CstNode } from 'chevrotain';

import * as hrg from './hrg';
import * as rbg from './rbg';
import * as rg from './rg';
import * as translators from './translators';
import { Extension, Flag, Settings } from './types';
import * as utils from './utils';
import * as wasm from './wasm';

export type AnalyzedGame = {
  astHrg: hrg.ast.GameDeclaration | null;
  astRbg: rbg.ast.Game | null;
  astRg: rg.ast.GameDeclaration;
  cstHrg: CstNode | null;
  cstRbg: CstNode | null;
  graphvizRg: string;
  sourceHrg: string | null;
  sourceHrgFormatted: string | null;
  sourceRbg: string | null;
  sourceRbgFormatted: string | null;
  sourceRg: string;
  sourceRgFormatted: string;
};

async function analyzeHrg(source: string, settings: Settings) {
  const sourceHrg = source;
  const cstHrg = hrg.cst.parse(sourceHrg).cstNode;
  const astHrg = hrg.ast.visit(cstHrg);
  const astRg = translators.hrg2rg(astHrg, settings);
  const sourceRg = await wasm.serializeRg(astRg);

  const sourceHrgFormatted = hrg.ast.serializeGameDeclaration(astHrg);
  const cstHrgFormatted = hrg.cst.parse(sourceHrgFormatted).cstNode;
  const astHrgFormatted = hrg.ast.visit(cstHrgFormatted);
  if (!utils.isEqual(astHrg, astHrgFormatted)) {
    throw new Error('HrgFormattingError (AST mismatch)');
  }

  return {
    ...(await analyzeRg(sourceRg, settings)),
    astHrg,
    cstHrg,
    sourceHrg,
    sourceHrgFormatted,
  } as AnalyzedGame;
}

async function analyzeRbg(source: string, settings: Settings) {
  const sourceRbg = source;
  const cstRbg = rbg.cst.parse(sourceRbg).cstNode;
  const astRbg = rbg.ast.visit(cstRbg);
  const astRg = translators.rbg2rg(astRbg);
  const sourceRg = await wasm.serializeRg(astRg);

  const sourceRbgFormatted = rbg.ast.serializeGame(astRbg);
  const cstRbgFormatted = rbg.cst.parse(sourceRbgFormatted).cstNode;
  const astRbgFormatted = rbg.ast.visit(cstRbgFormatted);
  if (!utils.isEqual(astRbg, astRbgFormatted)) {
    throw new Error('RbgFormattingError (AST mismatch)');
  }

  return {
    ...(await analyzeRg(sourceRg, settings)),
    astRbg,
    cstRbg,
    sourceRbg,
    sourceRbgFormatted,
  } as AnalyzedGame;
}

async function analyzeRg(source: string, settings: Settings) {
  const sourceRg = source;
  const astRgBase = await wasm.parseRg(sourceRg);

  // Transformation helpers.
  function nodeTransform(name: Extract<keyof typeof rg.transformators, Flag>) {
    if (!settings.flags[name]) {
      return null;
    }

    return (gameDeclaration: rg.ast.GameDeclaration) => {
      const clone = utils.clone(gameDeclaration);
      rg.transformators[name](clone);
      return Promise.resolve(clone);
    };
  }

  function wasmTransform(
    flag: Flag,
    name: Extract<keyof typeof wasm, `transform${string}`>,
  ) {
    if (!settings.flags[flag]) {
      return null;
    }

    return (gameDeclaration: rg.ast.GameDeclaration) =>
      wasm[name](gameDeclaration);
  }

  // Other transformations are run in a fixpoint loop.
  const astRg = await utils.runTransformators(
    astRgBase,
    [wasm.validateCheckReachabilities, wasm.validateCheckTypes],
    [
      wasmTransform('normalizeTypes', 'transformNormalizeTypes'),
      wasmTransform('skipSelfAssignments', 'transformSkipSelfAssignments'),
      nodeTransform('compactSkipEdges'),
      wasmTransform('addExplicitCasts', 'transformAddExplicitCasts'),
      wasmTransform('expandGeneratorNodes', 'transformExpandGeneratorNodes'),
      nodeTransform('joinForkSuffixes'),
      nodeTransform('inlineReachability'),
      nodeTransform('mangleSymbols'),
    ].filter(utils.isNotNull),
  );

  const sourceRgFormatted = await wasm.serializeRg(astRg);
  const astRgFormatted = await wasm.parseRg(sourceRgFormatted);
  if (!utils.isEqual(astRg, astRgFormatted)) {
    throw new Error('RgFormattingError (AST mismatch)');
  }

  const graphvizRg = rg.ast.graphviz(astRg);

  return {
    astHrg: null,
    astRbg: null,
    astRg,
    cstHrg: null,
    cstRbg: null,
    graphvizRg,
    sourceHrg: null,
    sourceHrgFormatted: null,
    sourceRbg: null,
    sourceRbgFormatted: null,
    sourceRg,
    sourceRgFormatted,
  } as AnalyzedGame;
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
