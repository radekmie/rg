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
    calculateUniques: boolean;
    compactSkipEdges: boolean;
    expandGeneratorNodes: boolean;
    inlineReachability: boolean;
    joinForkSuffixes: boolean;
    mangleSymbols: boolean;
    normalizeTypes: boolean;
    pruneUnreachableNodes: boolean;
    reuseFunctions: boolean;
    skipSelfAssignments: boolean;
  };
};

export const noFlagsEnabled: Settings['flags'] = {
  addExplicitCasts: false,
  calculateUniques: false,
  normalizeTypes: false,
  compactSkipEdges: false,
  expandGeneratorNodes: false,
  inlineReachability: false,
  joinForkSuffixes: false,
  mangleSymbols: false,
  pruneUnreachableNodes: false,
  reuseFunctions: false,
  skipSelfAssignments: false,
};
