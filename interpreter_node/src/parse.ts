import { CstNode } from 'chevrotain';

import * as rbg from './rbg';
import * as rg from './rg';
import * as translators from './translators';
import { Language as L, Settings } from './types';
import * as utils from './utils';
import * as wasm from './wasm';

export type AnalyzedGameStep = { title?: string } & (
  | { kind: 'ast'; language: L.hrg; value: unknown }
  | { kind: 'ast'; language: L.rbg; value: rbg.ast.Game }
  | { kind: 'ast'; language: L.rg; value: rg.ast.GameDeclaration }
  | { kind: 'automaton'; value: string }
  | { kind: 'bench'; stats: string; value: rg.ast.GameDeclaration }
  | { kind: 'cst'; language: L.hrg; value: CstNode }
  | { kind: 'cst'; language: L.rbg; value: CstNode }
  | { kind: 'error'; value: unknown }
  | { kind: 'graphviz'; value: string }
  | { kind: 'source'; language: L; value: string }
  | { kind: 'stats'; value: string }
);

export class AnalyzedGame {
  steps: AnalyzedGameStep[] = [];

  get astHrg() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'ast' && step.language === L.hrg) {
        return step.value;
      }
    }
  }

  get astRbg() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'ast' && step.language === L.rbg) {
        return step.value;
      }
    }
  }

  get astRg() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'ast' && step.language === L.rg) {
        return step.value;
      }
    }

    utils.assert(false, 'RgAnalysisError');
  }

  get cstHrg() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'cst' && step.language === L.hrg) {
        return step.value;
      }
    }
  }

  get cstRbg() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'cst' && step.language === L.rbg) {
        return step.value;
      }
    }
  }

  get error() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'error') {
        return step.value;
      }
    }
  }

  get graphvizRg() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'graphviz') {
        return step.value;
      }
    }
  }

  get sourceHrg() {
    for (const step of this.steps) {
      if (step.kind === 'source' && step.language === L.hrg) {
        return step.value;
      }
    }
  }

  get sourceHrgFormatted() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'source' && step.language === L.hrg) {
        return step.value;
      }
    }
  }

  get sourceRbg() {
    for (const step of this.steps) {
      if (step.kind === 'source' && step.language === L.rbg) {
        return step.value;
      }
    }
  }

  get sourceRbgFormatted() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'source' && step.language === L.rbg) {
        return step.value;
      }
    }
  }

  get sourceRg() {
    for (const step of this.steps) {
      if (step.kind === 'source' && step.language === L.rg) {
        return step.value;
      }
    }

    utils.assert(false, 'RgAnalysisError');
  }

  get sourceRgFormatted() {
    for (const step of this.steps.slice().reverse()) {
      if (step.kind === 'source' && step.language === L.rg) {
        return step.value;
      }
    }

    utils.assert(false, 'RgAnalysisError');
  }
}

export async function parse(source: string, settings: Settings) {
  const game = new AnalyzedGame();
  const { extension, flags } = settings;

  try {
    game.steps.push({
      kind: 'source',
      language: extension,
      title: 'original',
      value: source,
    });

    switch (extension) {
      case L.gdl: {
        const steps = await wasm.analyzeGdl(source);
        game.steps.push(...steps);
        break;
      }
      case L.hrg: {
        const steps = await wasm.analyzeHrg(source, flags.reuseFunctions);
        game.steps.push(...steps);
        break;
      }
      case L.rbg: {
        let cst = rbg.cst.parse(source).cstNode;
        game.steps.push({ kind: 'cst', language: L.rbg, value: cst });

        const ast = rbg.ast.visit(cst);
        game.steps.push({ kind: 'ast', language: L.rbg, value: ast });

        source = rbg.ast.serializeGame(ast);
        game.steps.push({
          kind: 'source',
          language: L.rbg,
          title: 'formatted',
          value: source,
        });

        cst = rbg.cst.parse(source).cstNode;
        game.steps.push({ kind: 'cst', language: L.rbg, value: cst });

        if (!utils.isEqual(ast, rbg.ast.visit(cst))) {
          throw new Error('RbgFormattingError (AST mismatch)');
        }

        const astRg = translators.rbg2rg(ast);
        game.steps.push({ kind: 'ast', language: L.rg, value: astRg });

        break;
      }
    }

    if (extension !== L.rg) {
      const astRg = game.astRg;
      if (astRg) {
        const sourceRg = await wasm.serializeRg(astRg);
        game.steps.push({ kind: 'source', language: L.rg, value: sourceRg });
      }
    }

    const steps = await wasm.analyzeRg(game.sourceRg, flags);
    const graphviz = steps.pop();
    utils.assert(graphviz?.kind === 'graphviz', 'Graphviz step expected');
    const stats = steps.pop();
    utils.assert(stats?.kind === 'stats', 'Stats step expected');

    game.steps.push(...steps);
    game.steps.unshift(
      { kind: 'bench', stats: stats.value, value: game.astRg },
      { kind: 'automaton', value: graphviz.value },
      graphviz,
    );
  } catch (error) {
    // If the analysis failed, ignore register this error.
    if (game.steps[game.steps.length - 1].kind !== 'error') {
      game.steps.push({ kind: 'error', value: error });
    }
  }

  return game;
}
