'use strict';

const base = require('../../.base-ammanrc.js');

const validator = {
  ...base.validator,
  programs: [],
  accountsCluster: 'https://devnet.genesysgo.net',
  accounts: [
    {
      label: 'Token Metadata Program',
      accountId: 'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
      executable: true,
    },
    {
      label: 'Candy Machine Program',
      accountId: 'cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ',
      executable: true,
    },
  ],
};

module.exports = {
  validator,
  relay: {
    enabled: process.env.CI == null,
    killlRunningRelay: true,
  },
  storage: {
    enabled: process.env.CI == null,
    storageId: 'mock-storage',
    clearOnStart: true,
  },
};
