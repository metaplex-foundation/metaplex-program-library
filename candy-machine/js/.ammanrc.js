'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');
const tokenManager = require('@cardinal/token-manager');
const validator = {
  ...base.validator,
  accountsCluster: 'https://api.metaplex.solana.com',
  programs: [base.programs.metadata, base.programs.candy_machine],
  accounts: [
    {
      label: 'Token Manager',
      accountId: tokenManager.programs.tokenManager.TOKEN_MANAGER_ADDRESS.toString(),
      executable: true,
    },
    {
      label: 'Time Invalidator',
      accountId: tokenManager.programs.timeInvalidator.TIME_INVALIDATOR_ADDRESS.toString(),
      executable: true,
    },
  ],
};
module.exports = { validator };
