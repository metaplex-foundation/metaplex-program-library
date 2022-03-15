'use strict';
// @ts-check
const base = require('../../.ammanrc.js');
const { accountProviders } = require('./dist/test/utils/account-providers');

const validator = { ...base.validator, programs: [base.programs.metadata] };
module.exports = {
  validator,
  relay: {
    accountProviders,
  },
};
