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

// <TODO> where do I move this? Perhaps to tests where it's used?
function allOptimizations(x: boolean) {
  return {
    compactSkipEdges: x,
    expandGeneratorNodes: x,
    joinForkSuffixes: x,
    mangleSymbols: x,
    reuseFunctions: x,
    skipSelfAssignments: x,
  };
}

export const noOptimizations = allOptimizations(false);
// </TODO> where do I move this?
