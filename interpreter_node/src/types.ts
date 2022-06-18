export enum Extension {
  hrg = '.hrg',
  rg = '.rg',
}

export type Settings = {
  extension: Extension;
  flags: {
    compactSkipEdges: boolean;
    expandGeneratorNodes: boolean;
    mangleSymbols: boolean;
    reuseFunctions: boolean;
  };
};
