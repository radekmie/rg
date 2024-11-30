import { Language as L, Settings } from './types';
import * as utils from './utils';
import * as wasm from './wasm';

declare const RgGameDeclarationBrand: unique symbol;
export type RgGameDeclaration = unknown & { [RgGameDeclarationBrand]: '' };

export type AnalyzedGameStep = { title?: string } & (
  | { kind: 'ast'; language: Exclude<L, L.rg>; value: unknown }
  | { kind: 'ast'; language: L.rg; value: RgGameDeclaration }
  | { kind: 'automaton'; value: string }
  | { kind: 'bench'; stats: string; value: RgGameDeclaration }
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
        const steps = await wasm.analyzeRbg(source);
        game.steps.push(...steps);
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
