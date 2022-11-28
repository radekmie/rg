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

function allFlags(x: boolean) {
  return {
    compactSkipEdges: x,
    expandGeneratorNodes: x,
    joinForkSuffixes: x,
    mangleSymbols: x,
    reuseFunctions: x,
    skipSelfAssignments: x,
  };
}

export const noFlags = allFlags(false);
