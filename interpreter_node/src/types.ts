export enum Extension {
  hrg = '.hrg',
  rg = '.rg',
}

export enum Optimize {
  no = 'no',
  yes = 'yes',
}

export type Settings = {
  extension: Extension;
  optimize: Optimize;
};
