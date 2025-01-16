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
    calculateDisjoints: boolean;
    calculateRepeats: boolean;
    calculateSimpleApply: boolean;
    calculateTagIndexes: boolean;
    calculateUniques: boolean;
    compactComparisons: boolean;
    compactSkipEdges: boolean;
    expandGeneratorNodes: boolean;
    inlineReachability: boolean;
    inlineAssignment: boolean;
    joinExclusiveEdges: boolean;
    joinForkPrefixes: boolean;
    joinForkSuffixes: boolean;
    joinGenerators: boolean;
    mangleSymbols: boolean;
    mergeAccesses: boolean;
    normalizeConstants: boolean;
    normalizeTypes: boolean;
    propagateConstants: boolean;
    pruneSingletonTypes: boolean;
    pruneUnreachableNodes: boolean;
    pruneUnusedBindings: boolean;
    pruneUnusedConstants: boolean;
    pruneUnusedVariables: boolean;
    reuseFunctions: boolean;
    skipArtificialTags: boolean;
    skipGeneratorComparisons: boolean;
    skipSelfAssignments: boolean;
    skipSelfComparisons: boolean;
    skipUnusedTags: boolean;
  };
};

export const noFlagsEnabled: Settings['flags'] = {
  addExplicitCasts: false,
  calculateDisjoints: false,
  calculateRepeats: false,
  calculateSimpleApply: false,
  calculateTagIndexes: false,
  calculateUniques: false,
  compactComparisons: false,
  compactSkipEdges: false,
  expandGeneratorNodes: false,
  inlineAssignment: false,
  inlineReachability: false,
  joinExclusiveEdges: false,
  joinForkPrefixes: false,
  joinForkSuffixes: false,
  joinGenerators: false,
  mangleSymbols: false,
  mergeAccesses: false,
  normalizeConstants: false,
  normalizeTypes: false,
  propagateConstants: false,
  pruneSingletonTypes: false,
  pruneUnreachableNodes: false,
  pruneUnusedBindings: false,
  pruneUnusedConstants: false,
  pruneUnusedVariables: false,
  reuseFunctions: false,
  skipArtificialTags: false,
  skipGeneratorComparisons: false,
  skipSelfAssignments: false,
  skipSelfComparisons: false,
  skipUnusedTags: false,
};

export const availableFlags: { label: string; flags: Flag[] }[] = [
  {
    label: 'Optimizations',
    flags: [
      'compactComparisons',
      'compactSkipEdges',
      'inlineAssignment',
      'inlineReachability',
      'joinExclusiveEdges',
      'joinForkPrefixes',
      'joinForkSuffixes',
      'joinGenerators',
      'mergeAccesses',
      'propagateConstants',
      'pruneSingletonTypes',
      'pruneUnreachableNodes',
      'pruneUnusedBindings',
      'pruneUnusedConstants',
      'pruneUnusedVariables',
      'skipArtificialTags',
      'skipGeneratorComparisons',
      'skipSelfAssignments',
      'skipSelfComparisons',
      'skipUnusedTags',
    ],
  },
  {
    label: 'Pragmas',
    flags: [
      'calculateDisjoints',
      'calculateRepeats',
      'calculateSimpleApply',
      'calculateTagIndexes',
      'calculateUniques',
    ],
  },
  {
    label: 'Other',
    flags: [
      'addExplicitCasts',
      'expandGeneratorNodes',
      'mangleSymbols',
      'normalizeConstants',
      'normalizeTypes',
      'reuseFunctions',
    ],
  },
];
