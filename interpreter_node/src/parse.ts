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

  game.steps.push({
    kind: 'source',
    language: settings.extension,
    title: 'original',
    value: source,
  });

  const [steps, error] = await wasm.analyze(source, settings);
  game.steps.push(...steps);

  // If there's no error step (i.e., this error is not an analysis error), add
  // it to the list of steps.
  if (error && game.steps[game.steps.length - 1].kind !== 'error') {
    game.steps.push({ kind: 'error', value: error });
  }

  const graphviz = game.steps.find(x => x.kind === 'graphviz');
  if (graphviz?.kind === 'graphviz') {
    game.steps.splice(game.steps.indexOf(graphviz), 1);
    game.steps.unshift({ kind: 'automaton', value: graphviz.value }, graphviz);
  }

  const stats = game.steps.find(x => x.kind === 'stats');
  if (stats?.kind === 'stats') {
    game.steps.splice(game.steps.indexOf(stats), 1);
    game.steps.unshift({
      kind: 'bench',
      stats: stats.value,
      value: game.astRg,
    });
  }

  return game;
}
