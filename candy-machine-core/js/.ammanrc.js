'use strict';
// @ts-check
const base = require('../../.base-ammanrc.js');
const validator = {
    ...base.validator,
    programs: [base.programs.candy_machine_core, base.programs.metadata],
    accountsCluster: 'https://api.devnet.solana.com',
    accounts: [
        {
          label: 'Metaplex Denylist',
          accountId:'eBJLFYPxJmMGKuFwpDWkzxZeUrad92kZRC5BJLpzyT9',
          executable: false,
        },
      ]
};
module.exports = {validator};
