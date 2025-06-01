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
    calculateRepeatsAndUniques: boolean;
    calculateSimpleApply: boolean;
    calculateTagIndexes: boolean;
    compactComparisons: boolean;
    compactReachability: boolean;
    compactSkipEdges: boolean;
    expandAssignmentAny: boolean;
    expandTagVariable: boolean;
    inlineReachability: boolean;
    inlineAssignment: boolean;
    joinExclusiveEdges: boolean;
    joinForkPrefixes: boolean;
    joinForkSuffixes: boolean;
    mangleSymbols: boolean;
    mergeAccesses: boolean;
    normalizeConstants: boolean;
    normalizeTypes: boolean;
    propagateConstants: boolean;
    pruneSingletonTypes: boolean;
    pruneUnreachableNodes: boolean;
    pruneUnusedConstants: boolean;
    pruneUnusedVariables: boolean;
    reorderConditions: boolean;
    skipArtificialTags: boolean;
    skipRedundantTags: boolean;
    skipSelfAssignments: boolean;
    skipSelfComparisons: boolean;
    skipUnusedTags: boolean;
  };
};

export const noFlagsEnabled: Settings['flags'] = {
  addExplicitCasts: false,
  calculateDisjoints: false,
  calculateRepeatsAndUniques: false,
  calculateSimpleApply: false,
  calculateTagIndexes: false,
  compactComparisons: false,
  compactReachability: false,
  compactSkipEdges: false,
  expandAssignmentAny: false,
  expandTagVariable: false,
  inlineAssignment: false,
  inlineReachability: false,
  joinExclusiveEdges: false,
  joinForkPrefixes: false,
  joinForkSuffixes: false,
  mangleSymbols: false,
  mergeAccesses: false,
  normalizeConstants: false,
  normalizeTypes: false,
  propagateConstants: false,
  pruneSingletonTypes: false,
  pruneUnreachableNodes: false,
  pruneUnusedConstants: false,
  pruneUnusedVariables: false,
  reorderConditions: false,
  skipArtificialTags: false,
  skipRedundantTags: false,
  skipSelfAssignments: false,
  skipSelfComparisons: false,
  skipUnusedTags: false,
};

export const availableFlags: { label: string; flags: Flag[] }[] = [
  {
    label: 'Optimizations',
    flags: [
      'compactComparisons',
      'compactReachability',
      'compactSkipEdges',
      'inlineAssignment',
      'inlineReachability',
      'joinExclusiveEdges',
      'joinForkPrefixes',
      'joinForkSuffixes',
      'mergeAccesses',
      'propagateConstants',
      'pruneSingletonTypes',
      'pruneUnreachableNodes',
      'pruneUnusedConstants',
      'pruneUnusedVariables',
      'reorderConditions',
      'skipArtificialTags',
      'skipRedundantTags',
      'skipSelfAssignments',
      'skipSelfComparisons',
      'skipUnusedTags',
    ],
  },
  {
    label: 'Pragmas',
    flags: [
      'calculateDisjoints',
      'calculateRepeatsAndUniques',
      'calculateSimpleApply',
      'calculateTagIndexes',
    ],
  },
  {
    label: 'Other',
    flags: [
      'addExplicitCasts',
      'expandAssignmentAny',
      'expandTagVariable',
      'mangleSymbols',
      'normalizeConstants',
      'normalizeTypes',
    ],
  },
];
