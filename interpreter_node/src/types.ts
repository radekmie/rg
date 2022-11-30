export enum Extension {
  hrg = '.hrg',
  rbg = '.rbg',
  rg = '.rg',
}

export type Flag = keyof Settings['flags'];

export type Settings = {
  extension: Extension;
  flags: {
    compactSkipEdges: boolean;
    expandGeneratorNodes: boolean;
    joinForkSuffixes: boolean;
    mangleSymbols: boolean;
    reuseFunctions: boolean;
    skipSelfAssignments: boolean;
  };
};

export const noFlagsEnabled: Settings['flags'] = {
  compactSkipEdges: false,
  expandGeneratorNodes: false,
  joinForkSuffixes: false,
  mangleSymbols: false,
  reuseFunctions: false,
  skipSelfAssignments: false,
};
