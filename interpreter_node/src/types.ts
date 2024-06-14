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
    calculateRepeats: boolean;
    calculateSimpleApply: boolean;
    calculateTagIndexes: boolean;
    calculateUniques: boolean;
    compactComparisons: boolean;
    compactSkipEdges: boolean;
    expandGeneratorNodes: boolean;
    inlineReachability: boolean;
    inlineAssignment: boolean;
    joinForkPrefixes: boolean;
    joinForkSuffixes: boolean;
    mangleSymbols: boolean;
    normalizeTypes: boolean;
    pruneSingletonTypes: boolean;
    pruneUnreachableNodes: boolean;
    pruneUnusedConstants: boolean;
    pruneUnusedVariables: boolean;
    reuseFunctions: boolean;
    skipGeneratorComparisons: boolean;
    skipSelfAssignments: boolean;
    skipSelfComparisons: boolean;
    skipUnusedTags: boolean;
  };
};

export const noFlagsEnabled: Settings['flags'] = {
  addExplicitCasts: false,
  calculateRepeats: false,
  calculateSimpleApply: false,
  calculateTagIndexes: false,
  calculateUniques: false,
  compactComparisons: false,
  compactSkipEdges: false,
  expandGeneratorNodes: false,
  inlineAssignment: false,
  inlineReachability: false,
  joinForkPrefixes: false,
  joinForkSuffixes: false,
  mangleSymbols: false,
  normalizeTypes: false,
  pruneSingletonTypes: false,
  pruneUnreachableNodes: false,
  pruneUnusedConstants: false,
  pruneUnusedVariables: false,
  reuseFunctions: false,
  skipGeneratorComparisons: false,
  skipSelfAssignments: false,
  skipSelfComparisons: false,
  skipUnusedTags: false,
};
