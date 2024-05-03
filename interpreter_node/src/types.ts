export type Flag = keyof Settings['flags'];

export enum Language {
  gdl = 'kif',
  hrg = 'hrg',
  rbg = 'rbg',
  rg = 'rg',
}

export type Settings = {
  extension: Language;
  flags: {
    addExplicitCasts: boolean;
    calculateSimpleApply: boolean;
    calculateTagIndexes: boolean;
    calculateUniques: boolean;
    compactSkipEdges: boolean;
    expandGeneratorNodes: boolean;
    inlineReachability: boolean;
    inlineAssignment: boolean;
    joinForkSuffixes: boolean;
    mangleSymbols: boolean;
    normalizeTypes: boolean;
    pruneSingletonTypes: boolean;
    pruneUnreachableNodes: boolean;
    reuseFunctions: boolean;
    skipSelfAssignments: boolean;
    skipSelfComparisons: boolean;
  };
};

export const noFlagsEnabled: Settings['flags'] = {
  addExplicitCasts: false,
  calculateSimpleApply: false,
  calculateTagIndexes: false,
  calculateUniques: false,
  normalizeTypes: false,
  compactSkipEdges: false,
  expandGeneratorNodes: false,
  inlineReachability: false,
  inlineAssignment: false,
  joinForkSuffixes: false,
  mangleSymbols: false,
  pruneSingletonTypes: false,
  pruneUnreachableNodes: false,
  reuseFunctions: false,
  skipSelfAssignments: false,
  skipSelfComparisons: false,
};
