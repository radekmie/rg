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
    mangleSymbols: boolean;
    removeSelfAssignments: boolean;
    reuseFunctions: boolean;
  };
};
