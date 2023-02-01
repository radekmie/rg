export enum Extension {
  hrg = '.hrg',
  rbg = '.rbg',
  rg = '.rg',
}

export type Flag = keyof Settings['flags'];

export type Settings = {
  extension: Extension;
  flags: {
    addExplicitCasts: boolean;
    compactSkipEdges: boolean;
    expandGeneratorNodes: boolean;
    inlineReachability: boolean;
    joinForkSuffixes: boolean;
    mangleSymbols: boolean;
    normalizeTypes: boolean;
    reuseFunctions: boolean;
    skipSelfAssignments: boolean;
  };
};

export const noFlagsEnabled: Settings['flags'] = {
  addExplicitCasts: false,
  normalizeTypes: false,
  compactSkipEdges: false,
  expandGeneratorNodes: false,
  inlineReachability: false,
  joinForkSuffixes: false,
  mangleSymbols: false,
  reuseFunctions: false,
  skipSelfAssignments: false,
};
