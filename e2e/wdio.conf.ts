import path from 'node:path';

const appBinaryPath = path.resolve(
  process.cwd(),
  'target/debug/openrustbooks-desktop'
);

export const config: WebdriverIO.Config = {
  runner: 'local',
  specs: ['./specs/**/*.e2e.js'],
  maxInstances: 1,
  capabilities: [
    {
      browserName: 'tauri',
      'tauri:options': {
        application: appBinaryPath,
      },
    },
  ],
  services: [
    [
      '@wdio/tauri-service',
      {
        appBinaryPath,
        driverProvider: 'embedded',
      },
    ],
  ],
  logLevel: 'warn',
  bail: 1,
  waitforTimeout: 10000,
  connectionRetryTimeout: 60000,
  connectionRetryCount: 1,
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60000,
  },
};
