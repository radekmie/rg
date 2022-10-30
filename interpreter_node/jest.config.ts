import { Config } from '@jest/types';

const config: Config.InitialOptions = {
  preset: 'ts-jest',
  rootDir: 'src',
  testEnvironment: 'node',
};

export default config;
