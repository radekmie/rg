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
    calculateIterators: boolean;
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
    pruneSelfLoops: boolean;
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
  calculateIterators: false,
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
  pruneSelfLoops: false,
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
      'pruneSelfLoops',
      'pruneSingletonTypes',
      'pruneUnreachableNodes',
      'pruneUnusedConstants',
      'pruneUnusedVariables',
      'skipArtificialTags',
      'skipRedundantTags',
      'skipSelfAssignments',
      'skipSelfComparisons',
      'skipUnusedTags',
    ],
  },
  {
    label: 'Experimental optimizations',
    flags: ['reorderConditions'],
  },
  {
    label: 'Pragmas',
    flags: [
      'calculateDisjoints',
      'calculateIterators',
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
